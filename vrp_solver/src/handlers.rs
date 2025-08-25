//! Web API handlers for the VRP solver

use axum::{
    extract::{Path, Query, State, Multipart},
    http::StatusCode,
    response::{IntoResponse, Json, Response},
    routing::{get, post},
    Router,
};
use std::time::{Instant, SystemTime};
use uuid::Uuid;
use tracing::info;
use tempfile::NamedTempFile;
use std::io::Write;

use crate::{
    api_types::*,
    app_state::AppState,
    osm_parser::OsmParser,
    solver::*,
    distance::DistanceMethod,
    utils::VrpInstanceBuilder,
    types::*,
    VrpError,
};

// Error handling for handlers
impl IntoResponse for VrpError {
    fn into_response(self) -> Response {
        let (status, error_response) = match self {
            VrpError::InvalidInput(msg) => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new("invalid_input", &msg),
            ),
            VrpError::NoSolutionFound => (
                StatusCode::NOT_FOUND,
                ErrorResponse::new("no_solution", "No solution could be found for the given problem"),
            ),
            VrpError::CapacityViolation { required, available } => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::with_details(
                    "capacity_violation",
                    "Vehicle capacity constraint violated",
                    &format!("Required: {}, Available: {}", required, available),
                ),
            ),
            VrpError::TimeWindowViolation { arrival, start, end } => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::with_details(
                    "time_window_violation",
                    "Time window constraint violated",
                    &format!("Arrival: {}, Window: [{}, {}]", arrival, start, end),
                ),
            ),
            VrpError::DistanceLimitExceeded { distance, limit } => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::with_details(
                    "distance_limit_exceeded",
                    "Distance limit constraint violated",
                    &format!("Distance: {}, Limit: {}", distance, limit),
                ),
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse::new("internal_error", "An internal server error occurred"),
            ),
        };

        (status, Json(error_response)).into_response()
    }
}

// Custom error type for web handlers
#[derive(Debug)]
pub enum HandlerError {
    Vrp(VrpError),
    StateError(String),
    ParseError(String),
    NotFound(String),
    InternalError(String),
}

impl IntoResponse for HandlerError {
    fn into_response(self) -> Response {
        let (status, error_response) = match self {
            HandlerError::Vrp(vrp_err) => return vrp_err.into_response(),
            HandlerError::StateError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse::new("state_error", &msg),
            ),
            HandlerError::ParseError(msg) => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new("parse_error", &msg),
            ),
            HandlerError::NotFound(msg) => (
                StatusCode::NOT_FOUND,
                ErrorResponse::new("not_found", &msg),
            ),
            HandlerError::InternalError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse::new("internal_error", &msg),
            ),
        };

        (status, Json(error_response)).into_response()
    }
}

impl From<VrpError> for HandlerError {
    fn from(err: VrpError) -> Self {
        HandlerError::Vrp(err)
    }
}

/// Create routes for the VRP API  
pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/health", get(health_check))
        .route("/stats", get(get_stats))
        .route("/osm/upload", post(upload_osm))
        .route("/vrp/map", post(map_locations))
        .route("/vrp/generate", post(generate_vrp))
        .route("/vrp/solve", post(solve_vrp))
        .route("/vrp/solution/:solution_id", get(get_solution))
        .route("/vrp/solution/:solution_id/export", get(export_solution))
}

// Health check endpoint
async fn health_check(State(state): State<AppState>) -> Result<Json<serde_json::Value>, HandlerError> {
    let stats = state.get_stats()
        .map_err(|e| HandlerError::StateError(e))?;

    Ok(Json(serde_json::json!({
        "status": "healthy",
        "timestamp": SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        "stats": stats
    })))
}

// Get application statistics
async fn get_stats(State(state): State<AppState>) -> Result<Json<crate::app_state::AppStateStats>, HandlerError> {
    let stats = state.get_stats()
        .map_err(|e| HandlerError::StateError(e))?;
    Ok(Json(stats))
}

