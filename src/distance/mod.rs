pub mod simd;

use serde::{Deserialize, Serialize};

/// Distance metric types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DistanceMetric {
    /// Cosine similarity (normalized dot product). Range: 0-2 (0 = identical)
    Cosine,
    /// Euclidean distance (L2 norm). Range: 0-∞ (0 = identical)
    Euclidean,
    /// Dot product (inner product). Range: -∞ to ∞ (higher = more similar)
    DotProduct,
    /// Manhattan distance (L1 norm, city block). Range: 0-∞ (0 = identical)
    Manhattan,
    /// Chebyshev distance (L∞ norm, maximum). Range: 0-∞ (0 = identical)
    Chebyshev,
    /// Hamming distance (for binary/integer vectors). Range: 0-dimension
    Hamming,
    /// Angular distance (angle between vectors). Range: 0-π (0 = identical)
    Angular,
}

/// Calculate cosine similarity (1 - cosine distance)
/// Returns a value between 0 and 1, where 1 is most similar
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    debug_assert_eq!(a.len(), b.len(), "Vectors must have same dimension");

    let mut dot_product = 0.0;
    let mut norm_a = 0.0;
    let mut norm_b = 0.0;

    for i in 0..a.len() {
        dot_product += a[i] * b[i];
        norm_a += a[i] * a[i];
        norm_b += b[i] * b[i];
    }

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }

    let cosine = dot_product / (norm_a.sqrt() * norm_b.sqrt());
    // Convert to similarity (0-1 range, higher is better)
    (cosine + 1.0) / 2.0
}

/// Calculate Euclidean distance
/// Lower values indicate more similar vectors
pub fn euclidean_distance(a: &[f32], b: &[f32]) -> f32 {
    debug_assert_eq!(a.len(), b.len(), "Vectors must have same dimension");

    let mut sum = 0.0;
    for i in 0..a.len() {
        let diff = a[i] - b[i];
        sum += diff * diff;
    }
    sum.sqrt()
}

/// Calculate dot product
/// Higher values indicate more similar vectors
pub fn dot_product(a: &[f32], b: &[f32]) -> f32 {
    debug_assert_eq!(a.len(), b.len(), "Vectors must have same dimension");

    let mut sum = 0.0;
    for i in 0..a.len() {
        sum += a[i] * b[i];
    }
    sum
}

/// Calculate Manhattan distance (L1 norm)
/// Also known as city block distance or taxicab distance
/// Lower values indicate more similar vectors
pub fn manhattan_distance(a: &[f32], b: &[f32]) -> f32 {
    debug_assert_eq!(a.len(), b.len(), "Vectors must have same dimension");

    let mut sum = 0.0;
    for i in 0..a.len() {
        sum += (a[i] - b[i]).abs();
    }
    sum
}

/// Calculate Chebyshev distance (L∞ norm)
/// Also known as maximum metric or chessboard distance
/// Lower values indicate more similar vectors
pub fn chebyshev_distance(a: &[f32], b: &[f32]) -> f32 {
    debug_assert_eq!(a.len(), b.len(), "Vectors must have same dimension");

    let mut max_diff = 0.0;
    for i in 0..a.len() {
        let diff = (a[i] - b[i]).abs();
        if diff > max_diff {
            max_diff = diff;
        }
    }
    max_diff
}

/// Calculate Hamming distance
/// Counts the number of positions where vectors differ
/// Treats vectors as binary (rounds to nearest integer)
/// Lower values indicate more similar vectors
pub fn hamming_distance(a: &[f32], b: &[f32]) -> f32 {
    debug_assert_eq!(a.len(), b.len(), "Vectors must have same dimension");

    let mut count = 0.0;
    for i in 0..a.len() {
        if a[i].round() != b[i].round() {
            count += 1.0;
        }
    }
    count
}

/// Calculate Angular distance
/// Measures the angle between two vectors in radians
/// Range: 0 to π (0 = identical direction)
/// Lower values indicate more similar vectors
pub fn angular_distance(a: &[f32], b: &[f32]) -> f32 {
    debug_assert_eq!(a.len(), b.len(), "Vectors must have same dimension");

    let mut dot = 0.0;
    let mut norm_a = 0.0;
    let mut norm_b = 0.0;

    for i in 0..a.len() {
        dot += a[i] * b[i];
        norm_a += a[i] * a[i];
        norm_b += b[i] * b[i];
    }

    if norm_a == 0.0 || norm_b == 0.0 {
        return std::f32::consts::PI; // Maximum distance
    }

    let cos_angle = dot / (norm_a.sqrt() * norm_b.sqrt());
    // Clamp to [-1, 1] to handle floating point errors
    let cos_angle = cos_angle.clamp(-1.0, 1.0);
    cos_angle.acos()
}

