//! API request and response types for the VRP web server

use crate::types::*;
use crate::osm_parser::OsmData;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Helper module for SystemTime serialization
mod timestamp_serde {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::time::{SystemTime, UNIX_EPOCH};

    pub fn serialize<S>(time: &SystemTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let timestamp = time
            .duration_since(UNIX_EPOCH)
            .map_err(|_| serde::ser::Error::custom("SystemTime before Unix epoch"))?
            .as_secs();
        serializer.serialize_u64(timestamp)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<SystemTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let timestamp = u64::deserialize(deserializer)?;
        Ok(UNIX_EPOCH + std::time::Duration::from_secs(timestamp))
    }
}

// OSM Upload API Types
#[derive(Debug, Deserialize)]
pub struct OsmUploadRequest {
    pub file_url: Option<String>,
    // File upload will be handled via multipart form data
}

#[derive(Debug, Serialize)]
pub struct OsmUploadResponse {
    pub graph_id: Uuid,
    pub nodes: usize,
    pub edges: usize,
    pub message: String,
}

// Depot/Customer Mapping API Types
#[derive(Debug, Deserialize)]
pub struct MapLocationRequest {
    pub graph_id: Uuid,
    pub depot: LocationCoordinate,
    pub customers: Vec<LocationCoordinate>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LocationCoordinate {
    pub lat: f64,
    pub lon: f64,
    pub name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct MapLocationResponse {
    pub mapped_depot: MappedLocation,
    pub mapped_customers: Vec<MappedLocation>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MappedLocation {
    pub node_id: i64,
    pub lat: f64,
    pub lon: f64,
    pub distance_to_original: f64,
}

// VRP Instance Generation API Types
#[derive(Debug, Deserialize)]
pub struct GenerateVrpRequest {
    pub graph_id: Uuid,
    pub vehicles: usize,
    pub capacity: f64,
    pub constraints: VrpConstraints,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VrpConstraints {
    pub time_windows: bool,
    pub max_distance: Option<f64>,
    pub max_duration: Option<f64>,
    pub service_time: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct GenerateVrpResponse {
    pub vrp_id: Uuid,
    pub customers: usize,
    pub vehicles: usize,
    pub depot_count: usize,
}

// VRP Solving API Types
#[derive(Debug, Deserialize)]
pub struct SolveVrpRequest {
    pub vrp_id: Uuid,
    pub algorithm: SolverAlgorithm,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SolverAlgorithm {
    Greedy,
    GreedyFarthest,
    ClarkeWright,
    MultiStart,
}

#[derive(Debug, Serialize)]
pub struct SolveVrpResponse {
    pub solution_id: Uuid,
    pub routes: Vec<ApiRoute>,
    pub total_cost: f64,
    pub total_distance: f64,
    pub total_duration: f64,
    pub vehicles_used: usize,
    pub solve_time_ms: f64,
}

#[derive(Debug, Serialize)]
pub struct ApiRoute {
    pub vehicle_id: usize,
    pub path: Vec<usize>,
    pub distance: f64,
    pub duration: f64,
    pub demand: f64,
    pub locations: Vec<ApiLocation>,
}

#[derive(Debug, Serialize)]
pub struct ApiLocation {
    pub id: usize,
    pub name: String,
    pub lat: f64,
    pub lon: f64,
    pub demand: f64,
    pub service_time: f64,
}

// Solution Export API Types
#[derive(Debug, Deserialize)]
pub struct ExportFormat {
    pub format: Option<String>, // "geojson", "json", "csv"
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    pub details: Option<String>,
}

impl ErrorResponse {
    pub fn new(error: &str, message: &str) -> Self {
        Self {
            error: error.to_string(),
            message: message.to_string(),
            details: None,
        }
    }

    pub fn with_details(error: &str, message: &str, details: &str) -> Self {
        Self {
            error: error.to_string(),
            message: message.to_string(),
            details: Some(details.to_string()),
        }
    }
}

// Internal state structures
#[derive(Debug, Clone)]
pub struct StoredGraph {
    pub id: Uuid,
    pub osm_data: OsmData,
    pub created_at: std::time::SystemTime,
    pub node_count: usize,
    pub way_count: usize,
}

#[derive(Debug, Clone)]
pub struct StoredMapping {
    pub graph_id: Uuid,
    pub depot: MappedLocation,
    pub customers: Vec<MappedLocation>,
    pub created_at: std::time::SystemTime,
}

#[derive(Debug, Clone)]
pub struct StoredVrpInstance {
    pub id: Uuid,
    pub mapping: StoredMapping,
    pub instance: VrpInstance,
    pub constraints: VrpConstraints,
    pub created_at: std::time::SystemTime,
}

#[derive(Debug, Clone, Serialize)]
pub struct StoredSolution {
    pub id: Uuid,
    pub vrp_id: Uuid,
    pub solution: Solution,
    pub algorithm: SolverAlgorithm,
    pub solve_time_ms: f64,
    #[serde(with = "timestamp_serde")]
    pub created_at: std::time::SystemTime,
}

// Conversion implementations
impl From<Coordinate> for LocationCoordinate {
    fn from(coord: Coordinate) -> Self {
        Self {
            lat: coord.lat,
            lon: coord.lon,
            name: None,
        }
    }
}

impl From<&LocationCoordinate> for Coordinate {
    fn from(coord: &LocationCoordinate) -> Self {
        Coordinate::new(coord.lat, coord.lon)
    }
}

impl From<&Location> for ApiLocation {
    fn from(location: &Location) -> Self {
        Self {
            id: location.id,
            name: location.name.clone(),
            lat: location.coordinate.lat,
            lon: location.coordinate.lon,
            demand: location.demand,
            service_time: location.service_time,
        }
    }
}
