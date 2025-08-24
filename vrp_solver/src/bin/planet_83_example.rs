use clap::{Arg, Command};
use serde_json::Map;
use std::fs::File;
use std::io::BufReader;
use vrp_solver::{
    distance::DistanceMethod,
    solver::{GreedyNearestNeighbor, MultiStartSolver, VrpSolver},
    types::{Coordinate, Solution},
    utils::{VrpInstanceBuilder, save_solution_to_json},
    osm_parser::OsmData,
};
use geojson::{GeoJson, Geometry, Value, Feature, FeatureCollection};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("Planet 83 OSM VRP Example")
        .version("1.0")
        .author("VRP Solver")
        .about("Complete workflow demo: Parse OSM → Create VRP → Solve → Export GeoJSON")
        .arg(
            Arg::new("osm_data")
                .short('o')
                .long("osm-data")
                .value_name("FILE")
                .help("OSM data JSON file")
                .default_value("planet_83.2932,17.7118_83.3388,17.7502.osm.pbf.json"),
        )
        .arg(
            Arg::new("num_customers")
                .short('n')
                .long("customers")
                .value_name("COUNT")
                .help("Number of customers to generate")
                .default_value("8"),
        )
        .arg(
            Arg::new("depot_lat")
                .long("depot-lat")
                .value_name("LATITUDE")
                .help("Depot latitude")
                .default_value("17.735"),
        )
        .arg(
            Arg::new("depot_lon")
                .long("depot-lon")
                .value_name("LONGITUDE")
                .help("Depot longitude")
                .default_value("83.315"),
        )
        .get_matches();

    let osm_data_file = matches.get_one::<String>("osm_data").unwrap();
    let num_customers: usize = matches.get_one::<String>("num_customers").unwrap().parse()?;
    let depot_lat: f64 = matches.get_one::<String>("depot_lat").unwrap().parse()?;
    let depot_lon: f64 = matches.get_one::<String>("depot_lon").unwrap().parse()?;

    println!("🌍 Planet 83 OSM VRP Workflow Example");
    println!("=====================================");
    println!("📁 OSM data file: {}", osm_data_file);
    println!("🏢 Depot location: {:.4}, {:.4}", depot_lat, depot_lon);
    println!("👥 Number of customers: {}", num_customers);

    // Step 1: Load OSM data
    println!("\n📖 Step 1: Loading OSM data...");
    if !std::path::Path::new(osm_data_file).exists() {
        eprintln!("❌ OSM data file not found: {}", osm_data_file);
        eprintln!("💡 Run: cargo run --bin osm_converter -- --input planet_83.2932,17.7118_83.3388,17.7502.osm.pbf --roads-only");
        return Ok(());
    }

    let osm_file_handle = File::open(osm_data_file)?;
    let osm_reader = BufReader::new(osm_file_handle);
    let osm_data: OsmData = serde_json::from_reader(osm_reader)?;
    
    println!("✅ Loaded OSM data: {} nodes, {} ways", osm_data.nodes.len(), osm_data.ways.len());

    // Step 2: Find depot node and nearby customer locations
    println!("\n🎯 Step 2: Finding depot and customer locations...");
    
    // Find nearest OSM node to depot coordinates
    let depot_node = find_nearest_node(&osm_data, depot_lat, depot_lon);
    let (depot_node_id, depot_distance) = depot_node.ok_or("No depot node found")?;
    let depot_osm_node = osm_data.nodes.get(&depot_node_id).unwrap();
    
    println!("🏢 Depot mapped to OSM node {} ({:.2}m away)", depot_node_id, depot_distance);
    println!("   Coordinates: {:.6}, {:.6}", depot_osm_node.lat, depot_osm_node.lon);

    // Find customer locations (nearby OSM nodes)
    let customer_nodes = find_nearby_nodes(&osm_data, depot_osm_node.lat, depot_osm_node.lon, num_customers + 5)
        .into_iter()
        .filter(|(node_id, _)| *node_id != depot_node_id)  // Exclude depot
        .take(num_customers)
        .collect::<Vec<_>>();

    if customer_nodes.len() < num_customers {
        eprintln!("❌ Not enough nearby nodes found. Found: {}, needed: {}", 
                 customer_nodes.len(), num_customers);
        return Ok(());
    }

    println!("👥 Found {} customer locations:", customer_nodes.len());
    for (i, (node_id, distance)) in customer_nodes.iter().enumerate() {
        let node = osm_data.nodes.get(node_id).unwrap();
        println!("   Customer {}: Node {} at {:.6},{:.6} ({:.0}m from depot)", 
                i + 1, node_id, node.lat, node.lon, distance);
    }

    // Step 3: Create VRP instance
    println!("\n🚛 Step 3: Creating VRP instance...");
    
    let mut builder = VrpInstanceBuilder::new()
        .with_distance_method(DistanceMethod::Haversine)
        .with_average_speed(15.0); // 15 m/s ≈ 54 km/h

    // Add depot
    let depot_coord = Coordinate::new(depot_osm_node.lat, depot_osm_node.lon);
    builder = builder.add_depot(depot_node_id as usize, "Main Depot".to_string(), depot_coord);

    // Add customers with realistic demands
    use rand::{Rng, SeedableRng};
    use rand::rngs::StdRng;
    let mut rng = StdRng::seed_from_u64(42);
    
    for (i, (node_id, _)) in customer_nodes.iter().enumerate() {
        let node = osm_data.nodes.get(node_id).unwrap();
        let coord = Coordinate::new(node.lat, node.lon);
        let demand = rng.gen_range(5.0..25.0);
        let service_time = rng.gen_range(300.0..900.0); // 5-15 minutes
        
        builder = builder.add_customer(
            *node_id as usize,
            format!("Customer {}", i + 1),
            coord,
            demand,
            None, // No time windows for simplicity
            service_time,
        );
    }

    // Add vehicles
    let num_vehicles = (num_customers / 4).max(1);
    for i in 0..num_vehicles {
        builder = builder.add_vehicle_simple(i, 100.0, depot_node_id as usize);
    }

    let instance = builder.build()?;
    println!("✅ Created VRP instance with {} locations, {} vehicles", 
             instance.locations.len(), instance.vehicles.len());

    // Step 4: Solve VRP
    println!("\n🧮 Step 4: Solving VRP...");
    
    let multi_solver = MultiStartSolver::new()
        .add_solver(Box::new(GreedyNearestNeighbor::new()));
    
    let solution = multi_solver.solve(&instance)?;
    
    println!("✅ VRP solved successfully!");
    println!("   Routes: {}", solution.routes.len());
    println!("   Total distance: {:.2} m", solution.total_distance);
    println!("   Vehicles used: {}", solution.num_vehicles_used);

    // Step 5: Save solution for later use
    println!("\n💾 Step 5: Saving solution...");
    let solution_file = "planet_83_solution.json";
    save_solution_to_json(&solution, solution_file)?;
    println!("✅ Solution saved to: {}", solution_file);

    // Step 6: Export to GeoJSON
    println!("\n🌍 Step 6: Exporting to GeoJSON...");
    let geojson_file = "planet_83_routes.geojson";
    export_solution_to_geojson(&solution, &instance, geojson_file, Some(depot_coord))?;
    println!("✅ GeoJSON exported to: {}", geojson_file);

    println!("\n🎉 Complete workflow finished successfully!");
    println!("📊 Summary:");
    println!("   - Parsed OSM data: {} road nodes, {} ways", osm_data.nodes.len(), osm_data.ways.len());
    println!("   - Created VRP with {} locations using real coordinates", instance.locations.len());
    println!("   - Solved with {} routes, {:.2}km total distance", 
             solution.routes.len(), solution.total_distance / 1000.0);
    println!("   - Exported solution and GeoJSON for visualization");
    println!("\n📂 Files created:");
    println!("   - {}: VRP solution data", solution_file);
    println!("   - {}: GeoJSON for map visualization", geojson_file);
    println!("\n💡 Next steps:");
    println!("   - Open {} in QGIS, Leaflet, or any GIS application", geojson_file);
    println!("   - Use the solution data for further analysis or visualization");

    Ok(())
}

