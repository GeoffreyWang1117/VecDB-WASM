/// Product Quantization (PQ) for vector compression
///
/// PQ splits vectors into subvectors and quantizes each subspace independently.
/// This achieves 10-100x memory compression with minimal accuracy loss.
///
/// # How it works
///
/// 1. Split vector into M subvectors
/// 2. Cluster each subspace with k-means (typically 256 clusters)
/// 3. Replace subvectors with cluster IDs (1 byte each)
/// 4. Store codebooks for reconstruction
///
/// # Example
///
/// ```ignore
/// // Original: 128D * 4 bytes = 512 bytes
/// // PQ: 8 subvectors * 1 byte = 8 bytes
/// // Compression: 64x
/// ```

use serde::{Deserialize, Serialize};

/// Product Quantization encoder
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductQuantizer {
    /// Vector dimension
    dimension: usize,
    /// Number of subvectors (M)
    num_subvectors: usize,
    /// Subvector dimension
    subvector_dim: usize,
    /// Number of clusters per subspace (typically 256 for 1-byte codes)
    num_clusters: usize,
    /// Codebooks: one per subvector
    /// Shape: [M, K, D/M] where M=subvectors, K=clusters, D=dimension
    codebooks: Vec<Vec<Vec<f32>>>,
    /// Whether the quantizer is trained
    trained: bool,
}

impl ProductQuantizer {
    /// Create a new product quantizer
    ///
    /// # Arguments
    ///
    /// * `dimension` - Vector dimension (must be divisible by num_subvectors)
    /// * `num_subvectors` - Number of subvectors (M), typically 8, 16, or 32
    /// * `num_clusters` - Number of clusters per subspace (K), typically 256
    ///
    /// # Example
    ///
    /// ```ignore
    /// let pq = ProductQuantizer::new(128, 8, 256);
    /// ```
    pub fn new(dimension: usize, num_subvectors: usize, num_clusters: usize) -> Self {
        assert!(
            dimension % num_subvectors == 0,
            "Dimension must be divisible by number of subvectors"
        );

        let subvector_dim = dimension / num_subvectors;

        Self {
            dimension,
            num_subvectors,
            subvector_dim,
            num_clusters,
            codebooks: Vec::new(),
            trained: false,
        }
    }

    /// Train the quantizer on a dataset
    ///
    /// Uses k-means clustering on each subspace
    ///
    /// # Arguments
    ///
    /// * `data` - Training vectors
    /// * `max_iterations` - Maximum k-means iterations
    pub fn train(&mut self, data: &[Vec<f32>], max_iterations: usize) {
        assert!(!data.is_empty(), "Training data cannot be empty");
        assert_eq!(data[0].len(), self.dimension, "Dimension mismatch");

        self.codebooks.clear();

        // Train a codebook for each subvector
        for m in 0..self.num_subvectors {
            let start_idx = m * self.subvector_dim;
            let end_idx = start_idx + self.subvector_dim;

            // Extract subvectors
            let subvectors: Vec<Vec<f32>> = data
                .iter()
                .map(|v| v[start_idx..end_idx].to_vec())
                .collect();

            // Run k-means
            let centroids = self.kmeans(&subvectors, self.num_clusters, max_iterations);
            self.codebooks.push(centroids);
        }

        self.trained = true;
    }

    /// Encode a vector to PQ codes
    ///
    /// Returns a vector of cluster IDs (one per subvector)
    pub fn encode(&self, vector: &[f32]) -> Vec<u8> {
        assert!(self.trained, "Quantizer must be trained before encoding");
        assert_eq!(vector.len(), self.dimension, "Dimension mismatch");

        let mut codes = Vec::with_capacity(self.num_subvectors);

        for m in 0..self.num_subvectors {
            let start_idx = m * self.subvector_dim;
            let end_idx = start_idx + self.subvector_dim;
            let subvector = &vector[start_idx..end_idx];

            // Find nearest centroid
            let code = self.find_nearest_centroid(m, subvector);
            codes.push(code as u8);
        }

        codes
    }

    /// Decode PQ codes back to approximate vector
    pub fn decode(&self, codes: &[u8]) -> Vec<f32> {
        assert!(self.trained, "Quantizer must be trained before decoding");
        assert_eq!(codes.len(), self.num_subvectors, "Invalid code length");

        let mut vector = Vec::with_capacity(self.dimension);

        for (m, &code) in codes.iter().enumerate() {
            let centroid = &self.codebooks[m][code as usize];
            vector.extend_from_slice(centroid);
        }

        vector
    }

