//! CLI interface for VRP solver
//! 
//! This binary provides a command-line interface for the VRP solver
//! that can be called from Python or other external programs.

use std::fs;
use std::env;
use serde::{Deserialize, Serialize};
use vrp_solver::*;

#[derive(Deserialize)]
struct CliInput {
    instance: VrpInstance,
    algorithm: String,
    settings: Option<CliSettings>,
}

#[derive(Deserialize)]
struct CliSettings {
    distance_method: Option<String>,
    parallel: Option<bool>,
}

#[derive(Serialize)]
struct CliOutput {
    routes: Vec<RouteOutput>,
    total_distance: f64,
    total_duration: f64,
    num_vehicles_used: usize,
    algorithm: String,
    solve_time_ms: f64,
    success: bool,
    error: Option<String>,
}

#[derive(Serialize)]
struct RouteOutput {
    vehicle_id: usize,
    locations: Vec<usize>,
    total_distance: f64,
    total_duration: f64,
    total_demand: f64,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 5 || args[1] != "--input" || args[3] != "--output" {
        eprintln!("Usage: {} --input <input.json> --output <output.json>", args[0]);
        std::process::exit(1);
    }
    
    let input_file = &args[2];
    let output_file = &args[4];
    
    // Read input file
    let input_data = fs::read_to_string(input_file)
        .map_err(|e| format!("Failed to read input file: {}", e))?;
    
    let cli_input: CliInput = serde_json::from_str(&input_data)
        .map_err(|e| format!("Failed to parse input JSON: {}", e))?;
    
    // Create VRP instance and calculate distance matrix
    let mut instance = cli_input.instance;
    
    // Reinitialize distance matrix with correct size
    let n = instance.locations.len();
    instance.distance_matrix = vec![vec![0.0; n]; n];
    
    // Determine distance calculation method
    let distance_method = cli_input.settings
        .as_ref()
        .and_then(|s| s.distance_method.as_ref())
        .map(|s| s.as_str())
        .unwrap_or("haversine");
    
    let method = match distance_method {
        "haversine" => DistanceMethod::Haversine,
        "euclidean" => DistanceMethod::Euclidean,
        "manhattan" => DistanceMethod::Manhattan,
        _ => DistanceMethod::Haversine,
    };
    
    // Calculate distance matrix
    calculate_distance_matrix(&mut instance, method);
    
    // Determine solver algorithm
    let solver: Box<dyn VrpSolver + Sync> = match cli_input.algorithm.as_str() {
        "greedy" => Box::new(GreedyNearestNeighbor::new()),
        "greedy_farthest" => Box::new(GreedyNearestNeighbor::new().with_farthest_start(true)),
        "clarke_wright" => {
            let parallel = cli_input.settings
                .as_ref()
                .and_then(|s| s.parallel)
                .unwrap_or(true);
            Box::new(ClarkeWrightSavings::new().with_parallel(parallel))
        },
        "multi_start" => Box::new(MultiStartSolver::new().with_default_solvers()),
        _ => Box::new(MultiStartSolver::new().with_default_solvers()),
    };
    
    // Solve VRP
    let start_time = std::time::Instant::now();
    let result = solver.solve(&instance);
    let solve_time = start_time.elapsed().as_millis() as f64;
    
    // Prepare output
    let output = match result {
        Ok(solution) => {
            let routes: Vec<RouteOutput> = solution.routes.into_iter().map(|route| {
                RouteOutput {
                    vehicle_id: route.vehicle_id,
                    locations: route.locations,
                    total_distance: route.total_distance,
                    total_duration: route.total_duration,
                    total_demand: route.total_demand,
                }
            }).collect();

            CliOutput {
                routes,
                total_distance: solution.total_distance,
                total_duration: solution.total_duration,
                num_vehicles_used: solution.num_vehicles_used,
                algorithm: cli_input.algorithm.clone(),
                solve_time_ms: solve_time,
                success: true,
                error: None,
            }
        },
        Err(e) => CliOutput {
            routes: Vec::new(),
            total_distance: 0.0,
            total_duration: 0.0,
            num_vehicles_used: 0,
            algorithm: cli_input.algorithm.clone(),
            solve_time_ms: solve_time,
            success: false,
            error: Some(e.to_string()),
        },
    };
    
    // Write output file
    let output_json = serde_json::to_string_pretty(&output)
        .map_err(|e| format!("Failed to serialize output: {}", e))?;
    
    fs::write(output_file, output_json)
        .map_err(|e| format!("Failed to write output file: {}", e))?;
    
    if !output.success {
        eprintln!("VRP solving failed: {:?}", output.error);
        std::process::exit(1);
    }
    
    println!("VRP solved successfully in {:.2}ms using {}", solve_time, solver.name());
    
    Ok(())
}