// OSM Upload endpoint - handles both file upload and URL
async fn upload_osm(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<OsmUploadResponse>, HandlerError> {
    info!("Received OSM upload request");

    let mut file_data: Option<Vec<u8>> = None;
    let mut file_url: Option<String> = None;

    // Process multipart form data
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        HandlerError::ParseError(format!("Failed to read multipart data: {}", e))
    })? {
        let name = field.name().unwrap_or("").to_string();
        
        match name.as_str() {
            "file" => {
                let data = field.bytes().await.map_err(|e| {
                    HandlerError::ParseError(format!("Failed to read file data: {}", e))
                })?;
                file_data = Some(data.to_vec());
            }
            "file_url" => {
                file_url = Some(field.text().await.map_err(|e| {
                    HandlerError::ParseError(format!("Failed to read URL: {}", e))
                })?);
            }
            _ => {
                // Ignore unknown fields
            }
        }
    }

    // Handle file upload or URL download
    let (_temp_file, temp_file_path) = if let Some(data) = file_data {
        info!("Processing uploaded file ({} bytes)", data.len());
        
        let mut temp_file = NamedTempFile::new()
            .map_err(|e| HandlerError::InternalError(format!("Failed to create temp file: {}", e)))?;
        
        temp_file.write_all(&data)
            .map_err(|e| HandlerError::InternalError(format!("Failed to write temp file: {}", e)))?;
        
        let temp_path = temp_file.path().to_string_lossy().to_string();
        (Some(temp_file), temp_path)
    } else if let Some(url) = file_url {
        info!("Downloading OSM data from URL: {}", url);
        
        // Download file from URL
        let response = reqwest::get(&url).await
            .map_err(|e| HandlerError::InternalError(format!("Failed to download file: {}", e)))?;
        
        let data = response.bytes().await
            .map_err(|e| HandlerError::InternalError(format!("Failed to read downloaded data: {}", e)))?;
        
        let mut temp_file = NamedTempFile::new()
            .map_err(|e| HandlerError::InternalError(format!("Failed to create temp file: {}", e)))?;
        
        temp_file.write_all(&data)
            .map_err(|e| HandlerError::InternalError(format!("Failed to write temp file: {}", e)))?;
        
        let temp_path = temp_file.path().to_string_lossy().to_string();
        (Some(temp_file), temp_path)
    } else {
        return Err(HandlerError::ParseError("No file or URL provided".to_string()));
    };

    // Parse OSM data
    let mut parser = OsmParser::new();
    parser.parse_pbf_file(&temp_file_path)
        .map_err(|e| HandlerError::InternalError(format!("Failed to parse OSM file: {}", e)))?;

    // Filter to roads only
    parser.filter_roads_only();

    let node_count = parser.data.nodes.len();
    let way_count = parser.data.ways.len();

    info!("Parsed OSM data: {} nodes, {} ways", node_count, way_count);

    // Store the graph
    let graph_id = Uuid::new_v4();
    let stored_graph = StoredGraph {
        id: graph_id,
        osm_data: parser.data,
        created_at: SystemTime::now(),
        node_count,
        way_count,
    };

    state.store_graph(stored_graph)
        .map_err(|e| HandlerError::StateError(e))?;

    info!("Stored OSM graph with ID: {}", graph_id);

    Ok(Json(OsmUploadResponse {
        graph_id,
        nodes: node_count,
        edges: way_count,
        message: format!("Successfully parsed OSM data: {} nodes, {} ways", node_count, way_count),
    }))
}

