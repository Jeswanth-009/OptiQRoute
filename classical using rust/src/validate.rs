//! Route validation functions for VRP constraints

use crate::distance::{calculate_route_distance, calculate_route_duration};
use crate::types::{Route, Solution, VrpInstance};
use crate::{VrpError, VrpResult};

/// Validation result for a route
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub violations: Vec<String>,
    pub capacity_utilization: f64,
    pub distance_utilization: Option<f64>,
    pub duration_utilization: Option<f64>,
}

impl ValidationResult {
    pub fn new() -> Self {
        Self {
            is_valid: true,
            violations: Vec::new(),
            capacity_utilization: 0.0,
            distance_utilization: None,
            duration_utilization: None,
        }
    }

    pub fn add_violation(&mut self, violation: String) {
        self.is_valid = false;
        self.violations.push(violation);
    }
}

impl Default for ValidationResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Route validator with configurable constraints
#[derive(Debug, Clone)]
pub struct RouteValidator {
    pub check_capacity: bool,
    pub check_time_windows: bool,
    pub check_distance_limits: bool,
    pub check_duration_limits: bool,
}

impl Default for RouteValidator {
    fn default() -> Self {
        Self {
            check_capacity: true,
            check_time_windows: true,
            check_distance_limits: true,
            check_duration_limits: true,
        }
    }
}

