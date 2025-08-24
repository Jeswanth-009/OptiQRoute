# VRP Solver - Vehicle Routing Problem Solver in Rust

A comprehensive Vehicle Routing Problem (VRP) solver implemented in Rust with parallel processing capabilities, real-world OSM data integration, and advanced geographic coordinate mapping.

## 🚀 Features

- **Multiple Algorithms**: 
  - Greedy Nearest Neighbor (with nearest/farthest start options)
  - Clarke-Wright Savings Algorithm
  - Multi-start solver for best results
  
- **Parallel Processing**: Utilizes Rayon for fast distance matrix computation and parallel algorithm execution

- **OSM Data Integration** 🗺️:
  - Parse OSM/PBF files with real-world road network data
  - Convert to structured JSON format for coordinate mapping
  - Find nearest road intersections for delivery locations
  - Filter to roads-only for efficient routing
  - Support for massive datasets (74M+ nodes processed)

- **Rich Constraints Support**:
  - Vehicle capacity limits
  - Time windows for deliveries
  - Maximum route distance/duration limits
  - Service times at locations

- **Geographic Distance Calculations**:
  - Haversine distance (great circle distance)
  - Manhattan distance approximation
  - Euclidean distance approximation
  - Real-world coordinate mapping from OSM data

- **Comprehensive Validation**: Route validation with detailed reporting

- **Data I/O**: JSON/CSV export, OSM data parsing, coordinate mapping

## 🏗️ Architecture

The project is structured into several key modules:

- **`types.rs`**: Core data structures (Location, Vehicle, Route, Solution, etc.)
- **`distance.rs`**: Distance calculations with parallel processing
- **`solver.rs`**: VRP solving algorithms
- **`validate.rs`**: Route validation and constraint checking
- **`utils.rs`**: Utility functions and helper tools
- **`osm_parser.rs`**: OSM/PBF parsing and coordinate mapping utilities

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
# Run the main VRP demo
cargo run

# Convert OSM data to JSON (roads only)
cargo run --bin osm_converter -- --input map.osm.pbf --roads-only

# Complete Planet 83 workflow (NEW!)
cargo run --bin planet_83_example

# Explore OSM data
cargo run --bin osm_demo -- --stats
cargo run --bin osm_demo -- --lat 28.6139 --lon 77.2090  # Find nearest node

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
save_instance_to_json(&instance, "problem.json")?
save_solution_to_json(&solution, "solution.json")?

// Load instance and solution
let instance = load_instance_from_json("problem.json")?
let solution = load_solution_from_json("solution.json")?
```

### OSM Data Integration 🗺️

Convert OSM/PBF files to structured JSON for coordinate mapping:

```bash
# Convert OSM data (filter to roads only)
cargo run --bin osm_converter -- --input southern-zone-latest.osm.pbf --roads-only

# This creates: southern-zone-latest.osm.pbf.json (road network data)
```

Use parsed OSM data in your VRP solutions:

```rust
use vrp_solver::osm_parser::OsmParser;

// Load OSM data
let osm_data = load_osm_data_from_json("map.json")?;
let parser = OsmParser::with_data(osm_data);

// Find nearest road intersection for delivery location
let (node_id, distance) = parser.find_nearest_node(lat, lon).unwrap();
let (road_lat, road_lon) = parser.get_node_coordinates(node_id).unwrap();

// Use road coordinates for realistic routing
let location = Location::new(1, "Customer A", 
    Coordinate::new(road_lat, road_lon), demand, time_window, service_time);
```

### CSV Export

Export solutions to CSV for analysis:

```rust
export_solution_to_csv(&solution, &instance, "routes.csv")?
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

## 🗺️ OSM Integration Workflow

