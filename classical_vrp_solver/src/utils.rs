//! Utility functions and helpers for the VRP solver

use crate::distance::{calculate_distance_matrix, calculate_time_matrix, DistanceMethod};
use crate::types::{Coordinate, Location, Solution, TimeWindow, Vehicle, VrpInstance};
use crate::{VrpError, VrpResult};
use serde_json;
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::path::Path;

/// Convert degrees to radians
pub fn deg_to_rad(degrees: f64) -> f64 {
    degrees * std::f64::consts::PI / 180.0
}

/// Convert radians to degrees
pub fn rad_to_deg(radians: f64) -> f64 {
    radians * 180.0 / std::f64::consts::PI
}

/// Convert latitude/longitude to UTM coordinates (simplified)
/// Returns (easting, northing) in meters for the given zone
pub fn lat_lon_to_utm(lat: f64, lon: f64, zone: u8) -> (f64, f64) {
    let lat_rad = deg_to_rad(lat);
    let lon_rad = deg_to_rad(lon);
    
    // Central meridian for the zone
    let central_meridian = deg_to_rad(-183.0 + (zone as f64) * 6.0);
    let lon_diff = lon_rad - central_meridian;
    
    // UTM constants
    let k0 = 0.9996; // Scale factor
    let a = 6378137.0; // WGS84 semi-major axis
    let e2 = 0.00669438; // WGS84 eccentricity squared
    
    let n = a / (1.0 - e2 * lat_rad.sin().powi(2)).sqrt();
    let t = lat_rad.tan().powi(2);
    let c = e2 * lat_rad.cos().powi(2) / (1.0 - e2);
    
    let easting = k0 * n * (lon_diff.cos() * lat_rad.tan().atan2(1.0)
        + lon_diff.powi(3) * lat_rad.cos().powi(3) * (1.0 - t + c) / 6.0);
    
    let northing = k0 * (lat_rad - lat_rad.sin() * lat_rad.cos() * 
        (lon_diff.powi(2) / 2.0 * (1.0 + t + c)));
    
    (500000.0 + easting, if lat >= 0.0 { northing } else { 10000000.0 + northing })
}

/// Parse coordinates from string format "lat,lon"
pub fn parse_coordinate(coord_str: &str) -> VrpResult<Coordinate> {
    let parts: Vec<&str> = coord_str.split(',').collect();
    if parts.len() != 2 {
        return Err(VrpError::InvalidInput(
            format!("Invalid coordinate format: '{}'. Expected 'lat,lon'", coord_str)
        ));
    }

    let lat = parts[0].trim().parse::<f64>()
        .map_err(|_| VrpError::InvalidInput(format!("Invalid latitude: '{}'", parts[0])))?;
    
    let lon = parts[1].trim().parse::<f64>()
        .map_err(|_| VrpError::InvalidInput(format!("Invalid longitude: '{}'", parts[1])))?;
    
    Ok(Coordinate::new(lat, lon))
}

/// Format coordinate as string "lat,lon"
pub fn format_coordinate(coord: Coordinate) -> String {
    format!("{:.6},{:.6}", coord.lat, coord.lon)
}

/// Calculate the center (centroid) of a set of coordinates
pub fn calculate_centroid(coordinates: &[Coordinate]) -> Option<Coordinate> {
    if coordinates.is_empty() {
        return None;
    }

    let sum_lat: f64 = coordinates.iter().map(|c| c.lat).sum();
    let sum_lon: f64 = coordinates.iter().map(|c| c.lon).sum();
    let count = coordinates.len() as f64;

    Some(Coordinate::new(sum_lat / count, sum_lon / count))
}

/// Calculate bounding box for a set of coordinates
pub fn calculate_bounding_box(coordinates: &[Coordinate]) -> Option<(Coordinate, Coordinate)> {
    if coordinates.is_empty() {
        return None;
    }

    let min_lat = coordinates.iter().map(|c| c.lat).fold(f64::INFINITY, f64::min);
    let max_lat = coordinates.iter().map(|c| c.lat).fold(f64::NEG_INFINITY, f64::max);
    let min_lon = coordinates.iter().map(|c| c.lon).fold(f64::INFINITY, f64::min);
    let max_lon = coordinates.iter().map(|c| c.lon).fold(f64::NEG_INFINITY, f64::max);

    Some((
        Coordinate::new(min_lat, min_lon),
        Coordinate::new(max_lat, max_lon),
    ))
}

