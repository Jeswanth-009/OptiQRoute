//! Vehicle Routing Problem (VRP) Solver
//!
//! This library provides algorithms and tools for solving vehicle routing problems
//! with parallel processing capabilities using Rayon.

pub mod types;
pub mod distance;
pub mod solver;
pub mod validate;
pub mod utils;
pub mod osm_parser;

pub use types::*;
pub use distance::*;
pub use solver::*;
pub use validate::*;
pub use utils::*;

/// Result type for VRP operations
pub type VrpResult<T> = Result<T, VrpError>;

/// Main errors that can occur in VRP solving
#[derive(thiserror::Error, Debug)]
pub enum VrpError {
    #[error("Invalid route: {0}")]
    InvalidRoute(String),
    
    #[error("Capacity constraint violated: required {required}, available {available}")]
    CapacityViolation { required: f64, available: f64 },
    
    #[error("Time window violation: arrival {arrival}, window [{start}, {end}]")]
    TimeWindowViolation { arrival: f64, start: f64, end: f64 },
    
    #[error("Distance limit exceeded: {distance} > {limit}")]
    DistanceLimitExceeded { distance: f64, limit: f64 },
    
    #[error("No solution found")]
    NoSolutionFound,
    
    #[error("Invalid input: {0}")]
    InvalidInput(String),
}