### 1. Download OSM Data
Get OSM/PBF files from [Geofabrik](https://download.geofabrik.de/) or other sources:

```bash
# Example: Download a regional map
wget https://download.geofabrik.de/asia/india-latest.osm.pbf
```

### 2. Convert to JSON
```bash
# Parse OSM data and filter to roads only
cargo run --bin osm_converter -- --input india-latest.osm.pbf --roads-only

# This processes millions of nodes and creates a structured JSON file
# Example output: 74M+ nodes → 29M+ road nodes, 4.76GB JSON file
```

### 🌍 Planet 83 Example Workflow

The project includes a complete example using the `planet_83.2932,17.7118_83.3388,17.7502.osm.pbf` file:

```bash
# Step 1: Parse the Planet 83 OSM data
cargo run --bin osm_converter -- --input planet_83.2932,17.7118_83.3388,17.7502.osm.pbf --roads-only
# Result: 62,319 nodes → 14,350 road nodes, 3,130 ways

# Step 2: Run complete VRP workflow example
cargo run --bin planet_83_example
# This demonstrates:
# - Loading OSM data (14,350 road nodes)
# - Mapping depot to nearest OSM node
# - Finding nearby customer locations from real road network
# - Creating VRP instance with actual coordinates
# - Solving VRP (2 routes, 233m total distance)
# - Exporting solution and GeoJSON

# Step 3: Visualize results
# Files created:
# - planet_83.2932,17.7118_83.3388,17.7502.osm.pbf.json (2.6MB OSM data)
# - planet_83.2932,17.7118_83.3388,17.7502.osm.pbf.geojson (6MB visualization)
# - planet_83_solution.json (VRP solution)
# - planet_83_routes.geojson (Routes for mapping)
```

### 3. Use in VRP Solutions
```rust
// Map your delivery locations to actual road coordinates
let nearest_node = osm_parser.find_nearest_node(delivery_lat, delivery_lon)?;
let road_coords = osm_parser.get_node_coordinates(nearest_node.0)?;

// Create VRP location using road network coordinates
let location = Location::new(id, name, 
    Coordinate::new(road_coords.0, road_coords.1), 
    demand, time_window, service_time);
```

### 4. Validate Routes Against Real Roads
Use the road network data to ensure your VRP routes follow actual streets and intersections.

### 🎯 Real-World Coordinate Mapping

The Planet 83 example demonstrates realistic VRP solving with actual road coordinates:

- **Depot Location**: 17.735°N, 83.315°E → OSM node 3688822252
- **Customer Mapping**: Target coordinates → Nearest road intersection nodes
- **Distance Calculations**: Haversine distance between actual road points  
- **Route Optimization**: Using real-world geographic constraints
- **Visualization**: GeoJSON export for mapping applications

Example output:
```
🏢 Depot mapped to OSM node 3688822252 (26.10m away)
👥 Found 8 customer locations:
   Customer 1: Node 3762201264 at 17.735386,83.314849 (24m from depot)
   Customer 2: Node 8208196541 at 17.735418,83.314836 (28m from depot)
   ...
✅ VRP solved: 2 routes, 233.15m total distance
```

## 🔮 Future Enhancements

- Genetic Algorithm solver
- Simulated Annealing
- Tabu Search
- **Real-time traffic integration** (with OSM road network)
- **Multi-depot support with geographic constraints**
- **Dynamic VRP with real-world road updates**
- **Route optimization using actual road distances**
- **GeoJSON export for map visualization**

## 📜 License

This project is open source. Feel free to use, modify, and distribute according to your needs.

## 🛠️ Command-Line Tools

### OSM Converter
Convert OSM/PBF files to structured JSON:

```bash
# Basic conversion with road filtering
cargo run --bin osm_converter -- --input map.osm.pbf --roads-only

# Custom output files
cargo run --bin osm_converter -- -i map.osm.pbf -j roads.json -g roads.geojson -r

# Full data (no filtering)
cargo run --bin osm_converter -- --input map.osm.pbf

# Planet 83 example (specific region)
cargo run --bin osm_converter -- --input planet_83.2932,17.7118_83.3388,17.7502.osm.pbf --roads-only
```

### Planet 83 VRP Example
Complete workflow demonstration with real OSM data:

```bash
# Run the complete Planet 83 workflow
cargo run --bin planet_83_example

# With custom parameters
cargo run --bin planet_83_example -- --customers 12 --depot-lat 17.740 --depot-lon 83.310

# This creates:
# - VRP instance using real road coordinates
# - Optimized routes with actual distances
# - GeoJSON files for visualization
```

### OSM Demo
Explore and test OSM data:

```bash
# Show dataset statistics
cargo run --bin osm_demo -- --stats

# Find nearest road node to coordinates
cargo run --bin osm_demo -- --lat 28.6139 --lon 77.2090

# Get coordinates for specific node ID
cargo run --bin osm_demo -- --node-id 123456789
```

## 🤝 Contributing

Contributions are welcome! Areas for improvement:
- Additional solving algorithms
- Performance optimizations for large OSM datasets
- More constraint types
- Better heuristics
- Real-world traffic data integration
- Documentation improvements
- GeoJSON streaming for large datasets

---

Built with ❤️ using Rust and Rayon for high-performance parallel computing.
