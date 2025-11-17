/// Data integrity module for vector database
///
/// Provides checksum calculation and verification to detect data corruption

use std::collections::HashMap;

/// Checksum calculator using XXHash-like algorithm
/// Optimized for speed while maintaining good collision resistance
#[derive(Clone, Debug)]
pub struct IntegrityChecker {
    checksums: HashMap<u64, u32>,
}

impl IntegrityChecker {
    /// Create a new integrity checker
    pub fn new() -> Self {
        Self {
            checksums: HashMap::new(),
        }
    }

    /// Calculate checksum for a vector
    pub fn calculate_checksum(vector: &[f32]) -> u32 {
        let mut hash: u32 = 2166136261; // FNV offset basis
        let fnv_prime: u32 = 16777619;

        // Convert f32 to bytes and hash
        for &value in vector {
            let bytes = value.to_le_bytes();
            for &byte in &bytes {
                hash = hash.wrapping_mul(fnv_prime);
                hash ^= byte as u32;
            }
        }

        hash
    }

    /// Store checksum for a vector
    pub fn store_checksum(&mut self, id: u64, vector: &[f32]) {
        let checksum = Self::calculate_checksum(vector);
        self.checksums.insert(id, checksum);
    }

    /// Verify vector integrity
    pub fn verify(&self, id: u64, vector: &[f32]) -> bool {
        if let Some(&stored_checksum) = self.checksums.get(&id) {
            let calculated_checksum = Self::calculate_checksum(vector);
            stored_checksum == calculated_checksum
        } else {
            false // No checksum found
        }
    }

    /// Remove checksum for a vector
    pub fn remove(&mut self, id: u64) -> bool {
        self.checksums.remove(&id).is_some()
    }

    /// Get stored checksum
    pub fn get_checksum(&self, id: u64) -> Option<u32> {
        self.checksums.get(&id).copied()
    }

    /// Clear all checksums
    pub fn clear(&mut self) {
        self.checksums.clear();
    }

    /// Get total number of stored checksums
    pub fn len(&self) -> usize {
        self.checksums.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.checksums.is_empty()
    }

    /// Verify all vectors in a collection
    pub fn verify_all<F>(&self, get_vector: F) -> HashMap<u64, bool>
    where
        F: Fn(u64) -> Option<Vec<f32>>,
    {
        let mut results = HashMap::new();
        for (&id, _) in &self.checksums {
            if let Some(vector) = get_vector(id) {
                results.insert(id, self.verify(id, &vector));
            } else {
                results.insert(id, false); // Vector not found
            }
        }
        results
    }

    /// Get integrity report
    pub fn get_report<F>(&self, get_vector: F) -> IntegrityReport
    where
        F: Fn(u64) -> Option<Vec<f32>>,
    {
        let mut valid = 0;
        let mut corrupted = Vec::new();
        let mut missing = Vec::new();

        for (&id, &stored_checksum) in &self.checksums {
            match get_vector(id) {
                Some(vector) => {
                    let calculated = Self::calculate_checksum(&vector);
                    if calculated == stored_checksum {
                        valid += 1;
                    } else {
                        corrupted.push(id);
                    }
                }
                None => {
                    missing.push(id);
                }
            }
        }

        IntegrityReport {
            total: self.checksums.len(),
            valid,
            corrupted,
            missing,
        }
    }
}

impl Default for IntegrityChecker {
    fn default() -> Self {
        Self::new()
    }
}

/// Integrity verification report
#[derive(Debug, Clone)]
pub struct IntegrityReport {
    pub total: usize,
    pub valid: usize,
    pub corrupted: Vec<u64>,
    pub missing: Vec<u64>,
}

impl IntegrityReport {
    /// Check if all vectors are valid
    pub fn is_healthy(&self) -> bool {
        self.corrupted.is_empty() && self.missing.is_empty()
    }

    /// Get corruption rate
    pub fn corruption_rate(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            (self.corrupted.len() + self.missing.len()) as f64 / self.total as f64
        }
    }

    /// Get summary string
    pub fn summary(&self) -> String {
        format!(
            "Total: {}, Valid: {}, Corrupted: {}, Missing: {}",
            self.total,
            self.valid,
            self.corrupted.len(),
            self.missing.len()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checksum_calculation() {
        let vector = vec![1.0, 2.0, 3.0, 4.0];
        let checksum1 = IntegrityChecker::calculate_checksum(&vector);
        let checksum2 = IntegrityChecker::calculate_checksum(&vector);
        assert_eq!(checksum1, checksum2);
    }

    #[test]
    fn test_checksum_different_vectors() {
        let vector1 = vec![1.0, 2.0, 3.0];
        let vector2 = vec![1.0, 2.0, 3.1];
        let checksum1 = IntegrityChecker::calculate_checksum(&vector1);
        let checksum2 = IntegrityChecker::calculate_checksum(&vector2);
        assert_ne!(checksum1, checksum2);
    }

    #[test]
    fn test_store_and_verify() {
        let mut checker = IntegrityChecker::new();
        let vector = vec![1.0, 2.0, 3.0];

        checker.store_checksum(1, &vector);
        assert!(checker.verify(1, &vector));

        // Modified vector should fail
        let modified = vec![1.0, 2.0, 3.1];
        assert!(!checker.verify(1, &modified));
    }

    #[test]
    fn test_remove_checksum() {
        let mut checker = IntegrityChecker::new();
        let vector = vec![1.0, 2.0, 3.0];

        checker.store_checksum(1, &vector);
        assert_eq!(checker.len(), 1);

        assert!(checker.remove(1));
        assert_eq!(checker.len(), 0);
        assert!(!checker.verify(1, &vector));
    }

    #[test]
    fn test_integrity_report() {
        let mut checker = IntegrityChecker::new();
        let vectors: HashMap<u64, Vec<f32>> = [
            (1, vec![1.0, 2.0, 3.0]),
            (2, vec![4.0, 5.0, 6.0]),
            (3, vec![7.0, 8.0, 9.0]),
        ]
        .iter()
        .cloned()
        .collect();

        for (&id, vector) in &vectors {
            checker.store_checksum(id, vector);
        }

        let report = checker.get_report(|id| vectors.get(&id).cloned());
        assert_eq!(report.total, 3);
        assert_eq!(report.valid, 3);
        assert!(report.corrupted.is_empty());
        assert!(report.missing.is_empty());
        assert!(report.is_healthy());
    }

    #[test]
    fn test_detect_corruption() {
        let mut checker = IntegrityChecker::new();
        let original = vec![1.0, 2.0, 3.0];
        checker.store_checksum(1, &original);

        let mut vectors: HashMap<u64, Vec<f32>> = HashMap::new();
        vectors.insert(1, vec![1.0, 2.0, 3.1]); // Corrupted

        let report = checker.get_report(|id| vectors.get(&id).cloned());
        assert_eq!(report.corrupted.len(), 1);
        assert_eq!(report.corrupted[0], 1);
        assert!(!report.is_healthy());
    }
}
