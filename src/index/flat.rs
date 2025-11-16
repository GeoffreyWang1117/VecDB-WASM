use super::{Index, SearchResult};
use crate::distance::{calculate_distance, DistanceMetric};
use std::collections::BinaryHeap;
use std::cmp::Ordering;

/// Simple flat index that performs brute-force search
/// Serves as baseline for performance comparison
#[derive(Debug, Clone)]
pub struct FlatIndex {
    vectors: Vec<(u64, Vec<f32>)>,
    metric: DistanceMetric,
    dimension: usize,
}

/// Helper struct for maintaining a max-heap of results (for top-K selection)
#[derive(Debug)]
struct ScoredItem {
    id: u64,
    score: f32,
}

impl PartialEq for ScoredItem {
    fn eq(&self, other: &Self) -> bool {
        self.score == other.score
    }
}

impl Eq for ScoredItem {}

impl PartialOrd for ScoredItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ScoredItem {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse ordering for max-heap (we want to remove largest/worst scores)
        self.score.partial_cmp(&other.score)
            .unwrap_or(Ordering::Equal)
    }
}

impl FlatIndex {
    pub fn new(dimension: usize, metric: DistanceMetric) -> Self {
        Self {
            vectors: Vec::new(),
            metric,
            dimension,
        }
    }

    pub fn dimension(&self) -> usize {
        self.dimension
    }
}

impl Index for FlatIndex {
    fn insert(&mut self, id: u64, vector: &[f32]) {
        assert_eq!(
            vector.len(),
            self.dimension,
            "Vector dimension mismatch: expected {}, got {}",
            self.dimension,
            vector.len()
        );

        // Check if ID already exists and update
        if let Some(pos) = self.vectors.iter().position(|(vid, _)| *vid == id) {
            self.vectors[pos].1 = vector.to_vec();
        } else {
            self.vectors.push((id, vector.to_vec()));
        }
    }

    fn search(&self, query: &[f32], k: usize) -> Vec<SearchResult> {
        assert_eq!(
            query.len(),
            self.dimension,
            "Query dimension mismatch: expected {}, got {}",
            self.dimension,
            query.len()
        );

        if self.vectors.is_empty() {
            return Vec::new();
        }

        let k = k.min(self.vectors.len());

        // Use a max-heap to keep track of top-k results
        let mut heap = BinaryHeap::new();

        for (id, vector) in &self.vectors {
            let score = calculate_distance(query, vector, self.metric);

            if heap.len() < k {
                heap.push(ScoredItem { id: *id, score });
            } else if let Some(worst) = heap.peek() {
                if score > worst.score {
                    heap.pop();
                    heap.push(ScoredItem { id: *id, score });
                }
            }
        }

        // Convert heap to sorted vector (best scores first)
        let mut results: Vec<SearchResult> = heap
            .into_iter()
            .map(|item| SearchResult::new(item.id, item.score))
            .collect();

        // Sort by score descending (best first)
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(Ordering::Equal));

        results
    }

    fn remove(&mut self, id: u64) -> bool {
        if let Some(pos) = self.vectors.iter().position(|(vid, _)| *vid == id) {
            self.vectors.remove(pos);
            true
        } else {
            false
        }
    }

    fn len(&self) -> usize {
        self.vectors.len()
    }

    fn metric(&self) -> DistanceMetric {
        self.metric
    }

    fn clear(&mut self) {
        self.vectors.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flat_index_insert_search() {
        let mut index = FlatIndex::new(3, DistanceMetric::Cosine);

        let v1 = vec![1.0, 0.0, 0.0];
        let v2 = vec![0.0, 1.0, 0.0];
        let v3 = vec![1.0, 1.0, 0.0];

        index.insert(1, &v1);
        index.insert(2, &v2);
        index.insert(3, &v3);

        assert_eq!(index.len(), 3);

        // Search for vector similar to v1
        let results = index.search(&v1, 2);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].id, 1); // v1 should be most similar to itself
    }

    #[test]
    fn test_flat_index_remove() {
        let mut index = FlatIndex::new(2, DistanceMetric::Euclidean);

        index.insert(1, &[1.0, 2.0]);
        index.insert(2, &[3.0, 4.0]);

        assert_eq!(index.len(), 2);
        assert!(index.remove(1));
        assert_eq!(index.len(), 1);
        assert!(!index.remove(1)); // Already removed
    }

    #[test]
    fn test_flat_index_update() {
        let mut index = FlatIndex::new(2, DistanceMetric::Cosine);

        index.insert(1, &[1.0, 0.0]);
        index.insert(1, &[0.0, 1.0]); // Update same ID

        assert_eq!(index.len(), 1); // Should still be 1 vector
    }
}