impl RouteValidator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_capacity_check(mut self, check: bool) -> Self {
        self.check_capacity = check;
        self
    }

    pub fn with_time_window_check(mut self, check: bool) -> Self {
        self.check_time_windows = check;
        self
    }

    pub fn with_distance_limit_check(mut self, check: bool) -> Self {
        self.check_distance_limits = check;
        self
    }

    pub fn with_duration_limit_check(mut self, check: bool) -> Self {
        self.check_duration_limits = check;
        self
    }

    /// Validate a single route against all configured constraints
    pub fn validate_route(
        &self,
        instance: &VrpInstance,
        route: &Route,
    ) -> VrpResult<ValidationResult> {
        let mut result = ValidationResult::new();
        
        let vehicle = instance.get_vehicle(route.vehicle_id)
            .ok_or_else(|| VrpError::InvalidInput(format!("Vehicle {} not found", route.vehicle_id)))?;

        let depot_idx = instance.locations
            .iter()
            .position(|loc| loc.id == vehicle.depot_id)
            .ok_or_else(|| VrpError::InvalidInput(format!("Depot {} not found", vehicle.depot_id)))?;

        // Get route location indices
        let route_indices: Vec<usize> = route.locations
            .iter()
            .filter_map(|&id| instance.locations.iter().position(|loc| loc.id == id))
            .collect();

        if route_indices.len() != route.locations.len() {
            result.add_violation("Some locations in route not found in instance".to_string());
            return Ok(result);
        }

        // Validate capacity constraints
        if self.check_capacity {
            self.validate_capacity_constraint(instance, route, vehicle.capacity, &mut result);
        }

        // Validate time window constraints
        if self.check_time_windows {
            self.validate_time_windows(instance, &route_indices, depot_idx, &mut result)?;
        }

        // Validate distance limits
        if self.check_distance_limits {
            if let Some(max_distance) = vehicle.max_distance {
                self.validate_distance_limit(instance, &route_indices, depot_idx, max_distance, &mut result);
            }
        }

        // Validate duration limits
        if self.check_duration_limits {
            if let Some(max_duration) = vehicle.max_duration {
                self.validate_duration_limit(instance, &route_indices, depot_idx, max_duration, &mut result)?;
            }
        }

        // Calculate utilization metrics
        result.capacity_utilization = route.total_demand / vehicle.capacity;
        
        if let Some(max_distance) = vehicle.max_distance {
            let actual_distance = calculate_route_distance(instance, &route_indices, depot_idx);
            result.distance_utilization = Some(actual_distance / max_distance);
        }
        
        if let Some(max_duration) = vehicle.max_duration {
            if let Some(actual_duration) = calculate_route_duration(instance, &route_indices, depot_idx) {
                result.duration_utilization = Some(actual_duration / max_duration);
            }
        }

        Ok(result)
    }

    /// Validate capacity constraints for a route
    fn validate_capacity_constraint(
        &self,
        instance: &VrpInstance,
        route: &Route,
        vehicle_capacity: f64,
        result: &mut ValidationResult,
    ) {
        let mut total_demand = 0.0;
        
        for &location_id in &route.locations {
            if let Some(location) = instance.get_location(location_id) {
                total_demand += location.demand;
            }
        }

        if total_demand > vehicle_capacity {
            result.add_violation(format!(
                "Capacity violated: demand {} > capacity {}",
                total_demand, vehicle_capacity
            ));
        }

        result.capacity_utilization = total_demand / vehicle_capacity;
    }

    /// Validate time window constraints for a route
    fn validate_time_windows(
        &self,
        instance: &VrpInstance,
        route_indices: &[usize],
        depot_idx: usize,
        result: &mut ValidationResult,
    ) -> VrpResult<()> {
        if route_indices.is_empty() {
            return Ok(());
        }

        let time_matrix = instance.time_matrix.as_ref();
        if time_matrix.is_none() {
            // Cannot validate time windows without time matrix
            return Ok(());
        }
        let time_matrix = time_matrix.unwrap();

        let mut current_time = 0.0;
        let mut current_idx = depot_idx;

        for &location_idx in route_indices {
            let location = &instance.locations[location_idx];
            
            // Add travel time
            current_time += time_matrix[current_idx][location_idx];
            
            // Check time window constraint
            if let Some(time_window) = location.time_window {
                if current_time < time_window.start {
                    // Arrive early, wait until window opens
                    current_time = time_window.start;
                } else if current_time > time_window.end {
                    // Arrive late, violation
                    result.add_violation(format!(
                        "Time window violated at location {}: arrival {} > window end {}",
                        location.id, current_time, time_window.end
                    ));
                }
            }
            
            // Add service time
            current_time += location.service_time;
            current_idx = location_idx;
        }

        Ok(())
    }

    /// Validate distance limit constraints for a route
    fn validate_distance_limit(
        &self,
        instance: &VrpInstance,
        route_indices: &[usize],
        depot_idx: usize,
        max_distance: f64,
        result: &mut ValidationResult,
    ) {
        let actual_distance = calculate_route_distance(instance, route_indices, depot_idx);
        
        if actual_distance > max_distance {
            result.add_violation(format!(
                "Distance limit violated: {} > {}",
                actual_distance, max_distance
            ));
        }
    }

    /// Validate duration limit constraints for a route
    fn validate_duration_limit(
        &self,
        instance: &VrpInstance,
        route_indices: &[usize],
        depot_idx: usize,
        max_duration: f64,
        result: &mut ValidationResult,
    ) -> VrpResult<()> {
        if let Some(actual_duration) = calculate_route_duration(instance, route_indices, depot_idx) {
            if actual_duration > max_duration {
                result.add_violation(format!(
                    "Duration limit violated: {} > {}",
                    actual_duration, max_duration
                ));
            }
        }
        Ok(())
    }

    /// Validate an entire solution
    pub fn validate_solution(
        &self,
        instance: &VrpInstance,
        solution: &Solution,
    ) -> VrpResult<Vec<ValidationResult>> {
        let mut results = Vec::new();
        
        for route in &solution.routes {
            let route_result = self.validate_route(instance, route)?;
            results.push(route_result);
        }

        Ok(results)
    }

    /// Check if all customers are served exactly once
    pub fn validate_customer_coverage(
        &self,
        instance: &VrpInstance,
        solution: &Solution,
    ) -> VrpResult<ValidationResult> {
        let mut result = ValidationResult::new();
        
        // Get all customers (locations with demand > 0)
        let all_customers: std::collections::HashSet<usize> = instance.locations
            .iter()
            .filter(|loc| loc.demand > 0.0)
            .map(|loc| loc.id)
            .collect();

        // Get all served customers
        let mut served_customers = std::collections::HashSet::new();
        let mut customer_count = std::collections::HashMap::new();

        for route in &solution.routes {
            for &customer_id in &route.locations {
                if instance.get_location(customer_id).map_or(false, |loc| loc.demand > 0.0) {
                    served_customers.insert(customer_id);
                    *customer_count.entry(customer_id).or_insert(0) += 1;
                }
            }
        }

        // Check for unserved customers
        let unserved: Vec<usize> = all_customers.difference(&served_customers).copied().collect();
        if !unserved.is_empty() {
            result.add_violation(format!("Unserved customers: {:?}", unserved));
        }

        // Check for customers served multiple times
        let multiple_service: Vec<(usize, usize)> = customer_count
            .iter()
            .filter(|(_, &count)| count > 1)
            .map(|(&id, &count)| (id, count))
            .collect();
        
        if !multiple_service.is_empty() {
            result.add_violation(format!("Customers served multiple times: {:?}", multiple_service));
        }

        Ok(result)
    }
}

/// Convenience function to validate a solution with default settings
pub fn validate_solution(instance: &VrpInstance, solution: &Solution) -> VrpResult<bool> {
    let validator = RouteValidator::new();
    
    // Validate each route
    let route_results = validator.validate_solution(instance, solution)?;
    let routes_valid = route_results.iter().all(|result| result.is_valid);
    
    // Validate customer coverage
    let coverage_result = validator.validate_customer_coverage(instance, solution)?;
    
    Ok(routes_valid && coverage_result.is_valid)
}

