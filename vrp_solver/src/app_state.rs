//! Application state management for the VRP web server

use crate::api_types::*;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use uuid::Uuid;

/// Thread-safe application state
#[derive(Debug, Clone)]
pub struct AppState {
    pub graphs: Arc<RwLock<HashMap<Uuid, StoredGraph>>>,
    pub mappings: Arc<RwLock<HashMap<Uuid, StoredMapping>>>,
    pub vrp_instances: Arc<RwLock<HashMap<Uuid, StoredVrpInstance>>>,
    pub solutions: Arc<RwLock<HashMap<Uuid, StoredSolution>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            graphs: Arc::new(RwLock::new(HashMap::new())),
            mappings: Arc::new(RwLock::new(HashMap::new())),
            vrp_instances: Arc::new(RwLock::new(HashMap::new())),
            solutions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    // Graph operations
    pub fn store_graph(&self, graph: StoredGraph) -> Result<Uuid, String> {
        let graph_id = graph.id;
        match self.graphs.write() {
            Ok(mut graphs) => {
                graphs.insert(graph_id, graph);
                Ok(graph_id)
            }
            Err(_) => Err("Failed to acquire write lock for graphs".to_string()),
        }
    }

    pub fn get_graph(&self, graph_id: &Uuid) -> Result<Option<StoredGraph>, String> {
        match self.graphs.read() {
            Ok(graphs) => Ok(graphs.get(graph_id).cloned()),
            Err(_) => Err("Failed to acquire read lock for graphs".to_string()),
        }
    }

    pub fn list_graphs(&self) -> Result<Vec<Uuid>, String> {
        match self.graphs.read() {
            Ok(graphs) => Ok(graphs.keys().cloned().collect()),
            Err(_) => Err("Failed to acquire read lock for graphs".to_string()),
        }
    }

    // Mapping operations
    pub fn store_mapping(&self, mapping: StoredMapping) -> Result<(), String> {
        match self.mappings.write() {
            Ok(mut mappings) => {
                mappings.insert(mapping.graph_id, mapping);
                Ok(())
            }
            Err(_) => Err("Failed to acquire write lock for mappings".to_string()),
        }
    }

    pub fn get_mapping(&self, graph_id: &Uuid) -> Result<Option<StoredMapping>, String> {
        match self.mappings.read() {
            Ok(mappings) => Ok(mappings.get(graph_id).cloned()),
            Err(_) => Err("Failed to acquire read lock for mappings".to_string()),
        }
    }

    // VRP instance operations
    pub fn store_vrp_instance(&self, instance: StoredVrpInstance) -> Result<Uuid, String> {
        let instance_id = instance.id;
        match self.vrp_instances.write() {
            Ok(mut instances) => {
                instances.insert(instance_id, instance);
                Ok(instance_id)
            }
            Err(_) => Err("Failed to acquire write lock for VRP instances".to_string()),
        }
    }

    pub fn get_vrp_instance(&self, vrp_id: &Uuid) -> Result<Option<StoredVrpInstance>, String> {
        match self.vrp_instances.read() {
            Ok(instances) => Ok(instances.get(vrp_id).cloned()),
            Err(_) => Err("Failed to acquire read lock for VRP instances".to_string()),
        }
    }

    pub fn list_vrp_instances(&self) -> Result<Vec<Uuid>, String> {
        match self.vrp_instances.read() {
            Ok(instances) => Ok(instances.keys().cloned().collect()),
            Err(_) => Err("Failed to acquire read lock for VRP instances".to_string()),
        }
    }

    // Solution operations
    pub fn store_solution(&self, solution: StoredSolution) -> Result<Uuid, String> {
        let solution_id = solution.id;
        match self.solutions.write() {
            Ok(mut solutions) => {
                solutions.insert(solution_id, solution);
                Ok(solution_id)
            }
            Err(_) => Err("Failed to acquire write lock for solutions".to_string()),
        }
    }

