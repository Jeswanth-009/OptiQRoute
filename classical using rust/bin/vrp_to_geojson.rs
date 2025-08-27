use clap::{Arg, Command};
use geojson::{GeoJson, Geometry, Value, Feature, FeatureCollection};
use serde::Deserialize;
use serde_json::Map;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use vrp_solver::osm_parser::OsmData;

#[derive(Debug, Deserialize)]
struct VrpSolution {
    routes: Vec<VrpRoute>,
    total_distance: f64,
    total_duration: f64,
    num_vehicles_used: u32,
}

#[derive(Debug, Deserialize)]
struct VrpRoute {
    vehicle_id: u32,
    locations: Vec<i64>, // Location IDs from the VRP solution (can be OSM node IDs)
    total_distance: f64,
    total_duration: f64,
    total_demand: f64,
}

struct CoordinateLookup {
    osm_data: OsmData,
    location_id_to_node_id: HashMap<i64, i64>, // Map VRP location IDs to OSM node IDs
}

impl CoordinateLookup {
    fn new(osm_data: OsmData) -> Self {
        // For now, assume VRP location IDs correspond directly to OSM node IDs
        // In a real implementation, you'd have a proper mapping table
        let location_id_to_node_id = HashMap::new();
        
        Self {
            osm_data,
            location_id_to_node_id,
        }
    }
    
    fn with_id_mapping(osm_data: OsmData, mapping: HashMap<i64, i64>) -> Self {
        Self {
            osm_data,
            location_id_to_node_id: mapping,
        }
    }
    
    fn get_coordinates(&self, location_id: i64) -> Option<(f64, f64)> {
        // First, try direct mapping if we have it
        if let Some(&node_id) = self.location_id_to_node_id.get(&location_id) {
            return self.osm_data.nodes.get(&node_id).map(|node| (node.lat, node.lon));
        }
        
        // Fallback: assume location_id is directly the node_id
        self.osm_data.nodes.get(&location_id).map(|node| (node.lat, node.lon))
    }
    
    fn find_nearest_osm_node(&self, target_lat: f64, target_lon: f64) -> Option<(i64, f64)> {
        let mut nearest_node = None;
        let mut min_distance = f64::MAX;
        
        for (&node_id, node) in &self.osm_data.nodes {
            let distance = self.haversine_distance(target_lat, target_lon, node.lat, node.lon);
            if distance < min_distance {
                min_distance = distance;
                nearest_node = Some((node_id, distance));
            }
        }
        
        nearest_node
    }
    
    fn haversine_distance(&self, lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
        let r = 6371000.0; // Earth's radius in meters
        let phi1 = lat1.to_radians();
        let phi2 = lat2.to_radians();
        let delta_phi = (lat2 - lat1).to_radians();
        let delta_lambda = (lon2 - lon1).to_radians();

        let a = (delta_phi / 2.0).sin().powi(2) +
                phi1.cos() * phi2.cos() *
                (delta_lambda / 2.0).sin().powi(2);
        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

        r * c
    }
}

