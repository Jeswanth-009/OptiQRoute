use osmpbfreader::{OsmPbfReader, OsmObj};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use geojson::{GeoJson, Geometry, Value, Feature, FeatureCollection};
use serde_json::Map;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsmNode {
    pub id: i64,
    pub lat: f64,
    pub lon: f64,
    pub tags: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsmWay {
    pub id: i64,
    pub nodes: Vec<i64>,
    pub tags: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsmData {
    pub nodes: HashMap<i64, OsmNode>,
    pub ways: HashMap<i64, OsmWay>,
}

pub struct OsmParser {
    pub data: OsmData,
}

impl OsmParser {
    pub fn new() -> Self {
        Self {
            data: OsmData {
                nodes: HashMap::new(),
                ways: HashMap::new(),
            }
        }
    }

    pub fn parse_pbf_file(&mut self, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        println!("Opening PBF file: {}", file_path);
        let file = File::open(file_path)?;
        let buf_reader = BufReader::new(file);
        let mut pbf_reader = OsmPbfReader::new(buf_reader);

        let mut node_count = 0;
        let mut way_count = 0;

        for obj in pbf_reader.iter().map(|o| o.unwrap()) {
            match obj {
                OsmObj::Node(node) => {
                    node_count += 1;
                    if node_count % 100000 == 0 {
                        println!("Processed {} nodes", node_count);
                    }

                    let osm_node = OsmNode {
                        id: node.id.0,
                        lat: node.lat(),
                        lon: node.lon(),
                        tags: node.tags.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect(),
                    };
                    self.data.nodes.insert(node.id.0, osm_node);
                }
                OsmObj::Way(way) => {
                    way_count += 1;
                    if way_count % 10000 == 0 {
                        println!("Processed {} ways", way_count);
                    }

                    let osm_way = OsmWay {
                        id: way.id.0,
                        nodes: way.nodes.iter().map(|n| n.0).collect(),
                        tags: way.tags.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect(),
                    };
                    self.data.ways.insert(way.id.0, osm_way);
                }
                OsmObj::Relation(_) => {
                    // Skip relations for now, focus on nodes and ways
                    continue;
                }
            }
        }

        println!("Finished parsing PBF file:");
        println!("  - Nodes: {}", node_count);
        println!("  - Ways: {}", way_count);

        Ok(())
    }

    pub fn export_to_json(&self, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        println!("Exporting to JSON: {}", file_path);
        let json_str = serde_json::to_string_pretty(&self.data)?;
        std::fs::write(file_path, json_str)?;
        println!("JSON export completed");
        Ok(())
    }

    pub fn export_to_geojson(&self, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        println!("Exporting to GeoJSON: {}", file_path);
        
        let mut features = Vec::new();

        // Add nodes as Point features
        for (_, node) in &self.data.nodes {
            let geometry = Geometry::new(Value::Point(vec![node.lon, node.lat]));
            
            let mut properties = Map::new();
            properties.insert("id".to_string(), serde_json::Value::Number(node.id.into()));
            properties.insert("type".to_string(), serde_json::Value::String("node".to_string()));
            
            // Add all tags as properties
            for (key, value) in &node.tags {
                properties.insert(key.clone(), serde_json::Value::String(value.clone()));
            }

            let feature = Feature {
                bbox: None,
                geometry: Some(geometry),
                id: None,
                properties: Some(properties),
                foreign_members: None,
            };

            features.push(feature);
        }

        // Add ways as LineString features
        for (_, way) in &self.data.ways {
            let mut coordinates = Vec::new();
            let mut valid_way = true;

            // Get coordinates for all nodes in the way
            for node_id in &way.nodes {
                if let Some(node) = self.data.nodes.get(node_id) {
                    coordinates.push(vec![node.lon, node.lat]);
                } else {
                    // If we can't find a node, skip this way
                    valid_way = false;
                    break;
                }
            }

            if valid_way && coordinates.len() >= 2 {
                let geometry = Geometry::new(Value::LineString(coordinates));
                
                let mut properties = Map::new();
                properties.insert("id".to_string(), serde_json::Value::Number(way.id.into()));
                properties.insert("type".to_string(), serde_json::Value::String("way".to_string()));
                
                // Add all tags as properties
                for (key, value) in &way.tags {
                    properties.insert(key.clone(), serde_json::Value::String(value.clone()));
                }

                let feature = Feature {
                    bbox: None,
                    geometry: Some(geometry),
                    id: None,
                    properties: Some(properties),
                    foreign_members: None,
                };

                features.push(feature);
            }
        }

        let feature_collection = FeatureCollection {
            bbox: None,
            features,
            foreign_members: None,
        };

        let geojson = GeoJson::FeatureCollection(feature_collection);
        let geojson_str = serde_json::to_string_pretty(&geojson)?;
        std::fs::write(file_path, geojson_str)?;
        
        println!("GeoJSON export completed");
        Ok(())
    }

    pub fn filter_roads_only(&mut self) {
        // Keep only ways that are roads/highways
        self.data.ways.retain(|_, way| {
            way.tags.contains_key("highway")
        });

        // Keep only nodes that are referenced by remaining ways
        let mut referenced_nodes = std::collections::HashSet::new();
        for way in self.data.ways.values() {
            for node_id in &way.nodes {
                referenced_nodes.insert(*node_id);
            }
        }

        self.data.nodes.retain(|id, _| referenced_nodes.contains(id));
        
        println!("Filtered to roads only:");
        println!("  - Nodes: {}", self.data.nodes.len());
        println!("  - Ways: {}", self.data.ways.len());
    }

    pub fn get_node_coordinates(&self, node_id: i64) -> Option<(f64, f64)> {
        self.data.nodes.get(&node_id).map(|node| (node.lat, node.lon))
    }

    pub fn find_nearest_node(&self, lat: f64, lon: f64) -> Option<(i64, f64)> {
        let mut nearest_node = None;
        let mut min_distance = f64::MAX;

        for (id, node) in &self.data.nodes {
            let distance = self.haversine_distance(lat, lon, node.lat, node.lon);
            if distance < min_distance {
                min_distance = distance;
                nearest_node = Some((*id, distance));
            }
        }

        nearest_node
    }

    fn haversine_distance(&self, lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
        let r = 6371000.0; // Earth's radius in meters
        let phi1 = lat1.to_radians();
        let phi2 = lat2.to_radians();
        let delta_phi = (lat2 - lat1).to_radians();
        let delta_lambda = (lon2 - lon1).to_radians();

        let a = (delta_phi / 2.0).sin().powi(2) +
                phi1.cos() * phi2.cos() *
                (delta_lambda / 2.0).sin().powi(2);
        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

        r * c
    }
}
