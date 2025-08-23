//! Core data structures for the Vehicle Routing Problem solver

use serde::{Deserialize, Serialize};

/// Geographic coordinate (latitude, longitude)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Coordinate {
    pub lat: f64,
    pub lon: f64,
}

impl Coordinate {
    pub fn new(lat: f64, lon: f64) -> Self {
        Self { lat, lon }
    }
}

/// A location in the VRP problem (depot or customer)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub id: usize,
    pub name: String,
    pub coordinate: Coordinate,
    pub demand: f64,
    pub time_window: Option<TimeWindow>,
    pub service_time: f64, // Time required to service this location
}

impl Location {
    pub fn new(
        id: usize,
        name: String,
        coordinate: Coordinate,
        demand: f64,
        time_window: Option<TimeWindow>,
        service_time: f64,
    ) -> Self {
        Self {
            id,
            name,
            coordinate,
            demand,
            time_window,
            service_time,
        }
    }

    /// Create a depot location (typically with zero demand)
    pub fn depot(id: usize, name: String, coordinate: Coordinate) -> Self {
        Self::new(id, name, coordinate, 0.0, None, 0.0)
    }
}

/// Time window constraint for a location
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TimeWindow {
    pub start: f64,
    pub end: f64,
}

impl TimeWindow {
    pub fn new(start: f64, end: f64) -> Self {
        Self { start, end }
    }

    pub fn contains(&self, time: f64) -> bool {
        time >= self.start && time <= self.end
    }
}

/// Vehicle definition with constraints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vehicle {
    pub id: usize,
    pub capacity: f64,
    pub max_distance: Option<f64>,
    pub max_duration: Option<f64>,
    pub depot_id: usize,
}

impl Vehicle {
    pub fn new(
        id: usize,
        capacity: f64,
        max_distance: Option<f64>,
        max_duration: Option<f64>,
        depot_id: usize,
    ) -> Self {
        Self {
            id,
            capacity,
            max_distance,
            max_duration,
            depot_id,
        }
    }
}

/// A route for a single vehicle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Route {
    pub vehicle_id: usize,
    pub locations: Vec<usize>, // Location IDs in visiting order
    pub total_distance: f64,
    pub total_duration: f64,
    pub total_demand: f64,
}

impl Route {
    pub fn new(vehicle_id: usize) -> Self {
        Self {
            vehicle_id,
            locations: Vec::new(),
            total_distance: 0.0,
            total_duration: 0.0,
            total_demand: 0.0,
        }
    }

    pub fn add_location(&mut self, location_id: usize) {
        self.locations.push(location_id);
    }

    pub fn is_empty(&self) -> bool {
        self.locations.is_empty()
    }

    pub fn len(&self) -> usize {
        self.locations.len()
    }
}

/// Complete solution to a VRP instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Solution {
    pub routes: Vec<Route>,
    pub total_distance: f64,
    pub total_duration: f64,
    pub num_vehicles_used: usize,
}

impl Solution {
    pub fn new() -> Self {
        Self {
            routes: Vec::new(),
            total_distance: 0.0,
            total_duration: 0.0,
            num_vehicles_used: 0,
        }
    }

    pub fn add_route(&mut self, route: Route) {
        if !route.is_empty() {
            self.total_distance += route.total_distance;
            self.total_duration += route.total_duration;
            self.num_vehicles_used += 1;
            self.routes.push(route);
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.routes.is_empty() && self.num_vehicles_used > 0
    }
}

impl Default for Solution {
    fn default() -> Self {
        Self::new()
    }
}

/// VRP problem instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VrpInstance {
    pub locations: Vec<Location>,
    pub vehicles: Vec<Vehicle>,
    pub distance_matrix: Vec<Vec<f64>>,
    pub time_matrix: Option<Vec<Vec<f64>>>,
}

impl VrpInstance {
    pub fn new(locations: Vec<Location>, vehicles: Vec<Vehicle>) -> Self {
        let n = locations.len();
        Self {
            locations,
            vehicles,
            distance_matrix: vec![vec![0.0; n]; n],
            time_matrix: None,
        }
    }

    pub fn get_location(&self, id: usize) -> Option<&Location> {
        self.locations.iter().find(|loc| loc.id == id)
    }

    pub fn get_vehicle(&self, id: usize) -> Option<&Vehicle> {
        self.vehicles.iter().find(|vehicle| vehicle.id == id)
    }

    pub fn get_distance(&self, from: usize, to: usize) -> f64 {
        if from < self.distance_matrix.len() && to < self.distance_matrix[from].len() {
            self.distance_matrix[from][to]
        } else {
            f64::INFINITY
        }
    }

    pub fn num_locations(&self) -> usize {
        self.locations.len()
    }

    pub fn num_vehicles(&self) -> usize {
        self.vehicles.len()
    }
}

/// Savings value for Clarke-Wright algorithm
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Saving {
    pub from: usize,
    pub to: usize,
    pub value: f64,
}

impl Saving {
    pub fn new(from: usize, to: usize, value: f64) -> Self {
        Self { from, to, value }
    }
}

impl PartialOrd for Saving {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        other.value.partial_cmp(&self.value) // Reverse order for max-heap behavior
    }
}
