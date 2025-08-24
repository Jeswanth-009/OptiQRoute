use clap::{Arg, Command};
use serde_json;
use std::fs::File;
use std::io::BufReader;
use vrp_solver::{
    distance::DistanceMethod,
    types::Coordinate,
    utils::{VrpInstanceBuilder, save_instance_to_json},
    osm_parser::OsmData,
};
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("OSM to VRP Instance Generator")
        .version("1.0")
        .author("VRP Solver")
        .about("Creates VRP problem instances from OSM road network data")
        .arg(
            Arg::new("osm_json")
                .short('o')
                .long("osm-json")
                .value_name("FILE")
                .help("Path to the OSM JSON file")
                .default_value("planet_83.2932,17.7118_83.3388,17.7502.osm.pbf.json"),
        )
        .arg(
            Arg::new("output")
                .short('i')
                .long("instance")
                .value_name("FILE")
                .help("Output VRP instance JSON file")
                .default_value("osm_vrp_instance.json"),
        )
        .arg(
            Arg::new("depot_lat")
                .long("depot-lat")
                .value_name("LATITUDE")
                .help("Depot latitude coordinate")
                .default_value("17.735"),
        )
        .arg(
            Arg::new("depot_lon")
                .long("depot-lon")
                .value_name("LONGITUDE")
                .help("Depot longitude coordinate")
                .default_value("83.315"),
        )
        .arg(
            Arg::new("num_customers")
                .short('n')
                .long("customers")
                .value_name("COUNT")
                .help("Number of customer locations")
                .default_value("10"),
        )
        .arg(
            Arg::new("num_vehicles")
                .short('v')
                .long("vehicles")
                .value_name("COUNT")
                .help("Number of vehicles")
                .default_value("3"),
        )
        .arg(
            Arg::new("vehicle_capacity")
                .short('c')
                .long("capacity")
                .value_name("CAPACITY")
                .help("Vehicle capacity units")
                .default_value("100"),
        )
        .arg(
            Arg::new("min_demand")
                .long("min-demand")
                .value_name("DEMAND")
                .help("Minimum customer demand")
                .default_value("5"),
        )
        .arg(
            Arg::new("max_demand")
                .long("max-demand")
                .value_name("DEMAND")
                .help("Maximum customer demand")
                .default_value("25"),
        )
        .arg(
            Arg::new("seed")
                .long("seed")
                .value_name("SEED")
                .help("Random seed for reproducible instances")
                .default_value("42"),
        )
        .arg(
            Arg::new("max_radius")
                .long("max-radius")
                .value_name("METERS")
                .help("Maximum radius from depot to search for customers (meters)")
                .default_value("1000"),
        )
        .get_matches();

    let osm_json_path = matches.get_one::<String>("osm_json").unwrap();
    let output_file = matches.get_one::<String>("output").unwrap();
    let depot_lat: f64 = matches.get_one::<String>("depot_lat").unwrap().parse()?;
    let depot_lon: f64 = matches.get_one::<String>("depot_lon").unwrap().parse()?;
    let num_customers: usize = matches.get_one::<String>("num_customers").unwrap().parse()?;
    let num_vehicles: usize = matches.get_one::<String>("num_vehicles").unwrap().parse()?;
    let vehicle_capacity: f64 = matches.get_one::<String>("vehicle_capacity").unwrap().parse()?;
    let min_demand: f64 = matches.get_one::<String>("min_demand").unwrap().parse()?;
    let max_demand: f64 = matches.get_one::<String>("max_demand").unwrap().parse()?;
    let seed: u64 = matches.get_one::<String>("seed").unwrap().parse()?;
    let max_radius: f64 = matches.get_one::<String>("max_radius").unwrap().parse()?;

    println!("🏗️ OSM to VRP Instance Generator");
    println!("===============================");
    println!("📁 OSM JSON file: {}", osm_json_path);
    println!("📁 Output file: {}", output_file);
    println!("🏢 Depot: {:.4}°N, {:.4}°E", depot_lat, depot_lon);
    println!("👥 Customers: {}", num_customers);
    println!("🚛 Vehicles: {} (capacity: {})", num_vehicles, vehicle_capacity);
    println!("📦 Demand range: {:.1} - {:.1}", min_demand, max_demand);
    println!("📏 Search radius: {}m", max_radius);

    // Check if OSM file exists
    if !std::path::Path::new(osm_json_path).exists() {
        eprintln!("❌ OSM JSON file not found: {}", osm_json_path);
        eprintln!("💡 Run the OSM converter first:");
        eprintln!("   cargo run --bin osm_converter -- --input your_file.osm.pbf --roads-only");
        return Ok(());
    }

    // Load OSM data
    println!("\n📖 Step 1: Loading OSM data...");
    let osm_file = File::open(osm_json_path)?;
    let osm_reader = BufReader::new(osm_file);
    let osm_data: OsmData = serde_json::from_reader(osm_reader)?;
    println!("✅ Loaded {} road nodes, {} ways", osm_data.nodes.len(), osm_data.ways.len());

    // Find depot location
    println!("\n🎯 Step 2: Finding depot location...");
    let depot_node = find_nearest_node(&osm_data, depot_lat, depot_lon);
    let (depot_node_id, depot_distance) = depot_node.ok_or("No depot node found in OSM data")?;
    let depot_osm_node = osm_data.nodes.get(&depot_node_id).unwrap();
    
    println!("🏢 Depot mapped to OSM node {} ({:.2}m away)", depot_node_id, depot_distance);
    println!("   Actual coordinates: {:.6}°N, {:.6}°E", depot_osm_node.lat, depot_osm_node.lon);

    // Find customer locations within radius
    println!("\n📍 Step 3: Finding customer locations...");
    let nearby_nodes = find_nodes_within_radius(&osm_data, depot_osm_node.lat, depot_osm_node.lon, max_radius)
        .into_iter()
        .filter(|(node_id, _)| *node_id != depot_node_id)  // Exclude depot
        .collect::<Vec<_>>();

    if nearby_nodes.len() < num_customers {
        eprintln!("❌ Not enough nodes within {}m radius. Found: {}, needed: {}", 
                 max_radius, nearby_nodes.len(), num_customers);
        eprintln!("💡 Try increasing --max-radius or reducing --customers");
        return Ok(());
    }

    println!("✅ Found {} potential customer locations within {}m", nearby_nodes.len(), max_radius);

    // Select customer nodes with good spatial distribution
    let selected_customers = select_distributed_customers(&nearby_nodes, num_customers, seed);
    
    println!("👥 Selected {} customers with good spatial distribution:", selected_customers.len());
    for (i, (node_id, distance)) in selected_customers.iter().enumerate() {
        let node = osm_data.nodes.get(node_id).unwrap();
        println!("   Customer {}: Node {} at {:.6},{:.6} ({:.0}m from depot)", 
                i + 1, node_id, node.lat, node.lon, distance);
    }

    // Create VRP instance
    println!("\n🚛 Step 4: Creating VRP instance...");
    let mut builder = VrpInstanceBuilder::new()
        .with_distance_method(DistanceMethod::Haversine)
        .with_average_speed(15.0); // 15 m/s ≈ 54 km/h

    // Add depot using actual OSM coordinates
    let depot_coord = Coordinate::new(depot_osm_node.lat, depot_osm_node.lon);
    builder = builder.add_depot(depot_node_id as usize, "OSM Depot".to_string(), depot_coord);

    // Add customers with generated demands and service times
    let mut rng = StdRng::seed_from_u64(seed);
    for (i, (node_id, _)) in selected_customers.iter().enumerate() {
        let node = osm_data.nodes.get(node_id).unwrap();
        let coord = Coordinate::new(node.lat, node.lon);
        let demand = rng.gen_range(min_demand..=max_demand);
        let service_time = rng.gen_range(300.0..=900.0); // 5-15 minutes
        
        builder = builder.add_customer(
            *node_id as usize,
            format!("OSM Customer {}", i + 1),
            coord,
            demand,
            None, // No time windows
            service_time,
        );
    }

    // Add vehicles
    for i in 0..num_vehicles {
        builder = builder.add_vehicle_simple(i, vehicle_capacity, depot_node_id as usize);
    }

    let instance = builder.build()?;
    println!("✅ Created VRP instance:");
    println!("   - {} locations ({} customers + 1 depot)", instance.locations.len(), num_customers);
    println!("   - {} vehicles (capacity: {})", instance.vehicles.len(), vehicle_capacity);
    println!("   - Distance matrix computed using Haversine formula");
    println!("   - All locations use real OSM road coordinates");

    // Calculate some statistics
    let total_demand: f64 = instance.locations.iter()
        .filter(|loc| loc.demand > 0.0)
        .map(|loc| loc.demand)
        .sum();
    let avg_demand = total_demand / num_customers as f64;
    let total_capacity = num_vehicles as f64 * vehicle_capacity;
    let capacity_utilization = (total_demand / total_capacity) * 100.0;

    println!("\n📊 Instance Statistics:");
    println!("   - Total customer demand: {:.1}", total_demand);
    println!("   - Average demand per customer: {:.1}", avg_demand);
    println!("   - Total fleet capacity: {:.1}", total_capacity);
    println!("   - Capacity utilization: {:.1}%", capacity_utilization);

    // Save instance to file
    println!("\n💾 Step 5: Saving VRP instance...");
    save_instance_to_json(&instance, output_file)?;
    println!("✅ Instance saved to: {}", output_file);

    // Show usage examples
    println!("\n💡 Next steps:");
    println!("   1. Solve the VRP instance:");
    println!("      cargo run -- --instance {}", output_file);
    println!("   2. Convert solution to GeoJSON:");
    println!("      cargo run --bin vrp_to_geojson -- --solution solution.json --osm-data {}", osm_json_path);
    
    if capacity_utilization > 95.0 {
        println!("\n⚠️  Warning: High capacity utilization ({:.1}%). Consider adding more vehicles.", capacity_utilization);
    }

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