    pub fn get_solution(&self, solution_id: &Uuid) -> Result<Option<StoredSolution>, String> {
        match self.solutions.read() {
            Ok(solutions) => Ok(solutions.get(solution_id).cloned()),
            Err(_) => Err("Failed to acquire read lock for solutions".to_string()),
        }
    }

    pub fn list_solutions(&self) -> Result<Vec<Uuid>, String> {
        match self.solutions.read() {
            Ok(solutions) => Ok(solutions.keys().cloned().collect()),
            Err(_) => Err("Failed to acquire read lock for solutions".to_string()),
        }
    }

    pub fn get_solutions_for_vrp(&self, vrp_id: &Uuid) -> Result<Vec<StoredSolution>, String> {
        match self.solutions.read() {
            Ok(solutions) => Ok(solutions
                .values()
                .filter(|s| &s.vrp_id == vrp_id)
                .cloned()
                .collect()),
            Err(_) => Err("Failed to acquire read lock for solutions".to_string()),
        }
    }

    // Cleanup operations (for memory management)
    pub fn cleanup_old_data(&self, max_age_hours: u64) -> Result<(), String> {
        use std::time::{SystemTime, Duration};

        let cutoff_time = SystemTime::now()
            .checked_sub(Duration::from_secs(max_age_hours * 3600))
            .unwrap_or(SystemTime::UNIX_EPOCH);

        // Clean old graphs
        if let Ok(mut graphs) = self.graphs.write() {
            graphs.retain(|_, graph| graph.created_at > cutoff_time);
        }

        // Clean old mappings
        if let Ok(mut mappings) = self.mappings.write() {
            mappings.retain(|_, mapping| mapping.created_at > cutoff_time);
        }

        // Clean old VRP instances
        if let Ok(mut instances) = self.vrp_instances.write() {
            instances.retain(|_, instance| instance.created_at > cutoff_time);
        }

        // Clean old solutions
        if let Ok(mut solutions) = self.solutions.write() {
            solutions.retain(|_, solution| solution.created_at > cutoff_time);
        }

        Ok(())
    }

    // Statistics
    pub fn get_stats(&self) -> Result<AppStateStats, String> {
        let graph_count = self.graphs.read()
            .map_err(|_| "Failed to acquire read lock for graphs")?
            .len();

        let mapping_count = self.mappings.read()
            .map_err(|_| "Failed to acquire read lock for mappings")?
            .len();

        let vrp_count = self.vrp_instances.read()
            .map_err(|_| "Failed to acquire read lock for VRP instances")?
            .len();

        let solution_count = self.solutions.read()
            .map_err(|_| "Failed to acquire read lock for solutions")?
            .len();

        Ok(AppStateStats {
            graphs: graph_count,
            mappings: mapping_count,
            vrp_instances: vrp_count,
            solutions: solution_count,
        })
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, serde::Serialize)]
pub struct AppStateStats {
    pub graphs: usize,
    pub mappings: usize,
    pub vrp_instances: usize,
    pub solutions: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::osm_parser::OsmData;
    use std::collections::HashMap;

    #[test]
    fn test_app_state_operations() {
        let state = AppState::new();
        let graph_id = Uuid::new_v4();
        
        // Test storing and retrieving a graph
        let graph = StoredGraph {
            id: graph_id,
            osm_data: OsmData {
                nodes: HashMap::new(),
                ways: HashMap::new(),
            },
            created_at: std::time::SystemTime::now(),
            node_count: 0,
            way_count: 0,
        };

        // Store graph
        let stored_id = state.store_graph(graph.clone()).unwrap();
        assert_eq!(stored_id, graph_id);

        // Retrieve graph
        let retrieved = state.get_graph(&graph_id).unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, graph_id);

        // Test stats
        let stats = state.get_stats().unwrap();
        assert_eq!(stats.graphs, 1);
    }
}
