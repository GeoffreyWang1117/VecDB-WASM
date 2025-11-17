use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Vector type alias
pub type Vector = Vec<f32>;

/// Metadata associated with vectors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorMetadata {
    pub id: u64,
    pub data: HashMap<String, String>,
}

impl VectorMetadata {
    pub fn new(id: u64) -> Self {
        Self {
            id,
            data: HashMap::new(),
        }
    }

    pub fn with_data(id: u64, data: HashMap<String, String>) -> Self {
        Self { id, data }
    }

    pub fn insert(&mut self, key: String, value: String) {
        self.data.insert(key, value);
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.data.get(key)
    }
}

/// Storage for vectors and metadata
#[derive(Debug, Clone)]
pub struct VectorStorage {
    vectors: HashMap<u64, Vector>,
    metadata: HashMap<u64, VectorMetadata>,
    dimension: usize,
}

impl VectorStorage {
    pub fn new(dimension: usize) -> Self {
        Self {
            vectors: HashMap::new(),
            metadata: HashMap::new(),
            dimension,
        }
    }

    pub fn insert(
        &mut self,
        id: u64,
        vector: Vector,
        metadata: VectorMetadata,
    ) -> Result<(), String> {
        if vector.len() != self.dimension {
            return Err(format!(
                "Vector dimension mismatch: expected {}, got {}",
                self.dimension,
                vector.len()
            ));
        }

        self.vectors.insert(id, vector);
        self.metadata.insert(id, metadata);
        Ok(())
    }

    pub fn get_vector(&self, id: u64) -> Option<&Vector> {
        self.vectors.get(&id)
    }

    pub fn get_metadata(&self, id: u64) -> Option<&VectorMetadata> {
        self.metadata.get(&id)
    }

    pub fn remove(&mut self, id: u64) -> bool {
        let v = self.vectors.remove(&id).is_some();
        let m = self.metadata.remove(&id).is_some();
        v || m
    }

    pub fn len(&self) -> usize {
        self.vectors.len()
    }

    pub fn is_empty(&self) -> bool {
        self.vectors.is_empty()
    }

    pub fn dimension(&self) -> usize {
        self.dimension
    }

    pub fn iter(&self) -> impl Iterator<Item = (u64, &Vector)> {
        self.vectors.iter().map(|(k, v)| (*k, v))
    }

    pub fn ids(&self) -> Vec<u64> {
        self.vectors.keys().copied().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_storage() {
        let mut storage = VectorStorage::new(3);
        let vec = vec![1.0, 2.0, 3.0];
        let meta = VectorMetadata::new(1);

        assert!(storage.insert(1, vec.clone(), meta).is_ok());
        assert_eq!(storage.len(), 1);
        assert_eq!(storage.get_vector(1), Some(&vec));
    }

    #[test]
    fn test_dimension_validation() {
        let mut storage = VectorStorage::new(3);
        let vec = vec![1.0, 2.0];
        let meta = VectorMetadata::new(1);

        assert!(storage.insert(1, vec, meta).is_err());
    }
}
