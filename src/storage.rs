use crate::integrity::IntegrityChecker;
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
    integrity_checker: IntegrityChecker,
    enable_integrity_check: bool,
}

impl VectorStorage {
    pub fn new(dimension: usize) -> Self {
        Self::with_integrity_check(dimension, true)
    }

    pub fn with_integrity_check(dimension: usize, enable: bool) -> Self {
        Self {
            vectors: HashMap::new(),
            metadata: HashMap::new(),
            dimension,
            integrity_checker: IntegrityChecker::new(),
            enable_integrity_check: enable,
        }
    }

    pub fn enable_integrity_check(&mut self, enable: bool) {
        self.enable_integrity_check = enable;
    }

    pub fn is_integrity_check_enabled(&self) -> bool {
        self.enable_integrity_check
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

        // Store checksum if enabled
        if self.enable_integrity_check {
            self.integrity_checker.store_checksum(id, &vector);
        }

        self.vectors.insert(id, vector);
        self.metadata.insert(id, metadata);
        Ok(())
    }

    pub fn get_vector(&self, id: u64) -> Option<&Vector> {
        self.vectors.get(&id)
    }

    pub fn get_vector_with_verification(&self, id: u64) -> Option<&Vector> {
        if let Some(vector) = self.vectors.get(&id) {
            if self.enable_integrity_check {
                if self.integrity_checker.verify(id, vector) {
                    Some(vector)
                } else {
                    // Corruption detected
                    None
                }
            } else {
                Some(vector)
            }
        } else {
            None
        }
    }

    pub fn verify_vector(&self, id: u64) -> bool {
        if let Some(vector) = self.vectors.get(&id) {
            self.integrity_checker.verify(id, vector)
        } else {
            false
        }
    }

    pub fn verify_all(&self) -> crate::integrity::IntegrityReport {
        self.integrity_checker.get_report(|id| self.vectors.get(&id).cloned())
    }

    pub fn get_metadata(&self, id: u64) -> Option<&VectorMetadata> {
        self.metadata.get(&id)
    }

    pub fn remove(&mut self, id: u64) -> bool {
        let v = self.vectors.remove(&id).is_some();
        let m = self.metadata.remove(&id).is_some();

        // Remove checksum
        if self.enable_integrity_check {
            self.integrity_checker.remove(id);
        }

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

    pub fn get_checksum(&self, id: u64) -> Option<u32> {
        self.integrity_checker.get_checksum(id)
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
