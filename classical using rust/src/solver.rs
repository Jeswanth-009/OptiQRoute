//! Vehicle Routing Problem solving algorithms

use crate::distance::{calculate_route_distance, calculate_route_duration, calculate_savings};
use crate::types::{Route, Solution, VrpInstance};
use crate::{VrpError, VrpResult};
use rayon::prelude::*;
use std::collections::HashMap;

/// VRP Solver trait for different algorithms
pub trait VrpSolver {
    fn solve(&self, instance: &VrpInstance) -> VrpResult<Solution>;
    fn name(&self) -> &'static str;
}

/// Greedy Nearest Neighbor algorithm
#[derive(Debug, Default)]
pub struct GreedyNearestNeighbor {
    /// Start with the farthest customer from depot
    pub start_farthest: bool,
}

impl GreedyNearestNeighbor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_farthest_start(mut self, start_farthest: bool) -> Self {
        self.start_farthest = start_farthest;
        self
    }

    fn find_depot_index(&self, instance: &VrpInstance, vehicle_id: usize) -> VrpResult<usize> {
        let vehicle = instance.get_vehicle(vehicle_id)
            .ok_or_else(|| VrpError::InvalidInput(format!("Vehicle {} not found", vehicle_id)))?;
        
        instance.locations
            .iter()
            .position(|loc| loc.id == vehicle.depot_id)
            .ok_or_else(|| VrpError::InvalidInput(format!("Depot {} not found", vehicle.depot_id)))
    }

    fn build_route(&self, instance: &VrpInstance, vehicle_id: usize, unvisited: &mut Vec<usize>) -> VrpResult<Route> {
        let mut route = Route::new(vehicle_id);
        let depot_idx = self.find_depot_index(instance, vehicle_id)?;
        let vehicle = instance.get_vehicle(vehicle_id).unwrap();

        if unvisited.is_empty() {
            return Ok(route);
        }

        // Choose starting customer
        let start_idx = if self.start_farthest {
            // Find the customer farthest from depot
            unvisited.iter()
                .max_by(|&&a, &&b| {
                    instance.distance_matrix[depot_idx][a]
                        .partial_cmp(&instance.distance_matrix[depot_idx][b])
                        .unwrap()
                })
                .copied()
                .unwrap()
        } else {
            // Find the customer nearest to depot
            unvisited.iter()
                .min_by(|&&a, &&b| {
                    instance.distance_matrix[depot_idx][a]
                        .partial_cmp(&instance.distance_matrix[depot_idx][b])
                        .unwrap()
                })
                .copied()
                .unwrap()
        };

        // Add first customer
        let start_location = &instance.locations[start_idx];
        route.add_location(start_location.id);
        route.total_demand += start_location.demand;
        unvisited.retain(|&x| x != start_idx);

        let mut current_idx = start_idx;

        // Greedy nearest neighbor selection
        while !unvisited.is_empty() {
            let mut best_next: Option<(usize, f64)> = None;

            for &candidate_idx in unvisited.iter() {
                let candidate_location = &instance.locations[candidate_idx];
                
                // Check capacity constraint
                if route.total_demand + candidate_location.demand > vehicle.capacity {
                    continue;
                }

                let distance = instance.distance_matrix[current_idx][candidate_idx];
                
                if best_next.is_none() || distance < best_next.unwrap().1 {
                    best_next = Some((candidate_idx, distance));
                }
            }

            if let Some((next_idx, _)) = best_next {
                let next_location = &instance.locations[next_idx];
                route.add_location(next_location.id);
                route.total_demand += next_location.demand;
                current_idx = next_idx;
                unvisited.retain(|&x| x != next_idx);
            } else {
                // No more feasible customers for this vehicle
                break;
            }
        }

        // Calculate route metrics
        let route_indices: Vec<usize> = route.locations
            .iter()
            .filter_map(|&id| instance.locations.iter().position(|loc| loc.id == id))
            .collect();

        route.total_distance = calculate_route_distance(instance, &route_indices, depot_idx);
        
        if let Some(duration) = calculate_route_duration(instance, &route_indices, depot_idx) {
            route.total_duration = duration;
        }

        Ok(route)
    }
}

impl VrpSolver for GreedyNearestNeighbor {
    fn solve(&self, instance: &VrpInstance) -> VrpResult<Solution> {
        let mut solution = Solution::new();
        
        // Get all customer indices (excluding depots)
        let mut unvisited: Vec<usize> = instance.locations
            .iter()
            .enumerate()
            .filter_map(|(idx, loc)| {
                if loc.demand > 0.0 { Some(idx) } else { None }
            })
            .collect();

        // Build routes for each vehicle until all customers are served
        for vehicle in &instance.vehicles {
            if unvisited.is_empty() {
                break;
            }

            let route = self.build_route(instance, vehicle.id, &mut unvisited)?;
            solution.add_route(route);
        }

        if !unvisited.is_empty() {
            return Err(VrpError::NoSolutionFound);
        }

        Ok(solution)
    }