/// Normalize a vector to unit length (L2 norm = 1)
/// Returns a new normalized vector
///
/// # Arguments
/// * `vector` - Input vector to normalize
///
/// # Returns
/// Normalized vector with L2 norm = 1. Returns zero vector if input is zero vector.
///
/// # Example
/// ```
/// use vecdb_wasm::distance::normalize_vector;
/// let v = vec![3.0, 4.0];
/// let normalized = normalize_vector(&v);
/// // normalized ≈ [0.6, 0.8]
/// ```
pub fn normalize_vector(vector: &[f32]) -> Vec<f32> {
    let mut norm = 0.0;
    for &v in vector {
        norm += v * v;
    }
    norm = norm.sqrt();

    if norm == 0.0 {
        return vector.to_vec();
    }

    vector.iter().map(|&v| v / norm).collect()
}

/// Normalize a vector in-place to unit length (L2 norm = 1)
///
/// # Arguments
/// * `vector` - Mutable vector to normalize in-place
///
/// # Example
/// ```
/// use vecdb_wasm::distance::normalize_vector_inplace;
/// let mut v = vec![3.0, 4.0];
/// normalize_vector_inplace(&mut v);
/// // v ≈ [0.6, 0.8]
/// ```
pub fn normalize_vector_inplace(vector: &mut [f32]) {
    let mut norm = 0.0;
    for &v in vector.iter() {
        norm += v * v;
    }
    norm = norm.sqrt();

    if norm == 0.0 {
        return;
    }

    for v in vector.iter_mut() {
        *v /= norm;
    }
}

/// Calculate the L2 norm (magnitude) of a vector
pub fn vector_norm(vector: &[f32]) -> f32 {
    let mut sum = 0.0;
    for &v in vector {
        sum += v * v;
    }
    sum.sqrt()
}

