//! Example usage of the VRP solver library
//! 
//! This demonstrates how to create VRP instances, solve them, and validate solutions.

use vrp_solver::*;
use std::time::Instant;

fn main() -> VrpResult<()> {
    println!("🚛 Vehicle Routing Problem (VRP) Solver Demo");
    println!("==============================================\n");

    // Example 1: Simple test instance
    println!("📍 Example 1: Creating and solving a test instance with 10 customers");
    let depot_coord = Coordinate::new(52.5200, 13.4050); // Berlin coordinates
    let instance = create_test_instance(10, depot_coord);
    
    println!("Instance created:");
    println!("  - Locations: {}", instance.num_locations());
    println!("  - Vehicles: {}", instance.num_vehicles());
    println!("  - Depot: {}", depot_coord.lat);
    
    // Solve using multiple algorithms
    let solver = MultiStartSolver::new().with_default_solvers();
    
    let start_time = Instant::now();
    let solution = solver.solve(&instance)?;
    let solve_time = start_time.elapsed();
    
    println!("\n✅ Solution found in {:.2}ms", solve_time.as_secs_f64() * 1000.0);
    println!("{}", format_solution_summary(&solution));
    
    // Validate the solution
    let is_valid = validate_solution(&instance, &solution)?;
    println!("\n🔍 Solution validation: {}", if is_valid { "✅ VALID" } else { "❌ INVALID" });
    
    // Example 2: Custom instance with time windows
    println!("\n📍 Example 2: Custom instance with time windows");
    let custom_instance = create_custom_instance()?;
    
    let custom_solver = GreedyNearestNeighbor::new();
    let custom_solution = custom_solver.solve(&custom_instance)?;
    
    println!("Custom solution:");
    println!("{}", format_solution_summary(&custom_solution));
    
    // Get detailed validation report
    println!("\n📋 Detailed Validation Report:");
    let report = get_validation_report(&custom_instance, &custom_solution)?;
    println!("{}", report);
    
    // Example 3: Comparison of different algorithms
    println!("\n📊 Algorithm Comparison:");
    compare_algorithms(&instance)?;
    
    // Example 4: Save/load functionality
    println!("\n💾 Save/Load Example:");
    demonstrate_save_load(&instance, &solution)?;
    
    Ok(())
}

/// Create a custom VRP instance with time windows and various constraints
fn create_custom_instance() -> VrpResult<VrpInstance> {
    VrpInstanceBuilder::new()
        // Add depot
        .add_depot(0, "Main Depot".to_string(), Coordinate::new(52.5200, 13.4050))
        
        // Add customers with time windows
        .add_customer(
            1, 
            "Customer A".to_string(), 
            Coordinate::new(52.5100, 13.3900), 
            15.0,
            Some(TimeWindow::new(0.0, 3600.0)), // 1 hour window
            300.0 // 5 minutes service time
        )
        .add_customer(
            2,
            "Customer B".to_string(),
            Coordinate::new(52.5300, 13.4200),
            20.0,
            Some(TimeWindow::new(1800.0, 5400.0)), // 30min - 1.5hr window
            600.0 // 10 minutes service time
        )
        .add_customer(
            3,
            "Customer C".to_string(),
            Coordinate::new(52.5000, 13.4100),
            12.0,
            Some(TimeWindow::new(3600.0, 7200.0)), // 1hr - 2hr window
            450.0 // 7.5 minutes service time
        )
        .add_customer(
            4,
            "Customer D".to_string(),
            Coordinate::new(52.5400, 13.3800),
            18.0,
            None, // No time window
            240.0 // 4 minutes service time
        )
        
        // Add vehicles with different capacities and constraints
        .add_vehicle(Vehicle::new(
            0,
            50.0,                    // capacity
            Some(50000.0),          // max distance (50km)
            Some(14400.0),          // max duration (4 hours)
            0                       // depot id
        ))
        .add_vehicle(Vehicle::new(
            1,
            35.0,                   // smaller capacity
            Some(30000.0),          // shorter max distance (30km)
            Some(10800.0),          // shorter max duration (3 hours)
            0                       // depot id
        ))
        
        .with_distance_method(DistanceMethod::Haversine)
        .with_average_speed(12.0) // 12 m/s ≈ 43 km/h (city traffic)
        .build()
}

/// Compare different solving algorithms on the same instance
fn compare_algorithms(instance: &VrpInstance) -> VrpResult<()> {
    let algorithms: Vec<(Box<dyn VrpSolver + Sync>, &str)> = vec![
        (Box::new(GreedyNearestNeighbor::new()), "Greedy (Nearest Start)"),
        (Box::new(GreedyNearestNeighbor::new().with_farthest_start(true)), "Greedy (Farthest Start)"),
        (Box::new(ClarkeWrightSavings::new()), "Clarke-Wright Savings"),
        (Box::new(MultiStartSolver::new().with_default_solvers()), "Multi-Start"),
    ];
    
    println!("Algorithm Performance Comparison:");
    println!("{:<25} | {:>10} | {:>10} | {:>8} | {:>10}", 
        "Algorithm", "Distance", "Duration", "Vehicles", "Time (ms)");
    println!("{}", "-".repeat(75));
    
    for (solver, name) in algorithms {
        let start_time = Instant::now();
        match solver.solve(instance) {
            Ok(solution) => {
                let solve_time = start_time.elapsed();
                let metrics = SolutionMetrics::from_solution(&solution);
                
                println!("{:<25} | {:>10.1} | {:>10.1} | {:>8} | {:>10.2}",
                    name,
                    metrics.total_distance,
                    metrics.total_duration,
                    metrics.num_vehicles,
                    solve_time.as_secs_f64() * 1000.0
                );
            }
            Err(_) => {
                let solve_time = start_time.elapsed();
                println!("{:<25} | {:>10} | {:>10} | {:>8} | {:>10.2}",
                    name, "FAILED", "FAILED", "FAILED", solve_time.as_secs_f64() * 1000.0);
            }
        }
    }
    
    Ok(())
}