// Map depot and customer locations to OSM nodes
async fn map_locations(
    State(state): State<AppState>,
    Json(request): Json<MapLocationRequest>,
) -> Result<Json<MapLocationResponse>, HandlerError> {
    info!("Mapping locations for graph: {}", request.graph_id);

    // Get the stored graph
    let stored_graph = state.get_graph(&request.graph_id)
        .map_err(|e| HandlerError::StateError(e))?
        .ok_or_else(|| HandlerError::NotFound(format!("Graph {} not found", request.graph_id)))?;

    let parser = OsmParser { data: stored_graph.osm_data.clone() };

    // Map depot location
    let depot_coord = Coordinate::from(&request.depot);
    let (depot_node_id, depot_distance) = parser.find_nearest_node(depot_coord.lat, depot_coord.lon)
        .ok_or_else(|| HandlerError::InternalError("No nodes found in graph".to_string()))?;

    let (depot_lat, depot_lon) = parser.get_node_coordinates(depot_node_id)
        .ok_or_else(|| HandlerError::InternalError("Depot node coordinates not found".to_string()))?;

    let mapped_depot = MappedLocation {
        node_id: depot_node_id,
        lat: depot_lat,
        lon: depot_lon,
        distance_to_original: depot_distance,
    };

    // Map customer locations
    let mut mapped_customers = Vec::new();
    for customer in &request.customers {
        let coord = Coordinate::from(customer);
        let (node_id, distance) = parser.find_nearest_node(coord.lat, coord.lon)
            .ok_or_else(|| HandlerError::InternalError("No nodes found for customer".to_string()))?;

        let (lat, lon) = parser.get_node_coordinates(node_id)
            .ok_or_else(|| HandlerError::InternalError("Customer node coordinates not found".to_string()))?;

        mapped_customers.push(MappedLocation {
            node_id,
            lat,
            lon,
            distance_to_original: distance,
        });
    }

    // Store the mapping
    let mapping = StoredMapping {
        graph_id: request.graph_id,
        depot: mapped_depot.clone(),
        customers: mapped_customers.clone(),
        created_at: SystemTime::now(),
    };

    state.store_mapping(mapping)
        .map_err(|e| HandlerError::StateError(e))?;

    info!("Mapped {} customers and 1 depot", mapped_customers.len());

    Ok(Json(MapLocationResponse {
        mapped_depot,
        mapped_customers,
    }))
}

// Generate VRP instance from mapped locations
async fn generate_vrp(
    State(state): State<AppState>,
    Json(request): Json<GenerateVrpRequest>,
) -> Result<Json<GenerateVrpResponse>, HandlerError> {
    info!("Generating VRP instance for graph: {}", request.graph_id);

    // Get the stored mapping
    let mapping = state.get_mapping(&request.graph_id)
        .map_err(|e| HandlerError::StateError(e))?
        .ok_or_else(|| HandlerError::NotFound(format!("Mapping for graph {} not found", request.graph_id)))?;

    // Create VRP instance
    let mut builder = VrpInstanceBuilder::new();

    // Add depot
    builder = builder.add_depot(
        0,
        "Depot".to_string(),
        Coordinate::new(mapping.depot.lat, mapping.depot.lon),
    );

    // Add customers with default demand
    for (i, customer) in mapping.customers.iter().enumerate() {
        let demand = 10.0; // Default demand - could be made configurable
        let service_time = request.constraints.service_time.unwrap_or(300.0); // 5 minutes default
        
        builder = builder.add_customer(
            i + 1,
            format!("Customer {}", i + 1),
            Coordinate::new(customer.lat, customer.lon),
            demand,
            None, // No time windows for now
            service_time,
        );
    }

    // Add vehicles
    for i in 0..request.vehicles {
        let mut vehicle = Vehicle::new(i, request.capacity, None, None, 0);
        
        if let Some(max_distance) = request.constraints.max_distance {
            vehicle.max_distance = Some(max_distance);
        }
        
        if let Some(max_duration) = request.constraints.max_duration {
            vehicle.max_duration = Some(max_duration);
        }
        
        builder = builder.add_vehicle(vehicle);
    }

    // Build the VRP instance
    let instance = builder
        .with_distance_method(DistanceMethod::Haversine)
        .with_average_speed(15.0) // 15 m/s ≈ 54 km/h
        .build()?;

    let customers = mapping.customers.len();
    let vehicles = request.vehicles;

    // Store the VRP instance
    let vrp_id = Uuid::new_v4();
    let stored_instance = StoredVrpInstance {
        id: vrp_id,
        mapping,
        instance,
        constraints: request.constraints,
        created_at: SystemTime::now(),
    };

    state.store_vrp_instance(stored_instance)
        .map_err(|e| HandlerError::StateError(e))?;

    info!("Generated VRP instance {} with {} customers and {} vehicles", vrp_id, customers, vehicles);

    Ok(Json(GenerateVrpResponse {
        vrp_id,
        customers,
        vehicles,
        depot_count: 1,
    }))
}