    fn name(&self) -> &'static str {
        "Greedy Nearest Neighbor"
    }
}

/// Clarke-Wright Savings algorithm
#[derive(Debug, Default)]
pub struct ClarkeWrightSavings {
    /// Whether to use parallel processing for savings calculation
    pub parallel: bool,
}

impl ClarkeWrightSavings {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_parallel(mut self, parallel: bool) -> Self {
        self.parallel = parallel;
        self
    }

    fn find_depot_index(&self, instance: &VrpInstance, depot_id: usize) -> VrpResult<usize> {
        instance.locations
            .iter()
            .position(|loc| loc.id == depot_id)
            .ok_or_else(|| VrpError::InvalidInput(format!("Depot {} not found", depot_id)))
    }
}

impl VrpSolver for ClarkeWrightSavings {
    fn solve(&self, instance: &VrpInstance) -> VrpResult<Solution> {
        if instance.vehicles.is_empty() {
            return Err(VrpError::InvalidInput("No vehicles available".to_string()));
        }

        let depot_id = instance.vehicles[0].depot_id;
        let depot_idx = self.find_depot_index(instance, depot_id)?;

        // Initialize: each customer has its own route
        let mut routes: Vec<Route> = Vec::new();
        let mut customer_to_route: HashMap<usize, usize> = HashMap::new();

        for (idx, location) in instance.locations.iter().enumerate() {
            if location.demand > 0.0 && idx != depot_idx {
                let mut route = Route::new(0); // Will assign vehicle later
                route.add_location(location.id);
                route.total_demand = location.demand;
                
                // Calculate initial route metrics (depot -> customer -> depot)
                route.total_distance = calculate_route_distance(instance, &[idx], depot_idx);
                
                if let Some(duration) = calculate_route_duration(instance, &[idx], depot_idx) {
                    route.total_duration = duration;
                }

                customer_to_route.insert(location.id, routes.len());
                routes.push(route);
            }
        }

        // Calculate all savings
        let mut savings = calculate_savings(instance, depot_id);
        
        // Sort savings in descending order
        savings.sort_by(|a, b| b.value.partial_cmp(&a.value).unwrap());

        // Process savings to merge routes
        for saving in savings {
            let route_i = customer_to_route.get(&saving.from);
            let route_j = customer_to_route.get(&saving.to);

            if let (Some(&ri), Some(&rj)) = (route_i, route_j) {
                if ri != rj && ri < routes.len() && rj < routes.len() {
                    let route_i_ref = &routes[ri];
                    let route_j_ref = &routes[rj];

                    // Check if routes can be merged (capacity constraints)
                    let total_demand = route_i_ref.total_demand + route_j_ref.total_demand;
                    
                    // Check if any vehicle can handle this merged route
                    let can_merge = instance.vehicles.iter().any(|vehicle| {
                        total_demand <= vehicle.capacity
                    });

                    if can_merge {
                        // Check if customers are at the end/start of routes for proper merging
                        let route_i_start = route_i_ref.locations.first();
                        let route_i_end = route_i_ref.locations.last();
                        let route_j_start = route_j_ref.locations.first();
                        let route_j_end = route_j_ref.locations.last();

                        let can_connect = 
                            (route_i_end == Some(&saving.from) && route_j_start == Some(&saving.to)) ||
                            (route_i_end == Some(&saving.to) && route_j_start == Some(&saving.from)) ||
                            (route_i_start == Some(&saving.from) && route_j_end == Some(&saving.to)) ||
                            (route_i_start == Some(&saving.to) && route_j_end == Some(&saving.from));

                        if can_connect {
                            // Merge routes (this is a simplified version)
                            let mut new_route = route_i_ref.clone();
                            
                            // Add locations from route j
                            for &loc_id in &route_j_ref.locations {
                                if !new_route.locations.contains(&loc_id) {
                                    new_route.locations.push(loc_id);
                                }
                            }
                            
                            new_route.total_demand = total_demand;

                            // Recalculate route metrics
                            let route_indices: Vec<usize> = new_route.locations
                                .iter()
                                .filter_map(|&id| instance.locations.iter().position(|loc| loc.id == id))
                                .collect();

                            new_route.total_distance = calculate_route_distance(instance, &route_indices, depot_idx);
                            
                            if let Some(duration) = calculate_route_duration(instance, &route_indices, depot_idx) {
                                new_route.total_duration = duration;
                            }

                            // Update customer mappings
                            for &loc_id in &new_route.locations {
                                customer_to_route.insert(loc_id, ri);
                            }

                            // Replace route i with merged route and mark route j as empty
                            routes[ri] = new_route;
                            routes[rj] = Route::new(0); // Empty route
                        }
                    }
                }
            }
        }

        // Filter out empty routes and assign vehicles
        let mut solution = Solution::new();
        let mut vehicle_iter = instance.vehicles.iter();

        for route in routes.into_iter().filter(|r| !r.is_empty()) {
            if let Some(vehicle) = vehicle_iter.next() {
                let mut final_route = route;
                final_route.vehicle_id = vehicle.id;
                solution.add_route(final_route);
            }
        }

        if solution.routes.is_empty() {
            Err(VrpError::NoSolutionFound)
        } else {
            Ok(solution)
        }
    }

