use clap::{Arg, Command};
use vrp_solver::osm_parser::OsmParser;
use serde_json;
use std::fs::File;
use std::io::BufReader;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("OSM Demo")
        .version("1.0")
        .author("VRP Solver")
        .about("Demonstrates using parsed OSM data for coordinate mapping")
        .arg(
            Arg::new("osm_json")
                .short('o')
                .long("osm-json")
                .value_name("FILE")
                .help("Path to the OSM JSON file")
                .default_value("southern-zone-latest.osm.pbf.json"),
        )
        .arg(
            Arg::new("lat")
                .long("lat")
                .value_name("LATITUDE")
                .help("Latitude to find nearest node for")
                .required(false),
        )
        .arg(
            Arg::new("lon")
                .long("lon")
                .value_name("LONGITUDE")
                .help("Longitude to find nearest node for")
                .required(false),
        )
        .arg(
            Arg::new("node_id")
                .long("node-id")
                .value_name("ID")
                .help("Node ID to get coordinates for")
                .required(false),
        )
        .arg(
            Arg::new("stats")
                .long("stats")
                .help("Show statistics about the OSM data")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    let osm_json_path = matches.get_one::<String>("osm_json").unwrap();
    
    // Check if file exists before trying to load it
    if !std::path::Path::new(osm_json_path).exists() {
        eprintln!("❌ OSM JSON file not found: {}", osm_json_path);
        eprintln!("💡 Run the OSM converter first: cargo run --bin osm_converter -- --input southern-zone-latest.osm.pbf --roads-only");
        return Ok(());
    }

    println!("📖 Loading OSM data from: {}", osm_json_path);
    
    // Load the OSM data from JSON
    let file = File::open(osm_json_path)?;
    let reader = BufReader::new(file);
    
    // Note: For very large files, you might want to use streaming JSON parsing
    // or load only a subset of the data. For now, we'll try to load the whole thing.
    println!("⚠️  Warning: Loading large JSON file into memory...");
    
    let osm_data: vrp_solver::osm_parser::OsmData = match serde_json::from_reader(reader) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("❌ Failed to parse OSM JSON: {}", e);
            eprintln!("💡 The JSON file might be too large to load into memory at once.");
            eprintln!("💡 Consider implementing streaming JSON parsing for production use.");
            return Err(Box::new(e));
        }
    };
    
    let mut parser = OsmParser::new();
    parser.data = osm_data;
    
    // Show statistics if requested
    if matches.get_flag("stats") {
        println!("\n📊 OSM Data Statistics:");
        println!("  - Total nodes: {}", parser.data.nodes.len());
        println!("  - Total ways: {}", parser.data.ways.len());
        
        // Show some highway types
        let mut highway_types = std::collections::HashMap::new();
        for way in parser.data.ways.values() {
            if let Some(highway) = way.tags.get("highway") {
                *highway_types.entry(highway.clone()).or_insert(0) += 1;
            }
        }
        
        println!("  - Highway types:");
        let mut sorted_highways: Vec<_> = highway_types.iter().collect();
        sorted_highways.sort_by(|a, b| b.1.cmp(a.1));
        for (highway_type, count) in sorted_highways.iter().take(10) {
            println!("    {} {}: {}", "   •", highway_type, count);
        }
    }
    
    // Find nearest node if coordinates provided
    if let (Some(lat_str), Some(lon_str)) = (matches.get_one::<String>("lat"), matches.get_one::<String>("lon")) {
        let lat: f64 = lat_str.parse()?;
        let lon: f64 = lon_str.parse()?;
        
        println!("\n🔍 Finding nearest node to ({}, {})...", lat, lon);
        
        if let Some((node_id, distance)) = parser.find_nearest_node(lat, lon) {
            println!("✅ Found nearest node:");
            println!("   - Node ID: {}", node_id);
            println!("   - Distance: {:.2} meters", distance);
            
            if let Some(coords) = parser.get_node_coordinates(node_id) {
                println!("   - Coordinates: ({}, {})", coords.0, coords.1);
            }
        } else {
            println!("❌ No nodes found in the dataset");
        }
    }
    
    // Get coordinates for specific node ID
    if let Some(node_id_str) = matches.get_one::<String>("node_id") {
        let node_id: i64 = node_id_str.parse()?;
        
        println!("\n📍 Looking up node ID: {}", node_id);
        
        if let Some(coords) = parser.get_node_coordinates(node_id) {
            println!("✅ Found coordinates: ({}, {})", coords.0, coords.1);
        } else {
            println!("❌ Node ID {} not found in the dataset", node_id);
        }
    }
    
    // If no specific action requested, show some sample data
    if !matches.get_flag("stats") 
        && matches.get_one::<String>("lat").is_none() 
        && matches.get_one::<String>("node_id").is_none() {
        
        println!("\n💡 Usage examples:");
        println!("   Show statistics:");
        println!("     cargo run --bin osm_demo -- --stats");
        println!("\n   Find nearest node to coordinates:");
        println!("     cargo run --bin osm_demo -- --lat 28.6139 --lon 77.2090");
        println!("\n   Get coordinates for a specific node ID:");
        println!("     cargo run --bin osm_demo -- --node-id 123456789");
        
        // Show a few sample nodes
        println!("\n🎲 Sample nodes from the dataset:");
        for (i, (node_id, node)) in parser.data.nodes.iter().take(5).enumerate() {
            println!("   {}. Node {}: ({}, {})", i + 1, node_id, node.lat, node.lon);
        }
    }
    
    Ok(())
}