/// Demonstrate save and load functionality
fn demonstrate_save_load(instance: &VrpInstance, solution: &Solution) -> VrpResult<()> {
    // Save instance to JSON
    save_instance_to_json(instance, "example_instance.json")?;
    println!("✅ Instance saved to example_instance.json");
    
    // Save solution to JSON
    save_solution_to_json(solution, "example_solution.json")?;
    println!("✅ Solution saved to example_solution.json");
    
    // Export solution to CSV
    export_solution_to_csv(solution, instance, "example_solution.csv")?;
    println!("✅ Solution exported to example_solution.csv");
    
    // Load instance back
    let loaded_instance = load_instance_from_json("example_instance.json")?;
    println!("✅ Instance loaded from JSON - {} locations, {} vehicles", 
        loaded_instance.num_locations(), 
        loaded_instance.num_vehicles()
    );
    
    // Load solution back
    let loaded_solution = load_solution_from_json("example_solution.json")?;
    println!("✅ Solution loaded from JSON - {} routes, {:.1}m total distance", 
        loaded_solution.routes.len(), 
        loaded_solution.total_distance
    );
    
    Ok(())
}

/// Demonstrate advanced usage patterns
fn demonstrate_advanced_usage() -> VrpResult<()> {
    println!("\n🔧 Advanced Usage Examples:");
    
    // Example: Custom validation settings
    let instance = create_test_instance(5, Coordinate::new(0.0, 0.0));
    let solver = GreedyNearestNeighbor::new();
    let solution = solver.solve(&instance)?;
    
    let custom_validator = RouteValidator::new()
        .with_capacity_check(true)
        .with_time_window_check(false)  // Ignore time windows
        .with_distance_limit_check(true)
        .with_duration_limit_check(false); // Ignore duration limits
    
    let validation_results = custom_validator.validate_solution(&instance, &solution)?;
    
    println!("Custom validation results:");
    for (i, result) in validation_results.iter().enumerate() {
        println!("  Route {}: {} violations, {:.1}% capacity utilization", 
            i + 1, 
            result.violations.len(),
            result.capacity_utilization * 100.0
        );
    }
    
    // Example: Solution metrics analysis
    let metrics = SolutionMetrics::from_solution(&solution);
    println!("\nSolution Metrics Analysis:");
    println!("  Total Distance: {:.1}m", metrics.total_distance);
    println!("  Average Route Distance: {:.1}m", metrics.average_route_distance);
    println!("  Distance Std Deviation: {:.1}m", metrics.distance_std_dev);
    println!("  Min/Max Route Distance: {:.1}m / {:.1}m", 
        metrics.min_route_distance, metrics.max_route_distance);
    
    Ok(())
}

/// Example of parallel distance matrix calculation
fn demonstrate_parallel_processing() -> VrpResult<()> {
    println!("\n⚡ Parallel Processing Demo:");
    
    let large_instance = create_test_instance(50, Coordinate::new(52.5200, 13.4050));
    
    // Time the distance matrix calculation
    let start_time = Instant::now();
    let _distances = &large_instance.distance_matrix;
    let calc_time = start_time.elapsed();
    
    println!("Distance matrix calculation for {} locations:", large_instance.num_locations());
    println!("  Time: {:.2}ms (using parallel iterators)", calc_time.as_secs_f64() * 1000.0);
    
    // Demonstrate parallel savings calculation
    if !large_instance.locations.is_empty() {
        let depot_id = large_instance.locations[0].id;
        let start_time = Instant::now();
        let savings = calculate_savings(&large_instance, depot_id);
        let savings_time = start_time.elapsed();
        
        println!("Clarke-Wright savings calculation:");
        println!("  {} savings computed in {:.2}ms", 
            savings.len(), 
            savings_time.as_secs_f64() * 1000.0
        );
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example_runs() {
        // Test that the main examples run without panicking
        let depot_coord = Coordinate::new(52.5200, 13.4050);
        let instance = create_test_instance(5, depot_coord);
        
        let solver = GreedyNearestNeighbor::new();
        let result = solver.solve(&instance);
        assert!(result.is_ok());
        
        let solution = result.unwrap();
        assert!(solution.is_valid());
    }
    
    #[test] 
    fn test_custom_instance_creation() {
        let result = create_custom_instance();
        assert!(result.is_ok());
        
        let instance = result.unwrap();
        assert_eq!(instance.num_locations(), 5); // 1 depot + 4 customers
        assert_eq!(instance.num_vehicles(), 2);
    }
}