    fn name(&self) -> &'static str {
        "Clarke-Wright Savings"
    }
}

/// Multi-start solver that runs multiple algorithms and returns the best solution
#[derive(Default)]
pub struct MultiStartSolver {
    solvers: Vec<Box<dyn VrpSolver + Sync>>,
}

impl MultiStartSolver {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_solver(mut self, solver: Box<dyn VrpSolver + Sync>) -> Self {
        self.solvers.push(solver);
        self
    }

    pub fn with_default_solvers(self) -> Self {
        self.add_solver(Box::new(GreedyNearestNeighbor::new()))
            .add_solver(Box::new(GreedyNearestNeighbor::new().with_farthest_start(true)))
            .add_solver(Box::new(ClarkeWrightSavings::new().with_parallel(true)))
    }
}

impl VrpSolver for MultiStartSolver {
    fn solve(&self, instance: &VrpInstance) -> VrpResult<Solution> {
        if self.solvers.is_empty() {
            return Err(VrpError::InvalidInput("No solvers configured".to_string()));
        }

        // Run all solvers in parallel
        let results: Vec<VrpResult<Solution>> = self.solvers
            .par_iter()
            .map(|solver| solver.solve(instance))
            .collect();

        // Find the best valid solution
        let mut best_solution: Option<Solution> = None;
        let mut best_distance = f64::INFINITY;

        for result in results {
            if let Ok(solution) = result {
                if solution.is_valid() && solution.total_distance < best_distance {
                    best_distance = solution.total_distance;
                    best_solution = Some(solution);
                }
            }
        }

        best_solution.ok_or(VrpError::NoSolutionFound)
    }

    fn name(&self) -> &'static str {
        "Multi-Start Solver"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::distance::{calculate_distance_matrix, DistanceMethod};
    use crate::types::*;

    fn create_test_instance() -> VrpInstance {
        let locations = vec![
            Location::depot(0, "Depot".to_string(), Coordinate::new(0.0, 0.0)),
            Location::new(1, "Customer 1".to_string(), Coordinate::new(1.0, 1.0), 10.0, None, 5.0),
            Location::new(2, "Customer 2".to_string(), Coordinate::new(2.0, 2.0), 15.0, None, 5.0),
            Location::new(3, "Customer 3".to_string(), Coordinate::new(-1.0, 1.0), 8.0, None, 5.0),
        ];

        let vehicles = vec![
            Vehicle::new(0, 50.0, None, None, 0),
            Vehicle::new(1, 30.0, None, None, 0),
        ];

        let mut instance = VrpInstance::new(locations, vehicles);
        calculate_distance_matrix(&mut instance, DistanceMethod::Euclidean);
        instance
    }

    #[test]
    fn test_greedy_nearest_neighbor() {
        let instance = create_test_instance();
        let solver = GreedyNearestNeighbor::new();
        
        let result = solver.solve(&instance);
        assert!(result.is_ok());
        
        let solution = result.unwrap();
        assert!(solution.is_valid());
        assert!(!solution.routes.is_empty());
    }

    #[test]
    fn test_clarke_wright_savings() {
        let instance = create_test_instance();
        let solver = ClarkeWrightSavings::new();
        
        let result = solver.solve(&instance);
        assert!(result.is_ok());
        
        let solution = result.unwrap();
        assert!(solution.is_valid());
    }

    #[test]
    fn test_multi_start_solver() {
        let instance = create_test_instance();
        let solver = MultiStartSolver::new().with_default_solvers();
        
        let result = solver.solve(&instance);
        assert!(result.is_ok());
        
        let solution = result.unwrap();
        assert!(solution.is_valid());
    }
}