/// Generate random coordinates within a bounding box
pub fn generate_random_coordinates(
    bounds: (Coordinate, Coordinate),
    count: usize,
    seed: Option<u64>,
) -> Vec<Coordinate> {
    use rand::{Rng, SeedableRng};
    use rand::rngs::StdRng;

    let mut rng = if let Some(seed_val) = seed {
        StdRng::seed_from_u64(seed_val)
    } else {
        StdRng::from_entropy()
    };

    let (min_coord, max_coord) = bounds;
    
    (0..count)
        .map(|_| {
            let lat = rng.gen_range(min_coord.lat..=max_coord.lat);
            let lon = rng.gen_range(min_coord.lon..=max_coord.lon);
            Coordinate::new(lat, lon)
        })
        .collect()
}

/// Create a VRP instance builder for easier instance creation
pub struct VrpInstanceBuilder {
    locations: Vec<Location>,
    vehicles: Vec<Vehicle>,
    distance_method: DistanceMethod,
    average_speed_ms: Option<f64>,
}

impl VrpInstanceBuilder {
    pub fn new() -> Self {
        Self {
            locations: Vec::new(),
            vehicles: Vec::new(),
            distance_method: DistanceMethod::Haversine,
            average_speed_ms: None,
        }
    }

    pub fn with_distance_method(mut self, method: DistanceMethod) -> Self {
        self.distance_method = method;
        self
    }

    pub fn with_average_speed(mut self, speed_ms: f64) -> Self {
        self.average_speed_ms = Some(speed_ms);
        self
    }

    pub fn add_location(mut self, location: Location) -> Self {
        self.locations.push(location);
        self
    }

    pub fn add_depot(mut self, id: usize, name: String, coordinate: Coordinate) -> Self {
        self.locations.push(Location::depot(id, name, coordinate));
        self
    }

    pub fn add_customer(
        mut self,
        id: usize,
        name: String,
        coordinate: Coordinate,
        demand: f64,
        time_window: Option<TimeWindow>,
        service_time: f64,
    ) -> Self {
        self.locations.push(Location::new(
            id, name, coordinate, demand, time_window, service_time,
        ));
        self
    }

    pub fn add_vehicle(mut self, vehicle: Vehicle) -> Self {
        self.vehicles.push(vehicle);
        self
    }

    pub fn add_vehicle_simple(
        mut self,
        id: usize,
        capacity: f64,
        depot_id: usize,
    ) -> Self {
        self.vehicles.push(Vehicle::new(id, capacity, None, None, depot_id));
        self
    }

    pub fn build(self) -> VrpResult<VrpInstance> {
        if self.locations.is_empty() {
            return Err(VrpError::InvalidInput("No locations provided".to_string()));
        }

        if self.vehicles.is_empty() {
            return Err(VrpError::InvalidInput("No vehicles provided".to_string()));
        }

        let mut instance = VrpInstance::new(self.locations, self.vehicles);
        
        // Calculate distance matrix
        calculate_distance_matrix(&mut instance, self.distance_method);
        
        // Calculate time matrix if speed is provided
        if let Some(speed) = self.average_speed_ms {
            calculate_time_matrix(&mut instance, speed);
        }

        Ok(instance)
    }
}

impl Default for VrpInstanceBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Load VRP instance from JSON file
pub fn load_instance_from_json<P: AsRef<Path>>(path: P) -> VrpResult<VrpInstance> {
    let file = File::open(&path)
        .map_err(|e| VrpError::InvalidInput(format!("Cannot open file {:?}: {}", path.as_ref(), e)))?;
    
    let reader = BufReader::new(file);
    let instance: VrpInstance = serde_json::from_reader(reader)
        .map_err(|e| VrpError::InvalidInput(format!("Cannot parse JSON: {}", e)))?;
    
    Ok(instance)
}

/// Save VRP instance to JSON file
pub fn save_instance_to_json<P: AsRef<Path>>(instance: &VrpInstance, path: P) -> VrpResult<()> {
    let file = File::create(&path)
        .map_err(|e| VrpError::InvalidInput(format!("Cannot create file {:?}: {}", path.as_ref(), e)))?;
    
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, instance)
        .map_err(|e| VrpError::InvalidInput(format!("Cannot write JSON: {}", e)))?;
    
    Ok(())
}