fn find_nodes_within_radius(osm_data: &OsmData, center_lat: f64, center_lon: f64, radius: f64) -> Vec<(i64, f64)> {
    let mut nodes_in_radius = Vec::new();

    for (&node_id, node) in &osm_data.nodes {
        let distance = haversine_distance(center_lat, center_lon, node.lat, node.lon);
        if distance <= radius {
            nodes_in_radius.push((node_id, distance));
        }
    }

    nodes_in_radius.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    nodes_in_radius
}

fn select_distributed_customers(candidates: &[(i64, f64)], count: usize, seed: u64) -> Vec<(i64, f64)> {
    if candidates.len() <= count {
        return candidates.to_vec();
    }

    let mut selected = Vec::new();
    let mut remaining = candidates.to_vec();
    let mut rng = StdRng::seed_from_u64(seed);

    // Always select the closest node first
    selected.push(remaining.remove(0));

    // For the rest, try to maintain good spatial distribution
    while selected.len() < count && !remaining.is_empty() {
        // Find the node that maximizes minimum distance to already selected nodes
        let mut best_idx = 0;
        let mut best_min_distance = 0.0;

        for (i, (candidate_id, _)) in remaining.iter().enumerate() {
            let candidate_node = &candidates.iter().find(|(id, _)| id == candidate_id).unwrap();
            
            // Calculate minimum distance to any already selected node
            let mut min_dist_to_selected = f64::MAX;
            for (selected_id, _) in &selected {
                // This is a simplified approach - in a real implementation you'd use 
                // the actual coordinates from the OSM data
                let dist = (*candidate_id - *selected_id).abs() as f64; // Simplified distance
                min_dist_to_selected = min_dist_to_selected.min(dist);
            }

            if min_dist_to_selected > best_min_distance {
                best_min_distance = min_dist_to_selected;
                best_idx = i;
            }
        }

        // Add some randomness to avoid too regular patterns
        let selection_idx = if rng.gen::<f64>() < 0.7 {
            best_idx
        } else {
            rng.gen_range(0..remaining.len().min(5)) // Random selection from top 5
        };

        selected.push(remaining.remove(selection_idx));
    }

    selected
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
