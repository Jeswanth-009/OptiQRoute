# VRP Solver - Vehicle Routing Problem Solver in Rust

A high-performance Vehicle Routing Problem (VRP) solver implemented in Rust with parallel processing capabilities using Rayon.

## 🚀 Features

- **Multiple Algorithms**: 
  - Greedy Nearest Neighbor (with nearest/farthest start options)
  - Clarke-Wright Savings Algorithm
  - Multi-start solver for best results
  
- **Parallel Processing**: Utilizes Rayon for fast distance matrix computation and parallel algorithm execution

- **Rich Constraints Support**:
  - Vehicle capacity limits
  - Time windows for deliveries
  - Maximum route distance/duration limits
  - Service times at locations

- **Geographic Distance Calculations**:
  - Haversine distance (great circle distance)
  - Manhattan distance approximation
  - Euclidean distance approximation

- **Comprehensive Validation**: Route validation with detailed reporting

- **Data I/O**: JSON serialization/deserialization and CSV export capabilities

## 🏗️ Architecture

The project is structured into several key modules:

- **`types.rs`**: Core data structures (Location, Vehicle, Route, Solution, etc.)
- **`distance.rs`**: Distance calculations with parallel processing
- **`solver.rs`**: VRP solving algorithms
- **`validate.rs`**: Route validation and constraint checking
- **`utils.rs`**: Utility functions and helper tools

## 🚀 Quick Start

### Basic Usage

```rust
use vrp_solver::*;

// Create a VRP instance
let instance = VrpInstanceBuilder::new()
    .add_depot(0, "Main Depot".to_string(), Coordinate::new(52.5200, 13.4050))
    .add_customer(1, "Customer A".to_string(), Coordinate::new(52.5100, 13.3900), 15.0, None, 300.0)
    .add_vehicle_simple(0, 50.0, 0)
    .with_distance_method(DistanceMethod::Haversine)
    .build()?;

// Solve using the multi-start solver
let solver = MultiStartSolver::new().with_default_solvers();
let solution = solver.solve(&instance)?;

// Validate the solution
let is_valid = validate_solution(&instance, &solution)?;
println!("Solution is valid: {}", is_valid);
```

### Running Examples

```bash
# Run the demo
cargo run

# Run tests
cargo test

# Run with optimizations
cargo run --release
```

## 📊 Performance

The solver uses parallel processing for:
- Distance matrix computation using Rayon parallel iterators
- Clarke-Wright savings calculation
- Multi-algorithm execution for optimal results

Example performance on a 10-customer problem:
- Greedy Nearest Neighbor: ~0.02ms
- Clarke-Wright Savings: ~1.23ms
- Multi-Start (all algorithms): ~0.23ms

## 🛠️ API Reference

### Core Types

- **`VrpInstance`**: Problem definition with locations, vehicles, and constraints
- **`Solution`**: Complete solution with routes and metrics
- **`Route`**: Single vehicle route with locations and statistics
- **`Location`**: Geographic location with demand and time windows
- **`Vehicle`**: Vehicle definition with capacity and limits

### Solvers

- **`GreedyNearestNeighbor`**: Fast greedy algorithm
- **`ClarkeWrightSavings`**: Classic savings algorithm
- **`MultiStartSolver`**: Runs multiple algorithms and returns best result

### Distance Methods

- **`Haversine`**: Great circle distance (most accurate for lat/lon)
- **`Manhattan`**: L1 distance approximation
- **`Euclidean`**: L2 distance approximation

## 🔧 Configuration

### Vehicle Constraints

```rust
Vehicle::new(
    0,              // Vehicle ID
    100.0,          // Capacity
    Some(50000.0),  // Max distance (meters)
    Some(14400.0),  // Max duration (seconds)
    0               // Depot ID
)
```

### Time Windows

```rust
Location::new(
    1,
    "Customer".to_string(),
    coordinate,
    demand,
    Some(TimeWindow::new(0.0, 3600.0)), // 1-hour window
    300.0 // 5-minute service time
)
```

## 📁 Data Formats

### JSON Input/Output

The solver supports JSON serialization for both instances and solutions:

```rust
// Save instance and solution
save_instance_to_json(&instance, "problem.json")?;
save_solution_to_json(&solution, "solution.json")?;

// Load instance and solution
let instance = load_instance_from_json("problem.json")?;
let solution = load_solution_from_json("solution.json")?;
```

### CSV Export

Export solutions to CSV for analysis:

```rust
export_solution_to_csv(&solution, &instance, "routes.csv")?;
```

## 🧪 Testing

The project includes comprehensive tests covering:

- Distance calculations (haversine accuracy)
- Algorithm correctness
- Validation logic
- Data serialization
- Constraint handling

Run tests with:
```bash
cargo test
```

## 📈 Validation and Reporting

The solver provides detailed validation with constraint checking:

```rust
let validator = RouteValidator::new()
    .with_capacity_check(true)
    .with_time_window_check(true)
    .with_distance_limit_check(true);

let results = validator.validate_solution(&instance, &solution)?;
let report = get_validation_report(&instance, &solution)?;
```

## 🔮 Future Enhancements

- Genetic Algorithm solver
- Simulated Annealing
- Tabu Search
- Real-time traffic integration
- Multi-depot support
- Dynamic VRP capabilities

## 📜 License

This project is open source. Feel free to use, modify, and distribute according to your needs.

## 🤝 Contributing

Contributions are welcome! Areas for improvement:
- Additional solving algorithms
- Performance optimizations
- More constraint types
- Better heuristics
- Documentation improvements

---

Built with ❤️ using Rust and Rayon for high-performance parallel computing.
