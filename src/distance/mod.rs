pub mod simd;

use serde::{Deserialize, Serialize};

/// Distance metric types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DistanceMetric {
    Cosine,
    Euclidean,
    DotProduct,
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

/// Calculate distance/similarity based on metric
/// Note: Higher values always mean "better" (more similar)
/// Automatically uses SIMD when available and dimension >= 4
pub fn calculate_distance(a: &[f32], b: &[f32], metric: DistanceMetric) -> f32 {
    // Use SIMD for larger vectors when available
    #[cfg(all(target_arch = "wasm32", target_feature = "simd128"))]
    if a.len() >= 4 {
        return match metric {
            DistanceMetric::Cosine => simd::cosine_similarity_simd(a, b),
            DistanceMetric::Euclidean => {
                let dist = simd::euclidean_distance_simd(a, b);
                if dist == 0.0 {
                    f32::MAX
                } else {
                    1.0 / (1.0 + dist)
                }
            }
            DistanceMetric::DotProduct => simd::dot_product_simd(a, b),
        };
    }

    // Fallback to scalar implementation
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
}