fn convert_vrp_solution_to_geojson(
    solution: &VrpSolution,
    lookup: &CoordinateLookup,
    include_points: bool,
    depot_coords: Option<(f64, f64)>,
) -> Result<FeatureCollection, Box<dyn std::error::Error>> {
    let mut features = Vec::new();
    
    // Process each route
    for (route_idx, route) in solution.routes.iter().enumerate() {
        println!("Processing route {} with {} locations", route_idx, route.locations.len());
        
        let mut coordinates = Vec::new();
        let mut missing_coords = Vec::new();
        
        // Add depot at the beginning if provided
        if let Some(depot) = depot_coords {
            coordinates.push(vec![depot.1, depot.0]); // GeoJSON uses [lon, lat]
        }
        
        // Get coordinates for each location in the route
        for &location_id in &route.locations {
            if let Some((lat, lon)) = lookup.get_coordinates(location_id) {
                coordinates.push(vec![lon, lat]); // GeoJSON uses [lon, lat]
                
                // Add individual point feature if requested
                if include_points {
                    let mut properties = Map::new();
                    properties.insert("type".to_string(), serde_json::Value::String("customer".to_string()));
                    properties.insert("location_id".to_string(), serde_json::Value::Number(location_id.into()));
                    properties.insert("route_id".to_string(), serde_json::Value::Number(route_idx.into()));
                    properties.insert("vehicle_id".to_string(), serde_json::Value::Number(route.vehicle_id.into()));
                    
                    let point_feature = Feature {
                        bbox: None,
                        geometry: Some(Geometry::new(Value::Point(vec![lon, lat]))),
                        id: None,
                        properties: Some(properties),
                        foreign_members: None,
                    };
                    
                    features.push(point_feature);
                }
            } else {
                missing_coords.push(location_id);
            }
        }
        
        // Add depot at the end if provided (complete the route)
        if let Some(depot) = depot_coords {
            coordinates.push(vec![depot.1, depot.0]);
        }
        
        // Create LineString feature for the route
        if coordinates.len() >= 2 {
            let mut properties = Map::new();
            properties.insert("type".to_string(), serde_json::Value::String("route".to_string()));
            properties.insert("route_id".to_string(), serde_json::Value::Number(route_idx.into()));
            properties.insert("vehicle_id".to_string(), serde_json::Value::Number(route.vehicle_id.into()));
            properties.insert("total_distance".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(route.total_distance).unwrap()));
            properties.insert("total_duration".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(route.total_duration).unwrap()));
            properties.insert("total_demand".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(route.total_demand).unwrap()));
            properties.insert("num_locations".to_string(), serde_json::Value::Number(route.locations.len().into()));
            
            if !missing_coords.is_empty() {
                properties.insert("missing_coordinates".to_string(), 
                    serde_json::Value::Array(missing_coords.iter().map(|&id| serde_json::Value::Number(id.into())).collect()));
            }
            
            let num_coords = coordinates.len();
            let route_feature = Feature {
                bbox: None,
                geometry: Some(Geometry::new(Value::LineString(coordinates))),
                id: None,
                properties: Some(properties),
                foreign_members: None,
            };
            
            features.push(route_feature);
            
            println!("Created route {} with {} coordinate points", route_idx, num_coords);
            if !missing_coords.is_empty() {
                println!("  Warning: Missing coordinates for location IDs: {:?}", missing_coords);
            }
        } else {
            println!("  Warning: Route {} has insufficient coordinates ({} points)", route_idx, coordinates.len());
        }
    }
    
    // Add depot point if provided
    if let Some(depot) = depot_coords {
        let mut properties = Map::new();
        properties.insert("type".to_string(), serde_json::Value::String("depot".to_string()));
        properties.insert("name".to_string(), serde_json::Value::String("Main Depot".to_string()));
        
        let depot_feature = Feature {
            bbox: None,
            geometry: Some(Geometry::new(Value::Point(vec![depot.1, depot.0]))),
            id: None,
            properties: Some(properties),
            foreign_members: None,
        };
        
        features.push(depot_feature);
    }
    
    Ok(FeatureCollection {
        bbox: None,
        features,
        foreign_members: None,
    })
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("VRP to GeoJSON Converter")
        .version("1.0")
        .author("VRP Solver")
        .about("Converts VRP solution JSON to GeoJSON format using OSM coordinate lookup")
        .arg(
            Arg::new("solution")
                .short('s')
                .long("solution")
                .value_name("FILE")
                .help("VRP solution JSON file")
                .default_value("example_solution.json"),
        )
        .arg(
            Arg::new("osm_data")
                .short('o')
                .long("osm-data")
                .value_name("FILE")
                .help("OSM data JSON file for coordinate lookup")
                .default_value("southern-zone-latest.osm.pbf.json"),
        )
        .arg(
            Arg::new("output")
                .short('g')
                .long("geojson")
                .value_name("FILE")
                .help("Output GeoJSON file")
                .default_value("routes.geojson"),
        )
        .arg(
            Arg::new("include_points")
                .short('p')
                .long("include-points")
                .help("Include individual Point features for customers")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("depot_lat")
                .long("depot-lat")
                .value_name("LATITUDE")
                .help("Depot latitude coordinate")
                .required(false),
        )
        .arg(
            Arg::new("depot_lon")
                .long("depot-lon")
                .value_name("LONGITUDE")
                .help("Depot longitude coordinate")
                .required(false),
        )
        .arg(
            Arg::new("sample_osm")
                .long("sample-osm")
                .help("Only load a sample of OSM data for testing (first 1000 nodes)")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    let solution_file = matches.get_one::<String>("solution").unwrap();
    let osm_data_file = matches.get_one::<String>("osm_data").unwrap();
    let output_file = matches.get_one::<String>("output").unwrap();
    let include_points = matches.get_flag("include_points");
    let sample_osm = matches.get_flag("sample_osm");
    
    // Parse depot coordinates if provided
    let depot_coords = match (matches.get_one::<String>("depot_lat"), matches.get_one::<String>("depot_lon")) {
        (Some(lat_str), Some(lon_str)) => {
            let lat: f64 = lat_str.parse()?;
            let lon: f64 = lon_str.parse()?;
            Some((lat, lon))
        }
        _ => None,
    };

    println!("🚀 Starting VRP to GeoJSON conversion...");
    println!("📁 Solution file: {}", solution_file);
    println!("📁 OSM data file: {}", osm_data_file);
    println!("📁 Output file: {}", output_file);

    // Check if files exist
    if !std::path::Path::new(solution_file).exists() {
        eprintln!("❌ VRP solution file not found: {}", solution_file);
        return Ok(());
    }
    
    if !std::path::Path::new(osm_data_file).exists() {
        eprintln!("❌ OSM data file not found: {}", osm_data_file);
        eprintln!("💡 Run the OSM converter first: cargo run --bin osm_converter -- --input southern-zone-latest.osm.pbf --roads-only");
        return Ok(());
    }

    // Load VRP solution
    println!("📖 Loading VRP solution...");
    let solution_file_handle = File::open(solution_file)?;
    let solution_reader = BufReader::new(solution_file_handle);
    let solution: VrpSolution = serde_json::from_reader(solution_reader)?;
    
    println!("✅ Loaded VRP solution:");
    println!("  - Routes: {}", solution.routes.len());
    println!("  - Vehicles used: {}", solution.num_vehicles_used);
    println!("  - Total distance: {:.2}", solution.total_distance);

    // Load OSM data
    println!("📖 Loading OSM data...");
    if sample_osm {
        println!("⚠️  Loading sample OSM data (first 1000 nodes for testing)...");
    } else {
        println!("⚠️  Loading full OSM data (this may take a while and use significant memory)...");
    }
    
    let osm_file_handle = File::open(osm_data_file)?;
    let osm_reader = BufReader::new(osm_file_handle);
    
    let osm_data: OsmData = if sample_osm {
        // For testing with large files, load only a sample
        let mut full_data: OsmData = serde_json::from_reader(osm_reader)?;
        let sample_nodes: HashMap<i64, _> = full_data.nodes.into_iter().take(1000).collect();
        full_data.nodes = sample_nodes;
        full_data
    } else {
        match serde_json::from_reader(osm_reader) {
            Ok(data) => data,
            Err(e) => {
                eprintln!("❌ Failed to load OSM data: {}", e);
                eprintln!("💡 The OSM JSON file might be too large. Try using --sample-osm for testing.");
                return Err(Box::new(e));
            }
        }
    };
    
    println!("✅ Loaded OSM data:");
    println!("  - Nodes: {}", osm_data.nodes.len());
    println!("  - Ways: {}", osm_data.ways.len());

    // Create coordinate lookup
    let lookup = CoordinateLookup::new(osm_data);
    
    // Show which location IDs we're trying to map
    println!("🔍 Analyzing location IDs in solution...");
    let mut all_location_ids = std::collections::HashSet::new();
    for route in &solution.routes {
        for &location_id in &route.locations {
            all_location_ids.insert(location_id);
        }
    }
    
    println!("  - Unique location IDs: {:?}", {
        let mut ids: Vec<_> = all_location_ids.iter().collect();
        ids.sort();
        ids
    });
    
    // Check which IDs have coordinates available
    let mut found_coords = 0;
    let mut missing_coords = Vec::new();
    for &location_id in &all_location_ids {
        if lookup.get_coordinates(location_id).is_some() {
            found_coords += 1;
        } else {
            missing_coords.push(location_id);
        }
    }
    
    println!("  - IDs with coordinates: {}/{}", found_coords, all_location_ids.len());
    if !missing_coords.is_empty() {
        println!("  - Missing coordinates for IDs: {:?}", missing_coords);
        
        // If we have missing coordinates, try to find the first few nodes in OSM data as examples
        println!("  - Sample OSM node IDs: {:?}", {
            let sample_node_ids: Vec<_> = lookup.osm_data.nodes.keys().take(10).collect();
            sample_node_ids
        });
    }

    // Convert to GeoJSON
    println!("🌍 Converting to GeoJSON...");
    let feature_collection = convert_vrp_solution_to_geojson(&solution, &lookup, include_points, depot_coords)?;
    
    println!("✅ Created GeoJSON:");
    println!("  - Features: {}", feature_collection.features.len());
    
    // Write to file
    println!("💾 Writing to file: {}", output_file);
    let geojson = GeoJson::FeatureCollection(feature_collection);
    let geojson_str = serde_json::to_string_pretty(&geojson)?;
    std::fs::write(output_file, geojson_str)?;
    
    println!("✅ Conversion completed successfully!");
    println!("📍 Open {} in a GIS application or web map to visualize the routes.", output_file);

    Ok(())
}
