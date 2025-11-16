use wasm_bindgen::prelude::*;
use crate::distance::DistanceMetric;
use crate::index::Index;
use crate::index::{FlatIndex, HNSWIndex};
use crate::storage::{VectorMetadata, VectorStorage};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[wasm_bindgen]
#[derive(Debug, Clone, Copy)]
pub enum IndexType {
    Flat,
    HNSW,
}

#[wasm_bindgen]
#[derive(Debug, Clone, Copy)]
pub enum Metric {
    Cosine,
    Euclidean,
    DotProduct,
}

impl From<Metric> for DistanceMetric {
    fn from(m: Metric) -> Self {
        match m {
            Metric::Cosine => DistanceMetric::Cosine,
            Metric::Euclidean => DistanceMetric::Euclidean,
            Metric::DotProduct => DistanceMetric::DotProduct,
        }
    }
}

/// Search result returned to JavaScript
#[wasm_bindgen]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    id: u64,
    score: f32,
    metadata: Option<String>,
}

#[wasm_bindgen]
impl SearchResult {
    #[wasm_bindgen(getter)]
    pub fn id(&self) -> u64 {
        self.id
    }

    #[wasm_bindgen(getter)]
    pub fn score(&self) -> f32 {
        self.score
    }

    #[wasm_bindgen(getter)]
    pub fn metadata(&self) -> Option<String> {
        self.metadata.clone()
    }
}

/// Main VectorDB interface exposed to JavaScript
#[wasm_bindgen]
pub struct VectorDB {
    storage: VectorStorage,
    index: Box<dyn Index>,
    dimension: usize,
}

#[wasm_bindgen]
impl VectorDB {
    /// Create a new VectorDB instance
    ///
    /// # Arguments
    /// * `dimension` - Vector dimension
    /// * `metric` - Distance metric (Cosine, Euclidean, DotProduct)
    /// * `index_type` - Index type (Flat, HNSW)
    #[wasm_bindgen(constructor)]
    pub fn new(dimension: usize, metric: Metric, index_type: IndexType) -> Result<VectorDB, JsValue> {
        let metric_internal: DistanceMetric = metric.into();

        let index: Box<dyn Index> = match index_type {
            IndexType::Flat => Box::new(FlatIndex::new(dimension, metric_internal)),
            IndexType::HNSW => Box::new(HNSWIndex::new(dimension, metric_internal, 16, 200)),
        };

        Ok(VectorDB {
            storage: VectorStorage::new(dimension),
            index,
            dimension,
        })
    }

    /// Insert a vector with metadata
    ///
    /// # Arguments
    /// * `id` - Unique vector ID
    /// * `vector` - Float32Array containing the vector
    /// * `metadata` - Optional JSON string containing metadata
    #[wasm_bindgen]
    pub fn insert(&mut self, id: u64, vector: Vec<f32>, metadata: Option<String>) -> Result<(), JsValue> {
        if vector.len() != self.dimension {
            return Err(JsValue::from_str(&format!(
                "Vector dimension mismatch: expected {}, got {}",
                self.dimension,
                vector.len()
            )));
        }

        // Parse metadata if provided
        let meta = if let Some(json_str) = metadata {
            let data: HashMap<String, String> = serde_json::from_str(&json_str)
                .map_err(|e| JsValue::from_str(&format!("Invalid metadata JSON: {}", e)))?;
            VectorMetadata::with_data(id, data)
        } else {
            VectorMetadata::new(id)
        };

        // Insert into storage
        self.storage.insert(id, vector.clone(), meta)
            .map_err(|e| JsValue::from_str(&e))?;

        // Insert into index
        self.index.insert(id, &vector);

        Ok(())
    }

    /// Search for k nearest neighbors
    ///
    /// # Arguments
    /// * `query` - Query vector as Float32Array
    /// * `k` - Number of results to return
    /// * `include_metadata` - Whether to include metadata in results
    #[wasm_bindgen]
    pub fn search(&self, query: Vec<f32>, k: usize, include_metadata: bool) -> Result<JsValue, JsValue> {
        if query.len() != self.dimension {
            return Err(JsValue::from_str(&format!(
                "Query dimension mismatch: expected {}, got {}",
                self.dimension,
                query.len()
            )));
        }

        let results = self.index.search(&query, k);

        // Convert to JavaScript-friendly format
        let js_results: Vec<SearchResult> = results
            .into_iter()
            .map(|r| {
                let metadata = if include_metadata {
                    self.storage.get_metadata(r.id)
                        .and_then(|m| serde_json::to_string(&m.data).ok())
                } else {
                    None
                };

                SearchResult {
                    id: r.id,
                    score: r.score,
                    metadata,
                }
            })
            .collect();

        serde_wasm_bindgen::to_value(&js_results)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Remove a vector by ID
    #[wasm_bindgen]
    pub fn remove(&mut self, id: u64) -> bool {
        self.storage.remove(id);
        self.index.remove(id)
    }

    /// Get the number of vectors in the database
    #[wasm_bindgen]
    pub fn len(&self) -> usize {
        self.index.len()
    }

    /// Check if database is empty
    #[wasm_bindgen]
    pub fn is_empty(&self) -> bool {
        self.index.is_empty()
    }

    /// Get vector dimension
    #[wasm_bindgen]
    pub fn dimension(&self) -> usize {
        self.dimension
    }

    /// Clear all vectors
    #[wasm_bindgen]
    pub fn clear(&mut self) {
        self.index.clear();
        self.storage = VectorStorage::new(self.dimension);
    }

    /// Get vector by ID
    #[wasm_bindgen]
    pub fn get_vector(&self, id: u64) -> Option<Vec<f32>> {
        self.storage.get_vector(id).map(|v| v.clone())
    }

    /// Batch insert multiple vectors
    #[wasm_bindgen]
    pub fn batch_insert(&mut self, vectors: JsValue) -> Result<usize, JsValue> {
        #[derive(Deserialize)]
        struct BatchVector {
            id: u64,
            vector: Vec<f32>,
            metadata: Option<HashMap<String, String>>,
        }

        let batch: Vec<BatchVector> = serde_wasm_bindgen::from_value(vectors)
            .map_err(|e| JsValue::from_str(&format!("Invalid batch data: {}", e)))?;

        let mut count = 0;
        for item in batch {
            if item.vector.len() != self.dimension {
                continue; // Skip invalid vectors
            }

            let meta = VectorMetadata::with_data(
                item.id,
                item.metadata.unwrap_or_default()
            );

            if self.storage.insert(item.id, item.vector.clone(), meta).is_ok() {
                self.index.insert(item.id, &item.vector);
                count += 1;
            }
        }

        Ok(count)
    }
}

// Additional utility functions
#[wasm_bindgen]
pub fn init_panic_hook() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
