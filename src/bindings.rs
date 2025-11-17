use crate::distance::DistanceMetric;
use crate::index::Index;
use crate::index::{FlatIndex, HNSWIndex};
use crate::performance::{OperationTimer, PerformanceMetrics};
use crate::persistence::{DatabaseSnapshot, IndexParams, IndexedDBStore};
use crate::storage::{VectorMetadata, VectorStorage};
use js_sys::Promise;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::future_to_promise;

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
    Manhattan,
    Chebyshev,
    Hamming,
    Angular,
}

impl From<Metric> for DistanceMetric {
    fn from(m: Metric) -> Self {
        match m {
            Metric::Cosine => DistanceMetric::Cosine,
            Metric::Euclidean => DistanceMetric::Euclidean,
            Metric::DotProduct => DistanceMetric::DotProduct,
            Metric::Manhattan => DistanceMetric::Manhattan,
            Metric::Chebyshev => DistanceMetric::Chebyshev,
            Metric::Hamming => DistanceMetric::Hamming,
            Metric::Angular => DistanceMetric::Angular,
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
    metric: DistanceMetric,
    index_type: IndexType,
    hnsw_m: usize,
    hnsw_ef: usize,
    metrics: RefCell<PerformanceMetrics>,
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
    pub fn new(
        dimension: usize,
        metric: Metric,
        index_type: IndexType,
    ) -> Result<VectorDB, JsValue> {
        let metric_internal: DistanceMetric = metric.into();
        let m = 16;
        let ef = 200;

        let index: Box<dyn Index> = match index_type {
            IndexType::Flat => Box::new(FlatIndex::new(dimension, metric_internal)),
            IndexType::HNSW => Box::new(HNSWIndex::new(dimension, metric_internal, m, ef)),
        };

        Ok(VectorDB {
            storage: VectorStorage::new(dimension),
            index,
            dimension,
            metric: metric_internal,
            index_type,
            hnsw_m: m,
            hnsw_ef: ef,
            metrics: RefCell::new(PerformanceMetrics::new()),
        })
    }

    /// Create a new VectorDB with custom HNSW parameters
    #[wasm_bindgen]
    pub fn new_with_hnsw_params(
        dimension: usize,
        metric: Metric,
        m: usize,
        ef_construction: usize,
    ) -> Result<VectorDB, JsValue> {
        let metric_internal: DistanceMetric = metric.into();

        let index = Box::new(HNSWIndex::new(
            dimension,
            metric_internal,
            m,
            ef_construction,
        ));

        Ok(VectorDB {
            storage: VectorStorage::new(dimension),
            index,
            dimension,
            metric: metric_internal,
            index_type: IndexType::HNSW,
            hnsw_m: m,
            hnsw_ef: ef_construction,
            metrics: RefCell::new(PerformanceMetrics::new()),
        })
    }

    /// Insert a vector with metadata
    ///
    /// # Arguments
    /// * `id` - Unique vector ID
    /// * `vector` - Float32Array containing the vector
    /// * `metadata` - Optional JSON string containing metadata
    #[wasm_bindgen]
    pub fn insert(
        &mut self,
        id: u64,
        vector: Vec<f32>,
        metadata: Option<String>,
    ) -> Result<(), JsValue> {
        let timer = OperationTimer::start();

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
        self.storage
            .insert(id, vector.clone(), meta)
            .map_err(|e| JsValue::from_str(&e))?;

        // Insert into index
        self.index.insert(id, &vector);

        // Record performance
        self.metrics.borrow_mut().record_insert(timer.elapsed_ms());

        Ok(())
    }

    /// Search for k nearest neighbors
    ///
    /// # Arguments
    /// * `query` - Query vector as Float32Array
    /// * `k` - Number of results to return
    /// * `include_metadata` - Whether to include metadata in results
    #[wasm_bindgen]
    pub fn search(
        &self,
        query: Vec<f32>,
        k: usize,
        include_metadata: bool,
    ) -> Result<JsValue, JsValue> {
        let timer = OperationTimer::start();

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
                    self.storage
                        .get_metadata(r.id)
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

        // Record performance
        self.metrics.borrow_mut().record_search(timer.elapsed_ms());

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
        self.storage.get_vector(id).cloned()
    }

    /// Batch insert multiple vectors
    #[wasm_bindgen]
    pub fn batch_insert(&mut self, vectors: JsValue) -> Result<usize, JsValue> {
        let timer = OperationTimer::start();

        #[derive(Deserialize)]
        struct BatchVector {
            id: u64,
            vector: Vec<f32>,
            metadata: Option<HashMap<String, String>>,
        }

        let batch: Vec<BatchVector> = serde_wasm_bindgen::from_value(vectors)
            .map_err(|e| JsValue::from_str(&format!("Invalid batch data: {}", e)))?;

        let batch_size = batch.len();
        let mut count = 0;
        for item in batch {
            if item.vector.len() != self.dimension {
                continue; // Skip invalid vectors
            }

            let meta = VectorMetadata::with_data(item.id, item.metadata.unwrap_or_default());

            if self
                .storage
                .insert(item.id, item.vector.clone(), meta)
                .is_ok()
            {
                self.index.insert(item.id, &item.vector);
                count += 1;
            }
        }

        // Record performance
        self.metrics
            .borrow_mut()
            .record_batch(batch_size, timer.elapsed_ms());

        Ok(count)
    }

    /// Search with metadata filter
    /// Filter is a JSON object where all key-value pairs must match
    #[wasm_bindgen]
    pub fn search_with_filter(
        &self,
        query: Vec<f32>,
        k: usize,
        filter: Option<String>,
        include_metadata: bool,
    ) -> Result<JsValue, JsValue> {
        if query.len() != self.dimension {
            return Err(JsValue::from_str(&format!(
                "Query dimension mismatch: expected {}, got {}",
                self.dimension,
                query.len()
            )));
        }

        // Parse filter if provided
        let filter_map: Option<HashMap<String, String>> = if let Some(f) = filter {
            Some(
                serde_json::from_str(&f)
                    .map_err(|e| JsValue::from_str(&format!("Invalid filter JSON: {}", e)))?,
            )
        } else {
            None
        };

        // Get more results than needed to account for filtering
        let search_k = if filter_map.is_some() { k * 3 } else { k };
        let results = self.index.search(&query, search_k);

        // Filter results based on metadata
        let filtered_results: Vec<SearchResult> = results
            .into_iter()
            .filter(|r| {
                if let Some(ref filter) = filter_map {
                    if let Some(metadata) = self.storage.get_metadata(r.id) {
                        // Check if all filter criteria match
                        filter.iter().all(|(key, value)| {
                            metadata.data.get(key).map(|v| v == value).unwrap_or(false)
                        })
                    } else {
                        false
                    }
                } else {
                    true
                }
            })
            .take(k)
            .map(|r| {
                let metadata = if include_metadata {
                    self.storage
                        .get_metadata(r.id)
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

        serde_wasm_bindgen::to_value(&filtered_results)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Export database snapshot as binary
    #[wasm_bindgen]
    pub fn export_snapshot(&self) -> Result<Vec<u8>, JsValue> {
        let metric_str = match self.metric {
            DistanceMetric::Cosine => "Cosine",
            DistanceMetric::Euclidean => "Euclidean",
            DistanceMetric::DotProduct => "DotProduct",
            DistanceMetric::Manhattan => "Manhattan",
            DistanceMetric::Chebyshev => "Chebyshev",
            DistanceMetric::Hamming => "Hamming",
            DistanceMetric::Angular => "Angular",
        };

        let index_type_str = match self.index_type {
            IndexType::Flat => "Flat",
            IndexType::HNSW => "HNSW",
        };

        let index_params = IndexParams {
            hnsw_m: Some(self.hnsw_m),
            hnsw_ef_construction: Some(self.hnsw_ef),
        };

        let mut snapshot = DatabaseSnapshot::new(
            self.dimension,
            metric_str.to_string(),
            index_type_str.to_string(),
            index_params,
        );

        // Add all vectors and metadata
        for (id, vector) in self.storage.iter() {
            let metadata = self
                .storage
                .get_metadata(id)
                .map(|m| m.data.clone())
                .unwrap_or_default();
            snapshot.add_vector(id, vector.clone(), metadata);
        }

        snapshot.to_binary().map_err(|e| JsValue::from_str(&e))
    }

    /// Export database snapshot as JSON
    #[wasm_bindgen]
    pub fn export_snapshot_json(&self) -> Result<String, JsValue> {
        let metric_str = match self.metric {
            DistanceMetric::Cosine => "Cosine",
            DistanceMetric::Euclidean => "Euclidean",
            DistanceMetric::DotProduct => "DotProduct",
            DistanceMetric::Manhattan => "Manhattan",
            DistanceMetric::Chebyshev => "Chebyshev",
            DistanceMetric::Hamming => "Hamming",
            DistanceMetric::Angular => "Angular",
        };

        let index_type_str = match self.index_type {
            IndexType::Flat => "Flat",
            IndexType::HNSW => "HNSW",
        };

        let index_params = IndexParams {
            hnsw_m: Some(self.hnsw_m),
            hnsw_ef_construction: Some(self.hnsw_ef),
        };

        let mut snapshot = DatabaseSnapshot::new(
            self.dimension,
            metric_str.to_string(),
            index_type_str.to_string(),
            index_params,
        );

        for (id, vector) in self.storage.iter() {
            let metadata = self
                .storage
                .get_metadata(id)
                .map(|m| m.data.clone())
                .unwrap_or_default();
            snapshot.add_vector(id, vector.clone(), metadata);
        }

        snapshot.to_json().map_err(|e| JsValue::from_str(&e))
    }

    /// Import database from snapshot binary
    #[wasm_bindgen]
    pub fn import_snapshot(data: &[u8]) -> Result<VectorDB, JsValue> {
        let snapshot = DatabaseSnapshot::from_binary(data).map_err(|e| JsValue::from_str(&e))?;

        let metric = match snapshot.metric.as_str() {
            "Cosine" => Metric::Cosine,
            "Euclidean" => Metric::Euclidean,
            "DotProduct" => Metric::DotProduct,
            "Manhattan" => Metric::Manhattan,
            "Chebyshev" => Metric::Chebyshev,
            "Hamming" => Metric::Hamming,
            "Angular" => Metric::Angular,
            _ => return Err(JsValue::from_str("Unknown metric")),
        };

        let index_type = match snapshot.index_type.as_str() {
            "Flat" => IndexType::Flat,
            "HNSW" => IndexType::HNSW,
            _ => return Err(JsValue::from_str("Unknown index type")),
        };

        let m = snapshot.index_params.hnsw_m.unwrap_or(16);
        let ef = snapshot.index_params.hnsw_ef_construction.unwrap_or(200);

        let mut db = if index_type == IndexType::HNSW {
            VectorDB::new_with_hnsw_params(snapshot.dimension, metric, m, ef)?
        } else {
            VectorDB::new(snapshot.dimension, metric, index_type)?
        };

        // Restore all vectors
        for entry in snapshot.vectors {
            let metadata = snapshot
                .metadata
                .get(&entry.id)
                .cloned()
                .unwrap_or_default();
            let meta = VectorMetadata::with_data(entry.id, metadata);

            db.storage
                .insert(entry.id, entry.vector.clone(), meta)
                .map_err(|e| JsValue::from_str(&e))?;
            db.index.insert(entry.id, &entry.vector);
        }

        Ok(db)
    }

    /// Import database from JSON snapshot
    #[wasm_bindgen]
    pub fn import_snapshot_json(json: &str) -> Result<VectorDB, JsValue> {
        let snapshot = DatabaseSnapshot::from_json(json).map_err(|e| JsValue::from_str(&e))?;

        let metric = match snapshot.metric.as_str() {
            "Cosine" => Metric::Cosine,
            "Euclidean" => Metric::Euclidean,
            "DotProduct" => Metric::DotProduct,
            "Manhattan" => Metric::Manhattan,
            "Chebyshev" => Metric::Chebyshev,
            "Hamming" => Metric::Hamming,
            "Angular" => Metric::Angular,
            _ => return Err(JsValue::from_str("Unknown metric")),
        };

        let index_type = match snapshot.index_type.as_str() {
            "Flat" => IndexType::Flat,
            "HNSW" => IndexType::HNSW,
            _ => return Err(JsValue::from_str("Unknown index type")),
        };

        let m = snapshot.index_params.hnsw_m.unwrap_or(16);
        let ef = snapshot.index_params.hnsw_ef_construction.unwrap_or(200);

        let mut db = if index_type == IndexType::HNSW {
            VectorDB::new_with_hnsw_params(snapshot.dimension, metric, m, ef)?
        } else {
            VectorDB::new(snapshot.dimension, metric, index_type)?
        };

        for entry in snapshot.vectors {
            let metadata = snapshot
                .metadata
                .get(&entry.id)
                .cloned()
                .unwrap_or_default();
            let meta = VectorMetadata::with_data(entry.id, metadata);

            db.storage
                .insert(entry.id, entry.vector.clone(), meta)
                .map_err(|e| JsValue::from_str(&e))?;
            db.index.insert(entry.id, &entry.vector);
        }

        Ok(db)
    }

    /// Save to IndexedDB (async)
    #[wasm_bindgen]
    pub fn save_to_indexeddb(&self, db_name: String) -> Promise {
        let snapshot = match self.export_snapshot() {
            Ok(s) => s,
            Err(e) => return Promise::reject(&e),
        };

        future_to_promise(async move {
            let mut store = IndexedDBStore::new();
            store.open().await?;
            store.save(&db_name, &snapshot).await?;
            Ok(JsValue::from_str("Saved successfully"))
        })
    }

    /// Load from IndexedDB (async)
    /// Returns success message, actual database needs to be reconstructed from snapshot
    #[wasm_bindgen]
    pub fn load_from_indexeddb(db_name: String) -> Promise {
        future_to_promise(async move {
            let mut store = IndexedDBStore::new();
            store.open().await?;
            let data = store.load(&db_name).await?;
            // Return the binary data so JS can call import_snapshot
            let array = js_sys::Uint8Array::new_with_length(data.len() as u32);
            array.copy_from(&data);
            Ok(JsValue::from(array))
        })
    }

    /// Delete from IndexedDB (async)
    #[wasm_bindgen]
    pub fn delete_from_indexeddb(db_name: String) -> Promise {
        future_to_promise(async move {
            let mut store = IndexedDBStore::new();
            store.open().await?;
            store.delete(&db_name).await?;
            Ok(JsValue::from_str("Deleted successfully"))
        })
    }

    /// List all saved databases in IndexedDB (async)
    #[wasm_bindgen]
    pub fn list_saved_databases() -> Promise {
        future_to_promise(async move {
            let mut store = IndexedDBStore::new();
            store.open().await?;
            let keys = store.list_keys().await?;
            serde_wasm_bindgen::to_value(&keys)
                .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
        })
    }

    /// Get statistics about the database
    #[wasm_bindgen]
    /// Batch search: search multiple queries at once
    /// More efficient than calling search() multiple times
    ///
    /// # Arguments
    /// * `queries` - Array of query vectors (as JsValue)
    /// * `k` - Number of results per query
    /// * `include_metadata` - Include metadata in results
    ///
    /// # Returns
    /// Array of arrays of search results
    #[wasm_bindgen]
    pub fn batch_search(
        &self,
        queries: JsValue,
        k: usize,
        include_metadata: bool,
    ) -> Result<JsValue, JsValue> {
        let queries: Vec<Vec<f32>> = serde_wasm_bindgen::from_value(queries)
            .map_err(|e| JsValue::from_str(&format!("Invalid queries format: {}", e)))?;

        let mut all_results = Vec::new();

        for query in queries {
            if query.len() != self.dimension {
                return Err(JsValue::from_str(&format!(
                    "Query dimension mismatch: expected {}, got {}",
                    self.dimension,
                    query.len()
                )));
            }

            let results = self.index.search(&query, k);

            let search_results: Vec<SearchResult> = results
                .into_iter()
                .map(|r| SearchResult {
                    id: r.id,
                    score: r.score,
                    metadata: if include_metadata {
                        self.storage
                            .get_metadata(r.id)
                            .map(|m| serde_json::to_string(&m.data).unwrap_or_default())
                    } else {
                        None
                    },
                })
                .collect();

            all_results.push(search_results);
        }

        serde_wasm_bindgen::to_value(&all_results)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Range search: find all vectors within a distance threshold
    ///
    /// # Arguments
    /// * `query` - Query vector
    /// * `radius` - Maximum distance threshold
    /// * `max_results` - Maximum number of results to return (0 = unlimited)
    /// * `include_metadata` - Include metadata in results
    ///
    /// # Returns
    /// Array of search results within the radius
    #[wasm_bindgen]
    pub fn search_radius(
        &self,
        query: Vec<f32>,
        radius: f32,
        max_results: usize,
        include_metadata: bool,
    ) -> Result<JsValue, JsValue> {
        if query.len() != self.dimension {
            return Err(JsValue::from_str(&format!(
                "Query dimension mismatch: expected {}, got {}",
                self.dimension,
                query.len()
            )));
        }

        // Get a large candidate set and filter by radius
        let candidate_count = if max_results > 0 {
            max_results * 10
        } else {
            self.storage.len()
        };

        let candidates = self.index.search(&query, candidate_count);

        let mut results: Vec<SearchResult> = candidates
            .into_iter()
            .filter(|r| {
                // For distance metrics, smaller distance is better
                // For similarity metrics, larger score is better
                match self.metric {
                    DistanceMetric::Euclidean
                    | DistanceMetric::Manhattan
                    | DistanceMetric::Chebyshev
                    | DistanceMetric::Hamming
                    | DistanceMetric::Angular => {
                        // Convert back from similarity score to distance
                        let dist = if r.score == f32::MAX {
                            0.0
                        } else {
                            (1.0 / r.score) - 1.0
                        };
                        dist <= radius
                    }
                    DistanceMetric::Cosine | DistanceMetric::DotProduct => {
                        // For similarity metrics, use inverse logic
                        r.score >= radius
                    }
                }
            })
            .map(|r| SearchResult {
                id: r.id,
                score: r.score,
                metadata: if include_metadata {
                    self.storage
                        .get_metadata(r.id)
                        .map(|m| serde_json::to_string(&m.data).unwrap_or_default())
                } else {
                    None
                },
            })
            .collect();

        // Limit results if max_results is specified
        if max_results > 0 && results.len() > max_results {
            results.truncate(max_results);
        }

        serde_wasm_bindgen::to_value(&results)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    pub fn get_stats(&self) -> Result<JsValue, JsValue> {
        #[derive(Serialize)]
        struct Stats {
            version: String,
            dimension: usize,
            vector_count: usize,
            metric: String,
            index_type: String,
            hnsw_m: Option<usize>,
            hnsw_ef: Option<usize>,
            estimated_size_bytes: usize,
        }

        let metric_str = match self.metric {
            DistanceMetric::Cosine => "Cosine",
            DistanceMetric::Euclidean => "Euclidean",
            DistanceMetric::DotProduct => "DotProduct",
            DistanceMetric::Manhattan => "Manhattan",
            DistanceMetric::Chebyshev => "Chebyshev",
            DistanceMetric::Hamming => "Hamming",
            DistanceMetric::Angular => "Angular",
        };

        let index_type_str = match self.index_type {
            IndexType::Flat => "Flat",
            IndexType::HNSW => "HNSW",
        };

        let (hnsw_m, hnsw_ef) = match self.index_type {
            IndexType::HNSW => (Some(self.hnsw_m), Some(self.hnsw_ef)),
            _ => (None, None),
        };

        let estimated_size = self.len() * (8 + self.dimension * 4);

        let stats = Stats {
            version: env!("CARGO_PKG_VERSION").to_string(),
            dimension: self.dimension,
            vector_count: self.len(),
            metric: metric_str.to_string(),
            index_type: index_type_str.to_string(),
            hnsw_m,
            hnsw_ef,
            estimated_size_bytes: estimated_size,
        };

        serde_wasm_bindgen::to_value(&stats)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Get performance metrics
    ///
    /// Returns comprehensive performance statistics including:
    /// - Total number of searches, inserts, and batch operations
    /// - Average and peak operation times
    /// - Throughput metrics
    #[wasm_bindgen]
    pub fn get_performance_metrics(&self) -> Result<JsValue, JsValue> {
        let metrics = self.metrics.borrow();
        serde_wasm_bindgen::to_value(&*metrics)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Reset performance metrics
    ///
    /// Clears all performance statistics and starts fresh tracking
    #[wasm_bindgen]
    pub fn reset_performance_metrics(&self) {
        self.metrics.borrow_mut().reset();
    }
}

impl PartialEq for IndexType {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (IndexType::Flat, IndexType::Flat) | (IndexType::HNSW, IndexType::HNSW)
        )
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

/// Normalize a vector to unit length (L2 norm = 1)
/// Useful for cosine similarity calculations
///
/// # Arguments
/// * `vector` - Input vector to normalize
///
/// # Returns
/// Normalized vector with L2 norm = 1
///
/// # Example
/// ```javascript
/// import { normalize_vector } from './pkg/vecdb_wasm.js';
/// const v = [3.0, 4.0];
/// const normalized = normalize_vector(v); // [0.6, 0.8]
/// ```
#[wasm_bindgen]
pub fn normalize_vector(vector: Vec<f32>) -> Vec<f32> {
    crate::distance::normalize_vector(&vector)
}

/// Calculate the L2 norm (magnitude) of a vector
///
/// # Arguments
/// * `vector` - Input vector
///
/// # Returns
/// L2 norm of the vector
///
/// # Example
/// ```javascript
/// import { vector_norm } from './pkg/vecdb_wasm.js';
/// const v = [3.0, 4.0];
/// const norm = vector_norm(v); // 5.0
/// ```
#[wasm_bindgen]
pub fn vector_norm(vector: Vec<f32>) -> f32 {
    crate::distance::vector_norm(&vector)
}