/// Load solution from JSON file
pub fn load_solution_from_json<P: AsRef<Path>>(path: P) -> VrpResult<Solution> {
    let file = File::open(&path)
        .map_err(|e| VrpError::InvalidInput(format!("Cannot open file {:?}: {}", path.as_ref(), e)))?;
    
    let reader = BufReader::new(file);
    let solution: Solution = serde_json::from_reader(reader)
        .map_err(|e| VrpError::InvalidInput(format!("Cannot parse JSON: {}", e)))?;
    
    Ok(solution)
}

/// Save solution to JSON file
pub fn save_solution_to_json<P: AsRef<Path>>(solution: &Solution, path: P) -> VrpResult<()> {
    let file = File::create(&path)
        .map_err(|e| VrpError::InvalidInput(format!("Cannot create file {:?}: {}", path.as_ref(), e)))?;
    
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, solution)
        .map_err(|e| VrpError::InvalidInput(format!("Cannot write JSON: {}", e)))?;
    
    Ok(())
}

/// Create a solution summary string
pub fn format_solution_summary(solution: &Solution) -> String {
    format!(
        "Solution Summary:\n\
         - Routes: {}\n\
         - Total Distance: {:.2} m\n\
         - Total Duration: {:.2} s\n\
         - Vehicles Used: {}\n\
         - Average Distance per Route: {:.2} m\n\
         - Average Duration per Route: {:.2} s",
        solution.routes.len(),
        solution.total_distance,
        solution.total_duration,
        solution.num_vehicles_used,
        solution.total_distance / solution.routes.len().max(1) as f64,
        solution.total_duration / solution.routes.len().max(1) as f64,
    )
}

/// Export solution to CSV format
pub fn export_solution_to_csv<P: AsRef<Path>>(
    solution: &Solution,
    instance: &VrpInstance,
    path: P,
) -> VrpResult<()> {
    let mut file = File::create(&path)
        .map_err(|e| VrpError::InvalidInput(format!("Cannot create file {:?}: {}", path.as_ref(), e)))?;

    writeln!(file, "route_id,vehicle_id,location_id,location_name,latitude,longitude,demand,service_time,arrival_time,departure_time")
        .map_err(|e| VrpError::InvalidInput(format!("Cannot write to file: {}", e)))?;

    for (route_idx, route) in solution.routes.iter().enumerate() {
        let mut current_time = 0.0;
        
        for (_seq, &location_id) in route.locations.iter().enumerate() {
            if let Some(location) = instance.get_location(location_id) {
                let arrival_time = current_time;
                let departure_time = current_time + location.service_time;
                
                writeln!(
                    file,
                    "{},{},{},{},{:.6},{:.6},{:.2},{:.2},{:.2},{:.2}",
                    route_idx + 1,
                    route.vehicle_id,
                    location.id,
                    location.name,
                    location.coordinate.lat,
                    location.coordinate.lon,
                    location.demand,
                    location.service_time,
                    arrival_time,
                    departure_time
                ).map_err(|e| VrpError::InvalidInput(format!("Cannot write to file: {}", e)))?;
                
                current_time = departure_time;
            }
        }
    }

    Ok(())
}

/// Create test instance for development and testing
pub fn create_test_instance(num_customers: usize, depot_coord: Coordinate) -> VrpInstance {
    let mut builder = VrpInstanceBuilder::new();
    
    // Add depot
    builder = builder.add_depot(0, "Main Depot".to_string(), depot_coord);
    
    // Generate random customer locations around depot
    let bounds = (
        Coordinate::new(depot_coord.lat - 0.1, depot_coord.lon - 0.1),
        Coordinate::new(depot_coord.lat + 0.1, depot_coord.lon + 0.1),
    );
    
    let customer_coords = generate_random_coordinates(bounds, num_customers, Some(42));
    
    // Add customers with random demands
    use rand::{Rng, SeedableRng};
    use rand::rngs::StdRng;
    let mut rng = StdRng::seed_from_u64(42);
    
    for (i, coord) in customer_coords.into_iter().enumerate() {
        let demand = rng.gen_range(5.0..25.0);
        let service_time = rng.gen_range(300.0..900.0); // 5-15 minutes
        
        builder = builder.add_customer(
            i + 1,
            format!("Customer {}", i + 1),
            coord,
            demand,
            None,
            service_time,
        );
    }
    
    // Add vehicles
    let num_vehicles = (num_customers / 5).max(1);
    for i in 0..num_vehicles {
        builder = builder.add_vehicle_simple(i, 100.0, 0);
    }
    
    builder
        .with_distance_method(DistanceMethod::Haversine)
        .with_average_speed(15.0) // 15 m/s ≈ 54 km/h
        .build()
        .expect("Failed to create test instance")
}

