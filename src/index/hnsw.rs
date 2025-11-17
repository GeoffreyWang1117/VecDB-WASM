use super::{Index, SearchResult};
use crate::distance::DistanceMetric;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};

/// HNSW (Hierarchical Navigable Small World) index
/// High-performance approximate nearest neighbor search
#[derive(Debug, Clone)]
pub struct HNSWIndex {
    layers: Vec<Layer>,
    vectors: HashMap<u64, Vec<f32>>,
    metric: DistanceMetric,
    dimension: usize,
    ef_construction: usize,
    m: usize,
    m_max: usize,
    m_max0: usize,
    #[allow(dead_code)]
    ml: f32,
    entry_point: Option<u64>,
}

#[derive(Debug, Clone)]
struct Layer {
    graph: HashMap<u64, Vec<u64>>,
}

impl Layer {
    fn new() -> Self {
        Self {
            graph: HashMap::new(),
        }
    }

    fn insert_node(&mut self, id: u64) {
        self.graph.entry(id).or_default();
    }

    fn add_edge(&mut self, from: u64, to: u64, m_max: usize) {
        let neighbors = self.graph.entry(from).or_default();
        if !neighbors.contains(&to) {
            neighbors.push(to);
            // Prune connections if exceeds m_max
            if neighbors.len() > m_max {
                neighbors.truncate(m_max);
            }
        }
    }

    fn get_neighbors(&self, id: u64) -> Option<&Vec<u64>> {
        self.graph.get(&id)
    }

    fn remove_node(&mut self, id: u64) {
        self.graph.remove(&id);
        // Remove edges pointing to this node
        for neighbors in self.graph.values_mut() {
            neighbors.retain(|&nid| nid != id);
        }
    }
}

#[derive(Debug)]
struct Candidate {
    id: u64,
    distance: f32,
}

impl PartialEq for Candidate {
    fn eq(&self, other: &Self) -> bool {
        self.distance == other.distance
    }
}

impl Eq for Candidate {}

impl PartialOrd for Candidate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Candidate {
    fn cmp(&self, other: &Self) -> Ordering {
        // Min-heap: smaller distances first
        other
            .distance
            .partial_cmp(&self.distance)
            .unwrap_or(Ordering::Equal)
    }
}

impl HNSWIndex {
    /// Create new HNSW index
    ///
    /// # Arguments
    /// * `dimension` - Vector dimension
    /// * `metric` - Distance metric to use
    /// * `m` - Number of connections per layer (default: 16)
    /// * `ef_construction` - Size of dynamic candidate list (default: 200)
    pub fn new(dimension: usize, metric: DistanceMetric, m: usize, ef_construction: usize) -> Self {
        let m_max = m;
        let m_max0 = m * 2;
        let ml = 1.0 / (m as f32).ln();

        Self {
            layers: vec![Layer::new()],
            vectors: HashMap::new(),
            metric,
            dimension,
            ef_construction,
            m,
            m_max,
            m_max0,
            ml,
            entry_point: None,
        }
    }

    /// Get random level for new node
    fn get_random_level(&self) -> usize {
        let mut level = 0;
        let mut r = rand::random::<f32>();
        while r < 0.5 && level < 16 {
            level += 1;
            r = rand::random::<f32>();
        }
        level
    }