// Solve VRP instance
async fn solve_vrp(
    State(state): State<AppState>,
    Json(request): Json<SolveVrpRequest>,
) -> Result<Json<SolveVrpResponse>, HandlerError> {
    info!("Solving VRP instance {} with algorithm {:?}", request.vrp_id, request.algorithm);

    // Get the stored VRP instance
    let stored_instance = state.get_vrp_instance(&request.vrp_id)
        .map_err(|e| HandlerError::StateError(e))?
        .ok_or_else(|| HandlerError::NotFound(format!("VRP instance {} not found", request.vrp_id)))?;

    // Select and create solver
    let solver: Box<dyn VrpSolver + Sync> = match request.algorithm {
        SolverAlgorithm::Greedy => Box::new(GreedyNearestNeighbor::new()),
        SolverAlgorithm::GreedyFarthest => Box::new(GreedyNearestNeighbor::new().with_farthest_start(true)),
        SolverAlgorithm::ClarkeWright => Box::new(ClarkeWrightSavings::new()),
        SolverAlgorithm::MultiStart => Box::new(MultiStartSolver::new().with_default_solvers()),
    };

    // Solve the VRP
    let start_time = Instant::now();
    let solution = solver.solve(&stored_instance.instance)?;
    let solve_time = start_time.elapsed();
    let solve_time_ms = solve_time.as_secs_f64() * 1000.0;

    info!("Solved VRP in {:.2}ms, found {} routes", solve_time_ms, solution.routes.len());

    // Convert to API format
    let api_routes = solution.routes.iter().map(|route| {
        let locations: Vec<ApiLocation> = route.locations.iter()
            .filter_map(|&loc_id| stored_instance.instance.get_location(loc_id))
            .map(ApiLocation::from)
            .collect();

        ApiRoute {
            vehicle_id: route.vehicle_id,
            path: route.locations.clone(),
            distance: route.total_distance,
            duration: route.total_duration,
            demand: route.total_demand,
            locations,
        }
    }).collect();

    // Store the solution
    let solution_id = Uuid::new_v4();
    let stored_solution = StoredSolution {
        id: solution_id,
        vrp_id: request.vrp_id,
        solution,
        algorithm: request.algorithm,
        solve_time_ms,
        created_at: SystemTime::now(),
    };

    let total_cost = stored_solution.solution.total_distance;
    let total_distance = stored_solution.solution.total_distance;
    let total_duration = stored_solution.solution.total_duration;
    let vehicles_used = stored_solution.solution.num_vehicles_used;

    state.store_solution(stored_solution)
        .map_err(|e| HandlerError::StateError(e))?;

    Ok(Json(SolveVrpResponse {
        solution_id,
        routes: api_routes,
        total_cost,
        total_distance,
        total_duration,
        vehicles_used,
        solve_time_ms,
    }))
}

// Get solution details
async fn get_solution(
    State(state): State<AppState>,
    Path(solution_id): Path<Uuid>,
) -> Result<Json<StoredSolution>, HandlerError> {
    info!("Getting solution: {}", solution_id);

    let solution = state.get_solution(&solution_id)
        .map_err(|e| HandlerError::StateError(e))?
        .ok_or_else(|| HandlerError::NotFound(format!("Solution {} not found", solution_id)))?;

    Ok(Json(solution))
}