/// Calculate distance/similarity based on metric
/// Note: Higher values always mean "better" (more similar)
/// Automatically uses SIMD when available and dimension >= 4
pub fn calculate_distance(a: &[f32], b: &[f32], metric: DistanceMetric) -> f32 {
    // Use SIMD for larger vectors when available (only for existing metrics)
    #[cfg(all(target_arch = "wasm32", target_feature = "simd128"))]
    if a.len() >= 4 {
        match metric {
            DistanceMetric::Cosine => return simd::cosine_similarity_simd(a, b),
            DistanceMetric::Euclidean => {
                let dist = simd::euclidean_distance_simd(a, b);
                return if dist == 0.0 {
                    f32::MAX
                } else {
                    1.0 / (1.0 + dist)
                };
            }
            DistanceMetric::DotProduct => return simd::dot_product_simd(a, b),
            _ => {} // Fall through to scalar for new metrics
        }
    }

    // Scalar implementation for all metrics
    match metric {
        DistanceMetric::Cosine => cosine_similarity(a, b),
        DistanceMetric::Euclidean => {
            let dist = euclidean_distance(a, b);
            if dist == 0.0 {
                f32::MAX
            } else {
                1.0 / (1.0 + dist)
            }
        }
        DistanceMetric::DotProduct => dot_product(a, b),
        DistanceMetric::Manhattan => {
            let dist = manhattan_distance(a, b);
            if dist == 0.0 {
                f32::MAX
            } else {
                1.0 / (1.0 + dist)
            }
        }
        DistanceMetric::Chebyshev => {
            let dist = chebyshev_distance(a, b);
            if dist == 0.0 {
                f32::MAX
            } else {
                1.0 / (1.0 + dist)
            }
        }
        DistanceMetric::Hamming => {
            let dist = hamming_distance(a, b);
            if dist == 0.0 {
                f32::MAX
            } else {
                1.0 / (1.0 + dist)
            }
        }
        DistanceMetric::Angular => {
            let dist = angular_distance(a, b);
            if dist == 0.0 {
                f32::MAX
            } else {
                // Convert from radians [0, π] to similarity score
                1.0 / (1.0 + dist)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        let sim = cosine_similarity(&a, &b);
        assert!((sim - 1.0).abs() < 1e-6);

        let c = vec![1.0, 0.0, 0.0];
        let d = vec![0.0, 1.0, 0.0];
        let sim2 = cosine_similarity(&c, &d);
        assert!((sim2 - 0.5).abs() < 1e-6); // Orthogonal vectors
    }

    #[test]
    fn test_euclidean_distance() {
        let a = vec![0.0, 0.0];
        let b = vec![3.0, 4.0];
        let dist = euclidean_distance(&a, &b);
        assert!((dist - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_dot_product() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![4.0, 5.0, 6.0];
        let dot = dot_product(&a, &b);
        assert!((dot - 32.0).abs() < 1e-6); // 1*4 + 2*5 + 3*6 = 32
    }

    #[test]
    fn test_normalize_vector() {
        // Test with [3, 4] -> should normalize to [0.6, 0.8]
        let v = vec![3.0, 4.0];
        let normalized = normalize_vector(&v);
        assert!((normalized[0] - 0.6).abs() < 1e-6);
        assert!((normalized[1] - 0.8).abs() < 1e-6);

        // Verify norm is 1
        let norm = vector_norm(&normalized);
        assert!((norm - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_normalize_vector_inplace() {
        let mut v = vec![3.0, 4.0];
        normalize_vector_inplace(&mut v);
        assert!((v[0] - 0.6).abs() < 1e-6);
        assert!((v[1] - 0.8).abs() < 1e-6);

        // Verify norm is 1
        let norm = vector_norm(&v);
        assert!((norm - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_zero_vector_normalization() {
        let v = vec![0.0, 0.0, 0.0];
        let normalized = normalize_vector(&v);
        assert_eq!(normalized, vec![0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_vector_norm() {
        let v = vec![3.0, 4.0];
        let norm = vector_norm(&v);
        assert!((norm - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_manhattan_distance() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![4.0, 6.0, 8.0];
        let dist = manhattan_distance(&a, &b);
        // |1-4| + |2-6| + |3-8| = 3 + 4 + 5 = 12
        assert!((dist - 12.0).abs() < 1e-6);

        // Same vectors should have distance 0
        let c = vec![1.0, 2.0, 3.0];
        let d = vec![1.0, 2.0, 3.0];
        assert_eq!(manhattan_distance(&c, &d), 0.0);
    }

    #[test]
    fn test_chebyshev_distance() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![4.0, 6.0, 5.0];
        let dist = chebyshev_distance(&a, &b);
        // max(|1-4|, |2-6|, |3-5|) = max(3, 4, 2) = 4
        assert!((dist - 4.0).abs() < 1e-6);

        // Same vectors should have distance 0
        let c = vec![1.0, 2.0, 3.0];
        let d = vec![1.0, 2.0, 3.0];
        assert_eq!(chebyshev_distance(&c, &d), 0.0);
    }

    #[test]
    fn test_hamming_distance() {
        // Binary-like vectors
        let a = vec![1.0, 0.0, 1.0, 1.0, 0.0];
        let b = vec![1.0, 1.0, 1.0, 0.0, 0.0];
        let dist = hamming_distance(&a, &b);
        // Differs at positions 1 and 3 = 2
        assert!((dist - 2.0).abs() < 1e-6);

        // Same vectors should have distance 0
        let c = vec![1.0, 0.0, 1.0];
        let d = vec![1.0, 0.0, 1.0];
        assert_eq!(hamming_distance(&c, &d), 0.0);
    }

    #[test]
    fn test_angular_distance() {
        // Parallel vectors (same direction)
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![2.0, 0.0, 0.0];
        let dist = angular_distance(&a, &b);
        assert!(dist.abs() < 1e-6); // Should be ~0

        // Orthogonal vectors (90 degrees)
        let c = vec![1.0, 0.0, 0.0];
        let d = vec![0.0, 1.0, 0.0];
        let dist2 = angular_distance(&c, &d);
        assert!((dist2 - std::f32::consts::FRAC_PI_2).abs() < 1e-6); // Should be π/2

        // Opposite vectors (180 degrees)
        let e = vec![1.0, 0.0, 0.0];
        let f = vec![-1.0, 0.0, 0.0];
        let dist3 = angular_distance(&e, &f);
        assert!((dist3 - std::f32::consts::PI).abs() < 1e-6); // Should be π
    }

    #[test]
    fn test_calculate_distance_new_metrics() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![4.0, 5.0, 6.0];

        // Test Manhattan
        let sim_manhattan = calculate_distance(&a, &b, DistanceMetric::Manhattan);
        assert!(sim_manhattan > 0.0); // Should return similarity score

        // Test Chebyshev
        let sim_chebyshev = calculate_distance(&a, &b, DistanceMetric::Chebyshev);
        assert!(sim_chebyshev > 0.0);

        // Test Hamming
        let sim_hamming = calculate_distance(&a, &b, DistanceMetric::Hamming);
        assert!(sim_hamming > 0.0);

        // Test Angular
        let sim_angular = calculate_distance(&a, &b, DistanceMetric::Angular);
        assert!(sim_angular > 0.0);

        // Identical vectors should give highest similarity
        let c = vec![1.0, 2.0, 3.0];
        let d = vec![1.0, 2.0, 3.0];
        let sim_identical = calculate_distance(&c, &d, DistanceMetric::Manhattan);
        assert_eq!(sim_identical, f32::MAX);
    }
}
