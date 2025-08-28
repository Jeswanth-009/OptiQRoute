//! Distance and time matrix calculations using parallel processing

use crate::types::{Coordinate, VrpInstance};
use rayon::prelude::*;

/// Calculate haversine distance between two coordinates (in meters)
pub fn haversine_distance(coord1: Coordinate, coord2: Coordinate) -> f64 {
    const EARTH_RADIUS_M: f64 = 6_371_000.0; // Earth's radius in meters

    let lat1_rad = coord1.lat.to_radians();
    let lat2_rad = coord2.lat.to_radians();
    let delta_lat = (coord2.lat - coord1.lat).to_radians();
    let delta_lon = (coord2.lon - coord1.lon).to_radians();

    let a = (delta_lat / 2.0).sin().powi(2)
        + lat1_rad.cos() * lat2_rad.cos() * (delta_lon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

    EARTH_RADIUS_M * c
}

/// Calculate Manhattan distance between two coordinates (approximation in meters)
pub fn manhattan_distance(coord1: Coordinate, coord2: Coordinate) -> f64 {
    const METERS_PER_DEGREE_LAT: f64 = 111_320.0;
    
    let lat_avg = (coord1.lat + coord2.lat) / 2.0;
    let meters_per_degree_lon = METERS_PER_DEGREE_LAT * lat_avg.to_radians().cos();
    
    let lat_diff = (coord2.lat - coord1.lat).abs() * METERS_PER_DEGREE_LAT;
    let lon_diff = (coord2.lon - coord1.lon).abs() * meters_per_degree_lon;
    
    lat_diff + lon_diff
}

/// Calculate Euclidean distance between two coordinates (approximation in meters)
pub fn euclidean_distance(coord1: Coordinate, coord2: Coordinate) -> f64 {
    const METERS_PER_DEGREE_LAT: f64 = 111_320.0;
    
    let lat_avg = (coord1.lat + coord2.lat) / 2.0;
    let meters_per_degree_lon = METERS_PER_DEGREE_LAT * lat_avg.to_radians().cos();
    
    let lat_diff = (coord2.lat - coord1.lat) * METERS_PER_DEGREE_LAT;
    let lon_diff = (coord2.lon - coord1.lon) * meters_per_degree_lon;
    
    (lat_diff.powi(2) + lon_diff.powi(2)).sqrt()
}

/// Distance calculation method
#[derive(Debug, Clone, Copy)]
pub enum DistanceMethod {
    Haversine,
    Manhattan,
    Euclidean,
}

/// Calculate distance matrix using parallel processing
pub fn calculate_distance_matrix(
    instance: &mut VrpInstance,
    method: DistanceMethod,
) -> &Vec<Vec<f64>> {
    let n = instance.locations.len();
    
    // Create coordinate pairs for parallel processing by copying coordinates
    let locations = &instance.locations;
    let coord_pairs: Vec<(usize, usize, Coordinate, Coordinate)> = (0..n)
        .flat_map(|i| {
            (0..n).map(move |j| {
                (i, j, locations[i].coordinate, locations[j].coordinate)
            })
        })
        .collect();

    // Calculate distances in parallel
    let distances: Vec<(usize, usize, f64)> = coord_pairs
        .par_iter()
        .map(|&(i, j, coord1, coord2)| {
            let distance = if i == j {
                0.0
            } else {
                match method {
                    DistanceMethod::Haversine => haversine_distance(coord1, coord2),
                    DistanceMethod::Manhattan => manhattan_distance(coord1, coord2),
                    DistanceMethod::Euclidean => euclidean_distance(coord1, coord2),
                }
            };
            (i, j, distance)
        })
        .collect();

    // Populate the distance matrix
    for (i, j, distance) in distances {
        instance.distance_matrix[i][j] = distance;
    }

    &instance.distance_matrix
}

/// Calculate time matrix based on distance matrix and average speed
pub fn calculate_time_matrix(
    instance: &mut VrpInstance,
    average_speed_ms: f64, // meters per second
) -> &Option<Vec<Vec<f64>>> {
    let n = instance.locations.len();
    let mut time_matrix = vec![vec![0.0; n]; n];

    // Calculate travel times in parallel
    let times: Vec<(usize, usize, f64)> = (0..n)
        .flat_map(|i| (0..n).map(move |j| (i, j)))
        .collect::<Vec<_>>()
        .par_iter()
        .map(|&(i, j)| {
            let time = if i == j {
                0.0
            } else {
                instance.distance_matrix[i][j] / average_speed_ms
            };
            (i, j, time)
        })
        .collect();

    // Populate the time matrix
    for (i, j, time) in times {
        time_matrix[i][j] = time;
    }

    instance.time_matrix = Some(time_matrix);
    &instance.time_matrix
}

/// Calculate nearest neighbors for each location using parallel processing
pub fn calculate_nearest_neighbors(
    instance: &VrpInstance,
    k: usize,
) -> Vec<Vec<(usize, f64)>> {
    let n = instance.locations.len();
    
    (0..n)
        .into_par_iter()
        .map(|i| {
            let mut neighbors: Vec<(usize, f64)> = (0..n)
                .filter(|&j| i != j)
                .map(|j| (j, instance.distance_matrix[i][j]))
                .collect();
            
            neighbors.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
            neighbors.truncate(k);
            neighbors
        })
        .collect()
}

/// Calculate all pairwise savings for Clarke-Wright algorithm using parallel processing
pub fn calculate_savings(instance: &VrpInstance, depot_id: usize) -> Vec<crate::types::Saving> {
    let n = instance.locations.len();
    let depot_idx = instance.locations
        .iter()
        .position(|loc| loc.id == depot_id)
        .unwrap_or(0);

    // Generate all pairs (excluding depot)
    let pairs: Vec<(usize, usize)> = (0..n)
        .flat_map(|i| {
            (i + 1..n)
                .filter(move |&j| i != depot_idx && j != depot_idx)
                .map(move |j| (i, j))
        })
        .collect();

    // Calculate savings in parallel
    pairs
        .par_iter()
        .map(|&(i, j)| {
            let saving_value = instance.distance_matrix[depot_idx][i]
                + instance.distance_matrix[depot_idx][j]
                - instance.distance_matrix[i][j];
            
            crate::types::Saving::new(
                instance.locations[i].id,
                instance.locations[j].id,
                saving_value,
            )
        })
        .collect()
}

/// Calculate route distance given a sequence of location indices
pub fn calculate_route_distance(
    instance: &VrpInstance,
    route: &[usize],
    depot_idx: usize,
) -> f64 {
    if route.is_empty() {
        return 0.0;
    }

    let mut total_distance = 0.0;

    // Distance from depot to first location
    total_distance += instance.distance_matrix[depot_idx][route[0]];

    // Distances between consecutive locations
    for window in route.windows(2) {
        total_distance += instance.distance_matrix[window[0]][window[1]];
    }

    // Distance from last location back to depot
    if let Some(&last) = route.last() {
        total_distance += instance.distance_matrix[last][depot_idx];
    }

    total_distance
}

/// Calculate route duration given a sequence of location indices
pub fn calculate_route_duration(
    instance: &VrpInstance,
    route: &[usize],
    depot_idx: usize,
) -> Option<f64> {
    if let Some(ref time_matrix) = instance.time_matrix {
        if route.is_empty() {
            return Some(0.0);
        }

        let mut total_duration = 0.0;

        // Time from depot to first location
        total_duration += time_matrix[depot_idx][route[0]];

        // Times between consecutive locations + service times
        for window in route.windows(2) {
            let from_idx = window[0];
            let to_idx = window[1];
            
            // Add service time at the 'from' location
            if let Some(location) = instance.locations.get(from_idx) {
                total_duration += location.service_time;
            }
            
            // Add travel time
            total_duration += time_matrix[from_idx][to_idx];
        }

        // Add service time at the last location
        if let Some(&last_idx) = route.last() {
            if let Some(location) = instance.locations.get(last_idx) {
                total_duration += location.service_time;
            }
        }

        // Time from last location back to depot
        if let Some(&last) = route.last() {
            total_duration += time_matrix[last][depot_idx];
        }

        Some(total_duration)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_haversine_distance() {
        let coord1 = Coordinate::new(52.5200, 13.4050); // Berlin
        let coord2 = Coordinate::new(48.8566, 2.3522);  // Paris
        
        let distance = haversine_distance(coord1, coord2);
        
        // Approximate distance between Berlin and Paris is ~878 km
        assert!((distance - 878_000.0).abs() < 10_000.0);
    }

    #[test]
    fn test_distance_matrix_calculation() {
        let locations = vec![
            crate::types::Location::new(
                0,
                "Depot".to_string(),
                Coordinate::new(0.0, 0.0),
                0.0,
                None,
                0.0,
            ),
            crate::types::Location::new(
                1,
                "Customer 1".to_string(),
                Coordinate::new(1.0, 1.0),
                10.0,
                None,
                5.0,
            ),
        ];
        
        let vehicles = vec![
            crate::types::Vehicle::new(0, 100.0, None, None, 0)
        ];
        
        let mut instance = VrpInstance::new(locations, vehicles);
        calculate_distance_matrix(&mut instance, DistanceMethod::Haversine);
        
        assert_eq!(instance.distance_matrix[0][0], 0.0);
        assert!(instance.distance_matrix[0][1] > 0.0);
        assert_eq!(instance.distance_matrix[0][1], instance.distance_matrix[1][0]);
    }
}