/// Convenience function to get detailed validation report
pub fn get_validation_report(
    instance: &VrpInstance,
    solution: &Solution,
) -> VrpResult<String> {
    let validator = RouteValidator::new();
    let mut report = String::new();
    
    report.push_str(&format!("Validation Report for Solution with {} routes:\n", solution.routes.len()));
    report.push_str(&format!("Total Distance: {:.2}\n", solution.total_distance));
    report.push_str(&format!("Total Duration: {:.2}\n", solution.total_duration));
    report.push_str(&format!("Vehicles Used: {}\n\n", solution.num_vehicles_used));

    // Validate each route
    let route_results = validator.validate_solution(instance, solution)?;
    
    for (i, (route, validation)) in solution.routes.iter().zip(route_results.iter()).enumerate() {
        report.push_str(&format!("Route {} (Vehicle {}):\n", i + 1, route.vehicle_id));
        report.push_str(&format!("  Locations: {:?}\n", route.locations));
        report.push_str(&format!("  Distance: {:.2}\n", route.total_distance));
        report.push_str(&format!("  Duration: {:.2}\n", route.total_duration));
        report.push_str(&format!("  Demand: {:.2}\n", route.total_demand));
        report.push_str(&format!("  Valid: {}\n", validation.is_valid));
        report.push_str(&format!("  Capacity Utilization: {:.2}%\n", validation.capacity_utilization * 100.0));
        
        if let Some(dist_util) = validation.distance_utilization {
            report.push_str(&format!("  Distance Utilization: {:.2}%\n", dist_util * 100.0));
        }
        
        if let Some(dur_util) = validation.duration_utilization {
            report.push_str(&format!("  Duration Utilization: {:.2}%\n", dur_util * 100.0));
        }
        
        if !validation.violations.is_empty() {
            report.push_str("  Violations:\n");
            for violation in &validation.violations {
                report.push_str(&format!("    - {}\n", violation));
            }
        }
        
        report.push('\n');
    }

    // Validate customer coverage
    let coverage_result = validator.validate_customer_coverage(instance, solution)?;
    report.push_str("Customer Coverage:\n");
    report.push_str(&format!("  Valid: {}\n", coverage_result.is_valid));
    
    if !coverage_result.violations.is_empty() {
        report.push_str("  Coverage Issues:\n");
        for violation in &coverage_result.violations {
            report.push_str(&format!("    - {}\n", violation));
        }
    }

    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::distance::{calculate_distance_matrix, DistanceMethod};
    use crate::types::*;

    fn create_test_instance() -> VrpInstance {
        let locations = vec![
            Location::depot(0, "Depot".to_string(), Coordinate::new(0.0, 0.0)),
            Location::new(1, "Customer 1".to_string(), Coordinate::new(1.0, 1.0), 10.0, 
                Some(TimeWindow::new(0.0, 100.0)), 5.0),
            Location::new(2, "Customer 2".to_string(), Coordinate::new(2.0, 2.0), 15.0, 
                Some(TimeWindow::new(0.0, 50.0)), 5.0),
        ];

        let vehicles = vec![
            Vehicle::new(0, 30.0, Some(1000.0), Some(200.0), 0),
        ];

        let mut instance = VrpInstance::new(locations, vehicles);
        calculate_distance_matrix(&mut instance, DistanceMethod::Euclidean);
        instance
    }

    #[test]
    fn test_valid_route() {
        let instance = create_test_instance();
        let validator = RouteValidator::new().with_distance_limit_check(false).with_duration_limit_check(false);
        
        let mut route = Route::new(0);
        route.add_location(1);
        route.total_demand = 10.0;
        route.total_distance = 100.0;
        route.total_duration = 50.0;

        let result = validator.validate_route(&instance, &route).unwrap();
        println!("Violations: {:?}", result.violations);
        assert!(result.is_valid);
        assert!(result.violations.is_empty());
    }

    #[test]
    fn test_capacity_violation() {
        let instance = create_test_instance();
        let validator = RouteValidator::new();
        
        let mut route = Route::new(0);
        route.add_location(1);
        route.add_location(2);
        route.total_demand = 40.0; // Exceeds vehicle capacity of 30
        route.total_distance = 100.0;

        let result = validator.validate_route(&instance, &route).unwrap();
        assert!(!result.is_valid);
        assert!(!result.violations.is_empty());
    }

    #[test]
    fn test_solution_validation() {
        let instance = create_test_instance();
        
        // Use a custom validator that ignores distance and duration limits
        let validator = RouteValidator::new()
            .with_distance_limit_check(false)
            .with_duration_limit_check(false);
        
        let mut route1 = Route::new(0);
        route1.add_location(1);
        route1.total_demand = 10.0;
        
        let mut route2 = Route::new(0);
        route2.add_location(2);
        route2.total_demand = 15.0;
        
        let mut solution = Solution::new();
        solution.add_route(route1);
        solution.add_route(route2);
        
        // Validate each route
        let route_results = validator.validate_solution(&instance, &solution).unwrap();
        let routes_valid = route_results.iter().all(|result| result.is_valid);
        
        // Validate customer coverage
        let coverage_result = validator.validate_customer_coverage(&instance, &solution).unwrap();
        
        let is_valid = routes_valid && coverage_result.is_valid;
        assert!(is_valid);
    }
}
