use clap::{Arg, Command};
use vrp_solver::osm_parser::OsmParser;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("OSM Converter")
        .version("1.0")
        .author("VRP Solver")
        .about("Converts OSM PBF files to JSON and GeoJSON formats")
        .arg(
            Arg::new("input")
                .short('i')
                .long("input")
                .value_name("FILE")
                .help("Input PBF file path")
                .required(true),
        )
        .arg(
            Arg::new("json")
                .short('j')
                .long("json")
                .value_name("FILE")
                .help("Output JSON file path")
                .required(false),
        )
        .arg(
            Arg::new("geojson")
                .short('g')
                .long("geojson")
                .value_name("FILE")
                .help("Output GeoJSON file path")
                .required(false),
        )
        .arg(
            Arg::new("roads-only")
                .short('r')
                .long("roads-only")
                .help("Filter to roads/highways only")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    let input_file = matches.get_one::<String>("input").unwrap();
    let json_file = matches.get_one::<String>("json");
    let geojson_file = matches.get_one::<String>("geojson");
    let roads_only = matches.get_flag("roads-only");

    println!("🚀 Starting OSM conversion process...");
    println!("📁 Input file: {}", input_file);

    // Initialize parser
    let mut parser = OsmParser::new();

    // Parse the PBF file
    println!("📖 Parsing PBF file...");
    parser.parse_pbf_file(input_file)?;

    // Filter to roads only if requested
    if roads_only {
        println!("🛣️ Filtering to roads only...");
        parser.filter_roads_only();
    }

    // Export to JSON if requested
    if let Some(json_path) = json_file {
        println!("💾 Exporting to JSON...");
        parser.export_to_json(json_path)?;
    }

    // Export to GeoJSON if requested
    if let Some(geojson_path) = geojson_file {
        println!("🌍 Exporting to GeoJSON...");
        parser.export_to_geojson(geojson_path)?;
    }

    // If neither output format was specified, export to default names
    if json_file.is_none() && geojson_file.is_none() {
        let base_name = input_file
            .strip_suffix(".osm.pbf")
            .unwrap_or(input_file)
            .strip_suffix(".pbf")
            .unwrap_or(input_file);

        let json_path = format!("{}.json", base_name);
        let geojson_path = format!("{}.geojson", base_name);

        println!("💾 Exporting to default JSON: {}", json_path);
        parser.export_to_json(&json_path)?;

        println!("🌍 Exporting to default GeoJSON: {}", geojson_path);
        parser.export_to_geojson(&geojson_path)?;
    }

    println!("✅ Conversion completed successfully!");
    
    // Print some statistics
    println!("\n📊 Statistics:");
    println!("  - Total nodes: {}", parser.data.nodes.len());
    println!("  - Total ways: {}", parser.data.ways.len());

    Ok(())
}