fn find_nearest_node(osm_data: &OsmData, target_lat: f64, target_lon: f64) -> Option<(i64, f64)> {
    let mut nearest_node = None;
    let mut min_distance = f64::MAX;

    for (&node_id, node) in &osm_data.nodes {
        let distance = haversine_distance(target_lat, target_lon, node.lat, node.lon);
        if distance < min_distance {
            min_distance = distance;
            nearest_node = Some((node_id, distance));
        }
    }

    nearest_node
}

fn find_nearby_nodes(osm_data: &OsmData, center_lat: f64, center_lon: f64, count: usize) -> Vec<(i64, f64)> {
    let mut nodes_with_distances: Vec<(i64, f64)> = osm_data.nodes
        .iter()
        .map(|(&node_id, node)| {
            let distance = haversine_distance(center_lat, center_lon, node.lat, node.lon);
            (node_id, distance)
        })
        .collect();

    // Sort by distance and take the closest nodes
    nodes_with_distances.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    nodes_with_distances.into_iter().take(count).collect()
}

fn haversine_distance(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
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

fn export_solution_to_geojson(
    solution: &Solution,
    instance: &vrp_solver::types::VrpInstance,
    file_path: &str,
    depot_coords: Option<Coordinate>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut features = Vec::new();

    // Process each route
    for (route_idx, route) in solution.routes.iter().enumerate() {
        let mut coordinates = Vec::new();

        // Add depot at the beginning if provided
        if let Some(depot) = depot_coords {
            coordinates.push(vec![depot.lon, depot.lat]); // GeoJSON uses [lon, lat]
        }

        // Get coordinates for each location in the route
        for &location_id in &route.locations {
            if let Some(location) = instance.get_location(location_id) {
                coordinates.push(vec![location.coordinate.lon, location.coordinate.lat]);

                // Add individual point feature for customer
                let mut properties = Map::new();
                properties.insert("type".to_string(), serde_json::Value::String("customer".to_string()));
                properties.insert("location_id".to_string(), serde_json::Value::Number(location_id.into()));
                properties.insert("name".to_string(), serde_json::Value::String(location.name.clone()));
                properties.insert("route_id".to_string(), serde_json::Value::Number(route_idx.into()));
                properties.insert("vehicle_id".to_string(), serde_json::Value::Number(route.vehicle_id.into()));
                properties.insert("demand".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(location.demand).unwrap()));

                let point_feature = Feature {
                    bbox: None,
                    geometry: Some(Geometry::new(Value::Point(vec![location.coordinate.lon, location.coordinate.lat]))),
                    id: None,
                    properties: Some(properties),
                    foreign_members: None,
                };

                features.push(point_feature);
            }
        }

        // Add depot at the end (complete the route)
        if let Some(depot) = depot_coords {
            coordinates.push(vec![depot.lon, depot.lat]);
        }

        // Create LineString feature for the route
        if coordinates.len() >= 2 {
            let mut properties = Map::new();
            properties.insert("type".to_string(), serde_json::Value::String("route".to_string()));
            properties.insert("route_id".to_string(), serde_json::Value::Number(route_idx.into()));
            properties.insert("vehicle_id".to_string(), serde_json::Value::Number(route.vehicle_id.into()));
            properties.insert("total_distance".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(route.total_distance).unwrap()));
            properties.insert("num_locations".to_string(), serde_json::Value::Number(route.locations.len().into()));

            let route_feature = Feature {
                bbox: None,
                geometry: Some(Geometry::new(Value::LineString(coordinates))),
                id: None,
                properties: Some(properties),
                foreign_members: None,
            };

            features.push(route_feature);
        }
    }

    // Add depot point if provided
    if let Some(depot) = depot_coords {
        let mut properties = Map::new();
        properties.insert("type".to_string(), serde_json::Value::String("depot".to_string()));
        properties.insert("name".to_string(), serde_json::Value::String("Main Depot".to_string()));

        let depot_feature = Feature {
            bbox: None,
            geometry: Some(Geometry::new(Value::Point(vec![depot.lon, depot.lat]))),
            id: None,
            properties: Some(properties),
            foreign_members: None,
        };

        features.push(depot_feature);
    }

    let feature_collection = FeatureCollection {
        bbox: None,
        features,
        foreign_members: None,
    };

    let geojson = GeoJson::FeatureCollection(feature_collection);
    let geojson_str = serde_json::to_string_pretty(&geojson)?;
    std::fs::write(file_path, geojson_str)?;

    Ok(())
}
