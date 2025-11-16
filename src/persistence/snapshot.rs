use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Serializable database state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseSnapshot {
    pub version: String,
    pub dimension: usize,
    pub metric: String,
    pub index_type: String,
    pub vectors: Vec<VectorEntry>,
    pub metadata: HashMap<u64, HashMap<String, String>>,
    pub index_params: IndexParams,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorEntry {
    pub id: u64,
    pub vector: Vec<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexParams {
    pub hnsw_m: Option<usize>,
    pub hnsw_ef_construction: Option<usize>,
}

impl DatabaseSnapshot {
    pub fn new(
        dimension: usize,
        metric: String,
        index_type: String,
        index_params: IndexParams,
    ) -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            dimension,
            metric,
            index_type,
            vectors: Vec::new(),
            metadata: HashMap::new(),
            index_params,
        }
    }

    pub fn add_vector(&mut self, id: u64, vector: Vec<f32>, metadata: HashMap<String, String>) {
        self.vectors.push(VectorEntry { id, vector });
        if !metadata.is_empty() {
            self.metadata.insert(id, metadata);
        }
    }

    /// Serialize to binary format using bincode
    pub fn to_binary(&self) -> Result<Vec<u8>, String> {
        bincode::serialize(self).map_err(|e| format!("Serialization error: {}", e))
    }

    /// Deserialize from binary format
    pub fn from_binary(data: &[u8]) -> Result<Self, String> {
        bincode::deserialize(data).map_err(|e| format!("Deserialization error: {}", e))
    }

    /// Serialize to JSON (less efficient but human-readable)
    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string(self).map_err(|e| format!("JSON serialization error: {}", e))
    }

    /// Deserialize from JSON
    pub fn from_json(json: &str) -> Result<Self, String> {
        serde_json::from_str(json).map_err(|e| format!("JSON deserialization error: {}", e))
    }

    /// Get size estimate in bytes
    pub fn size_estimate(&self) -> usize {
        let vector_size = self.vectors.len() * (8 + self.dimension * 4); // id + floats
        let metadata_size: usize = self
            .metadata
            .values()
            .map(|m| m.iter().map(|(k, v)| k.len() + v.len()).sum::<usize>())
            .sum();
        vector_size + metadata_size + 1024 // + overhead
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snapshot_serialization() {
        let mut snapshot = DatabaseSnapshot::new(
            128,
            "Cosine".to_string(),
            "HNSW".to_string(),
            IndexParams {
                hnsw_m: Some(16),
                hnsw_ef_construction: Some(200),
            },
        );

        snapshot.add_vector(1, vec![1.0; 128], HashMap::new());
        snapshot.add_vector(2, vec![2.0; 128], HashMap::new());

        // Binary serialization
        let binary = snapshot.to_binary().unwrap();
        let restored = DatabaseSnapshot::from_binary(&binary).unwrap();

        assert_eq!(snapshot.version, restored.version);
        assert_eq!(snapshot.dimension, restored.dimension);
        assert_eq!(snapshot.vectors.len(), restored.vectors.len());

        // JSON serialization
        let json = snapshot.to_json().unwrap();
        let restored_json = DatabaseSnapshot::from_json(&json).unwrap();

        assert_eq!(snapshot.dimension, restored_json.dimension);
    }

    #[test]
    fn test_size_estimate() {
        let snapshot = DatabaseSnapshot::new(
            128,
            "Cosine".to_string(),
            "Flat".to_string(),
            IndexParams {
                hnsw_m: None,
                hnsw_ef_construction: None,
            },
        );

        let size = snapshot.size_estimate();
        assert!(size > 0);
    }
}
