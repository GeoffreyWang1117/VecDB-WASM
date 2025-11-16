// WASM SIMD optimized distance calculations
// SIMD128 provides 128-bit wide operations for up to 4x speedup

#[cfg(target_arch = "wasm32")]
use std::arch::wasm32::*;

/// SIMD-optimized cosine similarity
/// Processes 4 floats at a time using WASM SIMD128
#[cfg(all(target_arch = "wasm32", target_feature = "simd128"))]
pub fn cosine_similarity_simd(a: &[f32], b: &[f32]) -> f32 {
    debug_assert_eq!(a.len(), b.len());

    let len = a.len();
    let chunks = len / 4;
    let remainder = len % 4;

    unsafe {
        let mut dot_sum = f32x4_splat(0.0);
        let mut norm_a_sum = f32x4_splat(0.0);
        let mut norm_b_sum = f32x4_splat(0.0);

        // Process 4 elements at a time
        for i in 0..chunks {
            let idx = i * 4;

            let va = v128_load(a.as_ptr().add(idx) as *const v128);
            let vb = v128_load(b.as_ptr().add(idx) as *const v128);

            // dot_product += a * b
            dot_sum = f32x4_add(dot_sum, f32x4_mul(va, vb));

            // norm_a += a * a
            norm_a_sum = f32x4_add(norm_a_sum, f32x4_mul(va, va));

            // norm_b += b * b
            norm_b_sum = f32x4_add(norm_b_sum, f32x4_mul(vb, vb));
        }

        // Horizontal sum of SIMD vectors
        let dot_product = f32x4_extract_lane::<0>(dot_sum)
            + f32x4_extract_lane::<1>(dot_sum)
            + f32x4_extract_lane::<2>(dot_sum)
            + f32x4_extract_lane::<3>(dot_sum);

        let norm_a = f32x4_extract_lane::<0>(norm_a_sum)
            + f32x4_extract_lane::<1>(norm_a_sum)
            + f32x4_extract_lane::<2>(norm_a_sum)
            + f32x4_extract_lane::<3>(norm_a_sum);

        let norm_b = f32x4_extract_lane::<0>(norm_b_sum)
            + f32x4_extract_lane::<1>(norm_b_sum)
            + f32x4_extract_lane::<2>(norm_b_sum)
            + f32x4_extract_lane::<3>(norm_b_sum);

        // Process remaining elements
        let mut dot_remainder = 0.0;
        let mut norm_a_remainder = 0.0;
        let mut norm_b_remainder = 0.0;

        for i in (chunks * 4)..(chunks * 4 + remainder) {
            dot_remainder += a[i] * b[i];
            norm_a_remainder += a[i] * a[i];
            norm_b_remainder += b[i] * b[i];
        }

        let final_dot = dot_product + dot_remainder;
        let final_norm_a = norm_a + norm_a_remainder;
        let final_norm_b = norm_b + norm_b_remainder;

        if final_norm_a == 0.0 || final_norm_b == 0.0 {
            return 0.0;
        }

        let cosine = final_dot / (final_norm_a.sqrt() * final_norm_b.sqrt());
        (cosine + 1.0) / 2.0
    }
}

/// SIMD-optimized Euclidean distance
#[cfg(all(target_arch = "wasm32", target_feature = "simd128"))]
pub fn euclidean_distance_simd(a: &[f32], b: &[f32]) -> f32 {
    debug_assert_eq!(a.len(), b.len());

    let len = a.len();
    let chunks = len / 4;
    let remainder = len % 4;

    unsafe {
        let mut sum = f32x4_splat(0.0);

        // Process 4 elements at a time
        for i in 0..chunks {
            let idx = i * 4;

            let va = v128_load(a.as_ptr().add(idx) as *const v128);
            let vb = v128_load(b.as_ptr().add(idx) as *const v128);

            let diff = f32x4_sub(va, vb);
            sum = f32x4_add(sum, f32x4_mul(diff, diff));
        }

        // Horizontal sum
        let mut result = f32x4_extract_lane::<0>(sum)
            + f32x4_extract_lane::<1>(sum)
            + f32x4_extract_lane::<2>(sum)
            + f32x4_extract_lane::<3>(sum);

        // Process remaining elements
        for i in (chunks * 4)..(chunks * 4 + remainder) {
            let diff = a[i] - b[i];
            result += diff * diff;
        }

        result.sqrt()
    }
}

/// SIMD-optimized dot product
#[cfg(all(target_arch = "wasm32", target_feature = "simd128"))]
pub fn dot_product_simd(a: &[f32], b: &[f32]) -> f32 {
    debug_assert_eq!(a.len(), b.len());

    let len = a.len();
    let chunks = len / 4;
    let remainder = len % 4;

    unsafe {
        let mut sum = f32x4_splat(0.0);

        // Process 4 elements at a time
        for i in 0..chunks {
            let idx = i * 4;

            let va = v128_load(a.as_ptr().add(idx) as *const v128);
            let vb = v128_load(b.as_ptr().add(idx) as *const v128);

            sum = f32x4_add(sum, f32x4_mul(va, vb));
        }

        // Horizontal sum
        let mut result = f32x4_extract_lane::<0>(sum)
            + f32x4_extract_lane::<1>(sum)
            + f32x4_extract_lane::<2>(sum)
            + f32x4_extract_lane::<3>(sum);

        // Process remaining elements
        for i in (chunks * 4)..(chunks * 4 + remainder) {
            result += a[i] * b[i];
        }

        result
    }
}

// Fallback implementations for non-SIMD builds
#[cfg(not(all(target_arch = "wasm32", target_feature = "simd128")))]
pub fn cosine_similarity_simd(a: &[f32], b: &[f32]) -> f32 {
    super::cosine_similarity(a, b)
}

#[cfg(not(all(target_arch = "wasm32", target_feature = "simd128")))]
pub fn euclidean_distance_simd(a: &[f32], b: &[f32]) -> f32 {
    super::euclidean_distance(a, b)
}

#[cfg(not(all(target_arch = "wasm32", target_feature = "simd128")))]
pub fn dot_product_simd(a: &[f32], b: &[f32]) -> f32 {
    super::dot_product(a, b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simd_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0, 0.0];
        let sim = cosine_similarity_simd(&a, &b);
        assert!((sim - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_simd_euclidean_distance() {
        let a = vec![0.0, 0.0, 0.0, 0.0];
        let b = vec![3.0, 4.0, 0.0, 0.0];
        let dist = euclidean_distance_simd(&a, &b);
        assert!((dist - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_simd_dot_product() {
        let a = vec![1.0, 2.0, 3.0, 4.0];
        let b = vec![5.0, 6.0, 7.0, 8.0];
        let dot = dot_product_simd(&a, &b);
        assert!((dot - 70.0).abs() < 1e-5); // 1*5 + 2*6 + 3*7 + 4*8 = 70
    }
}