    /// Compute asymmetric distance between query and encoded vector
    ///
    /// This is faster than decoding and computing distance
    pub fn asymmetric_distance(&self, query: &[f32], codes: &[u8]) -> f32 {
        assert!(self.trained, "Quantizer must be trained");
        assert_eq!(query.len(), self.dimension, "Query dimension mismatch");
        assert_eq!(codes.len(), self.num_subvectors, "Invalid code length");

        let mut distance = 0.0;

        for m in 0..self.num_subvectors {
            let start_idx = m * self.subvector_dim;
            let end_idx = start_idx + self.subvector_dim;
            let query_sub = &query[start_idx..end_idx];

            let code = codes[m] as usize;
            let centroid = &self.codebooks[m][code];

            // Compute squared Euclidean distance for this subspace
            for i in 0..self.subvector_dim {
                let diff = query_sub[i] - centroid[i];
                distance += diff * diff;
            }
        }

        distance.sqrt()
    }

    /// Precompute distance tables for faster search
    ///
    /// For a query vector, precompute distances to all centroids in all subspaces.
    /// This makes subsequent distance computations very fast (just table lookups and sums).
    pub fn compute_distance_table(&self, query: &[f32]) -> Vec<Vec<f32>> {
        assert!(self.trained, "Quantizer must be trained");
        assert_eq!(query.len(), self.dimension, "Query dimension mismatch");

        let mut tables = Vec::with_capacity(self.num_subvectors);

        for m in 0..self.num_subvectors {
            let start_idx = m * self.subvector_dim;
            let end_idx = start_idx + self.subvector_dim;
            let query_sub = &query[start_idx..end_idx];

            let mut distances = Vec::with_capacity(self.num_clusters);

            for centroid in &self.codebooks[m] {
                let mut dist = 0.0;
                for i in 0..self.subvector_dim {
                    let diff = query_sub[i] - centroid[i];
                    dist += diff * diff;
                }
                distances.push(dist);
            }

            tables.push(distances);
        }

        tables
    }

    /// Fast distance computation using precomputed tables
    pub fn table_distance(&self, tables: &[Vec<f32>], codes: &[u8]) -> f32 {
        assert_eq!(tables.len(), self.num_subvectors, "Invalid table size");
        assert_eq!(codes.len(), self.num_subvectors, "Invalid code length");

        let mut distance = 0.0;
        for m in 0..self.num_subvectors {
            distance += tables[m][codes[m] as usize];
        }

        distance.sqrt()
    }

    /// Find nearest centroid in a subspace
    fn find_nearest_centroid(&self, subvector_idx: usize, subvector: &[f32]) -> usize {
        let codebook = &self.codebooks[subvector_idx];

        let mut min_dist = f32::MAX;
        let mut best_idx = 0;

        for (idx, centroid) in codebook.iter().enumerate() {
            let dist = euclidean_distance(subvector, centroid);
            if dist < min_dist {
                min_dist = dist;
                best_idx = idx;
            }
        }

        best_idx
    }

    /// K-means clustering
    fn kmeans(&self, data: &[Vec<f32>], k: usize, max_iterations: usize) -> Vec<Vec<f32>> {
        let n = data.len();
        let d = data[0].len();

        // Initialize centroids randomly from data points
        let mut centroids = Vec::with_capacity(k);
        let step = n / k;
        for i in 0..k {
            let idx = (i * step) % n;
            centroids.push(data[idx].clone());
        }

        let mut assignments = vec![0; n];

        for _iteration in 0..max_iterations {
            let mut changed = false;

            // Assignment step
            for (i, point) in data.iter().enumerate() {
                let mut min_dist = f32::MAX;
                let mut best_cluster = 0;

                for (j, centroid) in centroids.iter().enumerate() {
                    let dist = euclidean_distance(point, centroid);
                    if dist < min_dist {
                        min_dist = dist;
                        best_cluster = j;
                    }
                }

                if assignments[i] != best_cluster {
                    assignments[i] = best_cluster;
                    changed = true;
                }
            }

            if !changed {
                break;
            }

            // Update step
            let mut counts = vec![0; k];
            let mut sums = vec![vec![0.0; d]; k];

            for (i, point) in data.iter().enumerate() {
                let cluster = assignments[i];
                counts[cluster] += 1;
                for j in 0..d {
                    sums[cluster][j] += point[j];
                }
            }

            for i in 0..k {
                if counts[i] > 0 {
                    for j in 0..d {
                        centroids[i][j] = sums[i][j] / counts[i] as f32;
                    }
                }
            }
        }

        centroids
    }