/// Performance metrics for solution comparison
#[derive(Debug, Clone)]
pub struct SolutionMetrics {
    pub total_distance: f64,
    pub total_duration: f64,
    pub num_vehicles: usize,
    pub num_routes: usize,
    pub average_route_distance: f64,
    pub average_route_duration: f64,
    pub max_route_distance: f64,
    pub min_route_distance: f64,
    pub distance_std_dev: f64,
}

impl SolutionMetrics {
    pub fn from_solution(solution: &Solution) -> Self {
        let distances: Vec<f64> = solution.routes.iter().map(|r| r.total_distance).collect();
        let durations: Vec<f64> = solution.routes.iter().map(|r| r.total_duration).collect();
        
        let avg_distance = if distances.is_empty() { 0.0 } else { distances.iter().sum::<f64>() / distances.len() as f64 };
        let avg_duration = if durations.is_empty() { 0.0 } else { durations.iter().sum::<f64>() / durations.len() as f64 };
        
        let max_distance = distances.iter().copied().fold(0.0, f64::max);
        let min_distance = distances.iter().copied().fold(f64::INFINITY, f64::min);
        
        // Calculate standard deviation of distances
        let distance_variance = if distances.len() > 1 {
            distances.iter()
                .map(|&d| (d - avg_distance).powi(2))
                .sum::<f64>() / (distances.len() - 1) as f64
        } else {
            0.0
        };
        let distance_std_dev = distance_variance.sqrt();
        
        Self {
            total_distance: solution.total_distance,
            total_duration: solution.total_duration,
            num_vehicles: solution.num_vehicles_used,
            num_routes: solution.routes.len(),
            average_route_distance: avg_distance,
            average_route_duration: avg_duration,
            max_route_distance: max_distance,
            min_route_distance: if min_distance.is_infinite() { 0.0 } else { min_distance },
            distance_std_dev,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coordinate_parsing() {
        let coord = parse_coordinate("52.5200,13.4050").unwrap();
        assert!((coord.lat - 52.5200).abs() < 1e-6);
        assert!((coord.lon - 13.4050).abs() < 1e-6);
    }

    #[test]
    fn test_coordinate_formatting() {
        let coord = Coordinate::new(52.5200, 13.4050);
        let formatted = format_coordinate(coord);
        assert_eq!(formatted, "52.520000,13.405000");
    }

    #[test]
    fn test_centroid_calculation() {
        let coords = vec![
            Coordinate::new(0.0, 0.0),
            Coordinate::new(2.0, 2.0),
            Coordinate::new(4.0, 4.0),
        ];
        
        let centroid = calculate_centroid(&coords).unwrap();
        assert!((centroid.lat - 2.0).abs() < 1e-6);
        assert!((centroid.lon - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_instance_builder() {
        let instance = VrpInstanceBuilder::new()
            .add_depot(0, "Depot".to_string(), Coordinate::new(0.0, 0.0))
            .add_customer(1, "Customer 1".to_string(), Coordinate::new(1.0, 1.0), 10.0, None, 5.0)
            .add_vehicle_simple(0, 50.0, 0)
            .build()
            .unwrap();
        
        assert_eq!(instance.locations.len(), 2);
        assert_eq!(instance.vehicles.len(), 1);
    }

    #[test]
    fn test_test_instance_creation() {
        let instance = create_test_instance(5, Coordinate::new(52.5200, 13.4050));
        assert_eq!(instance.locations.len(), 6); // 1 depot + 5 customers
        assert!(!instance.vehicles.is_empty());
    }
}