    /// Search for nearest neighbors at a specific layer
    fn search_layer(
        &self,
        query: &[f32],
        entry_points: Vec<u64>,
        ef: usize,
        layer_idx: usize,
    ) -> Vec<Candidate> {
        let mut visited = HashSet::new();
        let mut candidates = BinaryHeap::new();
        let mut results = BinaryHeap::new();

        // Initialize with entry points
        for ep in entry_points {
            if let Some(vec) = self.vectors.get(&ep) {
                let dist = self.calculate_distance_internal(query, vec);
                candidates.push(Candidate {
                    id: ep,
                    distance: dist,
                });
                results.push(Candidate {
                    id: ep,
                    distance: dist,
                });
                visited.insert(ep);
            }
        }

        while let Some(current) = candidates.pop() {
            // Check if current is worse than worst result
            if let Some(worst) = results.peek() {
                if current.distance > worst.distance {
                    break;
                }
            }

            // Explore neighbors
            if layer_idx < self.layers.len() {
                if let Some(neighbors) = self.layers[layer_idx].get_neighbors(current.id) {
                    for &neighbor_id in neighbors {
                        if visited.insert(neighbor_id) {
                            if let Some(neighbor_vec) = self.vectors.get(&neighbor_id) {
                                let dist = self.calculate_distance_internal(query, neighbor_vec);

                                if results.len() < ef {
                                    candidates.push(Candidate {
                                        id: neighbor_id,
                                        distance: dist,
                                    });
                                    results.push(Candidate {
                                        id: neighbor_id,
                                        distance: dist,
                                    });
                                } else if let Some(worst) = results.peek() {
                                    if dist < worst.distance {
                                        results.pop();
                                        results.push(Candidate {
                                            id: neighbor_id,
                                            distance: dist,
                                        });
                                        candidates.push(Candidate {
                                            id: neighbor_id,
                                            distance: dist,
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        results.into_sorted_vec()
    }

    fn calculate_distance_internal(&self, a: &[f32], b: &[f32]) -> f32 {
        // For HNSW we use actual distance (lower is better)
        match self.metric {
            DistanceMetric::Cosine => 1.0 - crate::distance::cosine_similarity(a, b),
            DistanceMetric::Euclidean => crate::distance::euclidean_distance(a, b),
            DistanceMetric::DotProduct => {
                -crate::distance::dot_product(a, b) // Negative for min-heap
            }
            DistanceMetric::Manhattan => crate::distance::manhattan_distance(a, b),
            DistanceMetric::Chebyshev => crate::distance::chebyshev_distance(a, b),
            DistanceMetric::Hamming => crate::distance::hamming_distance(a, b),
            DistanceMetric::Angular => crate::distance::angular_distance(a, b),
        }
    }

    pub fn dimension(&self) -> usize {
        self.dimension
    }
}

impl Index for HNSWIndex {
    fn insert(&mut self, id: u64, vector: &[f32]) {
        assert_eq!(vector.len(), self.dimension);

        let level = self.get_random_level();

        // Ensure we have enough layers
        while self.layers.len() <= level {
            self.layers.push(Layer::new());
        }

        // Store vector
        self.vectors.insert(id, vector.to_vec());

        // Insert into layers
        for layer_idx in 0..=level {
            self.layers[layer_idx].insert_node(id);
        }

        // Connect to neighbors
        if let Some(ep) = self.entry_point {
            // Search from entry point
            let mut entry_points = vec![ep];

            // Navigate from top to target layer
            for layer_idx in (level + 1..self.layers.len()).rev() {
                let nearest = self.search_layer(vector, entry_points.clone(), 1, layer_idx);
                if let Some(n) = nearest.first() {
                    entry_points = vec![n.id];
                }
            }

            // Insert at each layer
            for layer_idx in (0..=level).rev() {
                let m_max = if layer_idx == 0 {
                    self.m_max0
                } else {
                    self.m_max
                };
                let candidates = self.search_layer(
                    vector,
                    entry_points.clone(),
                    self.ef_construction,
                    layer_idx,
                );

                // Connect to M nearest neighbors
                for candidate in candidates.iter().take(self.m) {
                    self.layers[layer_idx].add_edge(id, candidate.id, m_max);
                    self.layers[layer_idx].add_edge(candidate.id, id, m_max);
                }

                if let Some(nearest) = candidates.first() {
                    entry_points = vec![nearest.id];
                }
            }
        } else {
            // First node - set as entry point
            self.entry_point = Some(id);
        }

        // Update entry point if new node is at higher level
        if level >= self.layers.len() - 1 {
            self.entry_point = Some(id);
        }
    }

    fn search(&self, query: &[f32], k: usize) -> Vec<SearchResult> {
        assert_eq!(query.len(), self.dimension);

        if let Some(ep) = self.entry_point {
            let mut entry_points = vec![ep];

            // Navigate from top layer to layer 0
            for layer_idx in (1..self.layers.len()).rev() {
                let nearest = self.search_layer(query, entry_points.clone(), 1, layer_idx);
                if let Some(n) = nearest.first() {
                    entry_points = vec![n.id];
                }
            }

            // Search at layer 0
            let ef = self.ef_construction.max(k);
            let candidates = self.search_layer(query, entry_points, ef, 0);

            // Convert to results (convert distance back to similarity score)
            candidates
                .into_iter()
                .take(k)
                .map(|c| {
                    let score = match self.metric {
                        DistanceMetric::Cosine => 1.0 - c.distance,
                        DistanceMetric::Euclidean => 1.0 / (1.0 + c.distance),
                        DistanceMetric::DotProduct => -c.distance,
                        DistanceMetric::Manhattan => 1.0 / (1.0 + c.distance),
                        DistanceMetric::Chebyshev => 1.0 / (1.0 + c.distance),
                        DistanceMetric::Hamming => 1.0 / (1.0 + c.distance),
                        DistanceMetric::Angular => 1.0 / (1.0 + c.distance),
                    };
                    SearchResult::new(c.id, score)
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    fn remove(&mut self, id: u64) -> bool {
        if self.vectors.remove(&id).is_some() {
            for layer in &mut self.layers {
                layer.remove_node(id);
            }

            // Update entry point if needed
            if self.entry_point == Some(id) {
                self.entry_point = self.vectors.keys().next().copied();
            }

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
        self.layers.clear();
        self.layers.push(Layer::new());
        self.vectors.clear();
        self.entry_point = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hnsw_basic() {
        let mut index = HNSWIndex::new(3, DistanceMetric::Cosine, 16, 200);

        let v1 = vec![1.0, 0.0, 0.0];
        let v2 = vec![0.0, 1.0, 0.0];
        let v3 = vec![1.0, 1.0, 0.0];

        index.insert(1, &v1);
        index.insert(2, &v2);
        index.insert(3, &v3);

        assert_eq!(index.len(), 3);

        let results = index.search(&v1, 2);
        assert!(results.len() > 0);
    }
}
