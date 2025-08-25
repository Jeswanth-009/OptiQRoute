use serde_json::{json, Value};
use std::fs;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Parse command-line args
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <input.geojson> <output.geojson>", args[0]);
        std::process::exit(1);
    }
    let input_file = &args[1];
    let output_file = &args[2];

    // 2. Load input GeoJSON
    let input = fs::read_to_string(input_file)?;
    let mut geojson: Value = serde_json::from_str(&input)?;

    // 3. Iterate over features
    if let Some(features) = geojson["features"].as_array_mut() {
        for feature in features {
            let geom_type = feature["geometry"]["type"].as_str().unwrap_or("");
            let ftype = feature["properties"]["type"].as_str().unwrap_or("");

            // Only process route LineStrings
            if geom_type == "LineString" && ftype == "route" {
                let coords = feature["geometry"]["coordinates"]
                    .as_array()
                    .expect("No coordinates in LineString");

                // Build OSRM query string (lon,lat pairs)
                let coord_strs: Vec<String> = coords.iter()
                    .map(|c| format!("{},{}", c[0].as_f64().unwrap(), c[1].as_f64().unwrap()))
                    .collect();

                let query_coords = coord_strs.join(";");

                // Call OSRM API
                let url = format!(
                    "http://router.project-osrm.org/route/v1/driving/{}?overview=full&geometries=geojson",
                    query_coords
                );

                let resp: Value = reqwest::get(&url).await?.json().await?;

                if let Some(snapped_geom) = resp["routes"][0]["geometry"].as_object() {
                    // Replace feature geometry with snapped geometry
                    feature["geometry"] = json!(snapped_geom);
                }
            }
        }
    }

    // 4. Save updated GeoJSON
    fs::write(output_file, serde_json::to_string_pretty(&geojson)?)?;
    println!("✅ Snapped routes saved to {}", output_file);

    Ok(())
}