// Export solution in different formats
async fn export_solution(
    State(state): State<AppState>,
    Path(solution_id): Path<Uuid>,
    Query(params): Query<ExportFormat>,
) -> Result<Response, HandlerError> {
    info!("Exporting solution: {} in format: {:?}", solution_id, params.format);

    let stored_solution = state.get_solution(&solution_id)
        .map_err(|e| HandlerError::StateError(e))?
        .ok_or_else(|| HandlerError::NotFound(format!("Solution {} not found", solution_id)))?;

    let stored_vrp = state.get_vrp_instance(&stored_solution.vrp_id)
        .map_err(|e| HandlerError::StateError(e))?
        .ok_or_else(|| HandlerError::NotFound(format!("VRP instance {} not found", stored_solution.vrp_id)))?;

    let format = params.format.as_deref().unwrap_or("json");

    match format {
        "geojson" => {
            let geojson = create_geojson_from_solution(&stored_solution.solution, &stored_vrp.instance)?;
            Ok(Json(geojson).into_response())
        }
        "json" => {
            Ok(Json(&stored_solution.solution).into_response())
        }
        _ => {
            Err(HandlerError::ParseError(format!("Unsupported export format: {}", format)))
        }
    }
}

// Helper function to create GeoJSON from solution
fn create_geojson_from_solution(
    solution: &Solution, 
    instance: &VrpInstance
) -> Result<geojson::GeoJson, HandlerError> {
    use geojson::{GeoJson, Geometry, Value, Feature, FeatureCollection};
    use serde_json::Map;

    let mut features = Vec::new();

    // Add route features
    for (route_idx, route) in solution.routes.iter().enumerate() {
        let mut coordinates = Vec::new();
        
        for &location_id in &route.locations {
            if let Some(location) = instance.get_location(location_id) {
                coordinates.push(vec![location.coordinate.lon, location.coordinate.lat]);
            }
        }

        if coordinates.len() >= 2 {
            let geometry = Geometry::new(Value::LineString(coordinates));
            
            let mut properties = Map::new();
            properties.insert("route_id".to_string(), serde_json::Value::Number((route_idx + 1).into()));
            properties.insert("vehicle_id".to_string(), serde_json::Value::Number(route.vehicle_id.into()));
            if let Some(distance_num) = serde_json::Number::from_f64(route.total_distance) {
                properties.insert("distance".to_string(), serde_json::Value::Number(distance_num));
            }
            if let Some(duration_num) = serde_json::Number::from_f64(route.total_duration) {
                properties.insert("duration".to_string(), serde_json::Value::Number(duration_num));
            }
            if let Some(demand_num) = serde_json::Number::from_f64(route.total_demand) {
                properties.insert("demand".to_string(), serde_json::Value::Number(demand_num));
            }

            let feature = Feature {
                bbox: None,
                geometry: Some(geometry),
                id: None,
                properties: Some(properties),
                foreign_members: None,
            };

            features.push(feature);
        }
    }

    // Add location points
    for location in &instance.locations {
        let geometry = Geometry::new(Value::Point(vec![location.coordinate.lon, location.coordinate.lat]));
        
        let mut properties = Map::new();
        properties.insert("id".to_string(), serde_json::Value::Number(location.id.into()));
        properties.insert("name".to_string(), serde_json::Value::String(location.name.clone()));
        if let Some(demand_num) = serde_json::Number::from_f64(location.demand) {
            properties.insert("demand".to_string(), serde_json::Value::Number(demand_num));
        }
        properties.insert("type".to_string(), serde_json::Value::String(
            if location.demand > 0.0 { "customer" } else { "depot" }.to_string()
        ));

        let feature = Feature {
            bbox: None,
            geometry: Some(geometry),
            id: None,
            properties: Some(properties),
            foreign_members: None,
        };

        features.push(feature);
    }

    let feature_collection = FeatureCollection {
        bbox: None,
        features,
        foreign_members: None,
    };

    Ok(GeoJson::FeatureCollection(feature_collection))
}