    /// Get compression ratio
    pub fn compression_ratio(&self) -> f32 {
        let original_size = self.dimension * 4; // 4 bytes per f32
        let compressed_size = self.num_subvectors * 1; // 1 byte per code
        original_size as f32 / compressed_size as f32
    }

    /// Get memory saved per vector (in bytes)
    pub fn memory_saved_per_vector(&self) -> usize {
        let original = self.dimension * 4;
        let compressed = self.num_subvectors;
        original - compressed
    }

    /// Check if trained
    pub fn is_trained(&self) -> bool {
        self.trained
    }

    /// Get configuration
    pub fn config(&self) -> (usize, usize, usize) {
        (self.dimension, self.num_subvectors, self.num_clusters)
    }
}

/// Euclidean distance between two vectors
fn euclidean_distance(a: &[f32], b: &[f32]) -> f32 {
    let mut sum = 0.0;
    for i in 0..a.len() {
        let diff = a[i] - b[i];
        sum += diff * diff;
    }
    sum.sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn generate_random_vectors(n: usize, dim: usize) -> Vec<Vec<f32>> {
        (0..n)
            .map(|_| (0..dim).map(|_| rand::random::<f32>()).collect())
            .collect()
    }

    #[test]
    fn test_pq_creation() {
        let pq = ProductQuantizer::new(128, 8, 256);
        assert_eq!(pq.dimension, 128);
        assert_eq!(pq.num_subvectors, 8);
        assert_eq!(pq.subvector_dim, 16);
        assert!(!pq.is_trained());
    }

    #[test]
    fn test_pq_training() {
        let mut pq = ProductQuantizer::new(64, 4, 16);
        let data = generate_random_vectors(100, 64);

        pq.train(&data, 10);
        assert!(pq.is_trained());
        assert_eq!(pq.codebooks.len(), 4);
        assert_eq!(pq.codebooks[0].len(), 16);
    }

    #[test]
    fn test_pq_encode_decode() {
        let mut pq = ProductQuantizer::new(64, 4, 16);
        let data = generate_random_vectors(100, 64);
        pq.train(&data, 10);

        let vector = &data[0];
        let codes = pq.encode(vector);
        assert_eq!(codes.len(), 4);

        let reconstructed = pq.decode(&codes);
        assert_eq!(reconstructed.len(), 64);

        // Check that reconstruction is close to original
        let dist = euclidean_distance(vector, &reconstructed);
        assert!(dist < 10.0); // Should be reasonably close
    }

    #[test]
    fn test_compression_ratio() {
        let pq = ProductQuantizer::new(128, 8, 256);
        assert_eq!(pq.compression_ratio(), 64.0); // 512 bytes / 8 bytes
        assert_eq!(pq.memory_saved_per_vector(), 504); // 512 - 8 bytes
    }

    #[test]
    fn test_asymmetric_distance() {
        let mut pq = ProductQuantizer::new(64, 4, 16);
        let data = generate_random_vectors(100, 64);
        pq.train(&data, 10);

        let query = &data[0];
        let target = &data[1];
        let codes = pq.encode(target);

        let dist = pq.asymmetric_distance(query, &codes);
        assert!(dist >= 0.0);
    }

    #[test]
    fn test_distance_table() {
        let mut pq = ProductQuantizer::new(64, 4, 16);
        let data = generate_random_vectors(100, 64);
        pq.train(&data, 10);

        let query = &data[0];
        let tables = pq.compute_distance_table(query);

        assert_eq!(tables.len(), 4); // 4 subvectors
        assert_eq!(tables[0].len(), 16); // 16 clusters

        // Test table-based distance
        let target_codes = pq.encode(&data[1]);
        let dist = pq.table_distance(&tables, &target_codes);
        assert!(dist >= 0.0);
    }
}
