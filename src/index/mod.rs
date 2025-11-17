pub mod flat;
pub mod hnsw;
pub mod pq_hnsw;

pub use flat::FlatIndex;
pub use hnsw::HNSWIndex;
pub use pq_hnsw::PQHNSWIndex;

use crate::distance::DistanceMetric;
use crate::storage::VectorMetadata;
use serde::{Deserialize, Serialize};

/// Search result containing ID, score, and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: u64,
    pub score: f32,
    pub metadata: Option<VectorMetadata>,
}

impl SearchResult {
    pub fn new(id: u64, score: f32) -> Self {
        Self {
            id,
            score,
            metadata: None,
        }
    }

    pub fn with_metadata(id: u64, score: f32, metadata: VectorMetadata) -> Self {
        Self {
            id,
            score,
            metadata: Some(metadata),
        }
    }
}

/// Base trait for all index implementations
pub trait Index: Send {
    /// Insert a vector with given ID
    fn insert(&mut self, id: u64, vector: &[f32]);

    /// Search for k nearest neighbors
    fn search(&self, query: &[f32], k: usize) -> Vec<SearchResult>;

    /// Remove a vector by ID
    fn remove(&mut self, id: u64) -> bool;

    /// Get number of vectors in index
    fn len(&self) -> usize;

    /// Check if index is empty
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get the distance metric used
    fn metric(&self) -> DistanceMetric;

    /// Clear all vectors from index
    fn clear(&mut self);
}
