/// Query result caching module
///
/// Provides LRU (Least Recently Used) cache for search results to improve
/// performance of repeated queries

use std::collections::{HashMap, VecDeque};
use std::hash::{Hash, Hasher};

/// Cached search result
#[derive(Debug, Clone)]
pub struct CachedResult {
    pub id: u64,
    pub score: f32,
}

/// Cache key for query results
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CacheKey {
    query_hash: u64,
    k: usize,
}

impl CacheKey {
    fn new(query: &[f32], k: usize) -> Self {
        Self {
            query_hash: hash_vector(query),
            k,
        }
    }
}

/// Hash a vector to u64 for cache key
fn hash_vector(vector: &[f32]) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();

    // Hash the bytes of the vector
    for &value in vector {
        value.to_bits().hash(&mut hasher);
    }

    hasher.finish()
}

/// LRU cache entry
#[derive(Debug, Clone)]
struct CacheEntry {
    results: Vec<CachedResult>,
    access_count: u64,
    last_access: u64,
}

/// LRU cache for query results
#[derive(Clone, Debug)]
pub struct QueryCache {
    cache: HashMap<CacheKey, CacheEntry>,
    access_order: VecDeque<CacheKey>,
    max_size: usize,
    enabled: bool,
    hits: u64,
    misses: u64,
    evictions: u64,
    access_counter: u64,
}

impl QueryCache {
    /// Create a new query cache
    ///
    /// # Arguments
    /// * `max_size` - Maximum number of cached queries (0 = disabled)
    pub fn new(max_size: usize) -> Self {
        Self {
            cache: HashMap::new(),
            access_order: VecDeque::new(),
            max_size,
            enabled: max_size > 0,
            hits: 0,
            misses: 0,
            evictions: 0,
            access_counter: 0,
        }
    }

    /// Enable or disable the cache
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled && self.max_size > 0;
        if !self.enabled {
            self.clear();
        }
    }

    /// Check if cache is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Set maximum cache size
    pub fn set_max_size(&mut self, max_size: usize) {
        self.max_size = max_size;
        self.enabled = max_size > 0;

        // Evict entries if cache is now too large
        while self.cache.len() > max_size {
            self.evict_lru();
        }
    }

    /// Get maximum cache size
    pub fn max_size(&self) -> usize {
        self.max_size
    }

    /// Get cached result if available
    pub fn get(&mut self, query: &[f32], k: usize) -> Option<Vec<CachedResult>> {
        if !self.enabled {
            return None;
        }

        let key = CacheKey::new(query, k);

        if let Some(entry) = self.cache.get_mut(&key) {
            // Cache hit
            self.hits += 1;
            self.access_counter += 1;
            entry.access_count += 1;
            entry.last_access = self.access_counter;

            // Move to end of access order (most recently used)
            if let Some(pos) = self.access_order.iter().position(|k| k == &key) {
                self.access_order.remove(pos);
            }
            self.access_order.push_back(key);

            Some(entry.results.clone())
        } else {
            // Cache miss
            self.misses += 1;
            None
        }
    }

    /// Store query result in cache
    pub fn put(&mut self, query: &[f32], k: usize, results: Vec<CachedResult>) {
        if !self.enabled {
            return;
        }

        let key = CacheKey::new(query, k);
        self.access_counter += 1;

        // If key already exists, update it
        if self.cache.contains_key(&key) {
            if let Some(entry) = self.cache.get_mut(&key) {
                entry.results = results;
                entry.access_count += 1;
                entry.last_access = self.access_counter;

                // Move to end
                if let Some(pos) = self.access_order.iter().position(|k| k == &key) {
                    self.access_order.remove(pos);
                }
                self.access_order.push_back(key);
            }
            return;
        }

        // Evict if at capacity
        if self.cache.len() >= self.max_size {
            self.evict_lru();
        }

        // Insert new entry
        let entry = CacheEntry {
            results,
            access_count: 1,
            last_access: self.access_counter,
        };

        self.cache.insert(key.clone(), entry);
        self.access_order.push_back(key);
    }

    /// Evict least recently used entry
    fn evict_lru(&mut self) {
        if let Some(key) = self.access_order.pop_front() {
            self.cache.remove(&key);
            self.evictions += 1;
        }
    }

    /// Clear all cached entries
    pub fn clear(&mut self) {
        self.cache.clear();
        self.access_order.clear();
        self.evictions += self.cache.len() as u64;
    }

    /// Invalidate cache entries (when index is modified)
    pub fn invalidate(&mut self) {
        self.clear();
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            size: self.cache.len(),
            max_size: self.max_size,
            hits: self.hits,
            misses: self.misses,
            evictions: self.evictions,
            hit_rate: if self.hits + self.misses > 0 {
                self.hits as f64 / (self.hits + self.misses) as f64
            } else {
                0.0
            },
            enabled: self.enabled,
        }
    }

    /// Reset statistics
    pub fn reset_stats(&mut self) {
        self.hits = 0;
        self.misses = 0;
        self.evictions = 0;
    }

    /// Get current cache size
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }
}

impl Default for QueryCache {
    fn default() -> Self {
        Self::new(100) // Default cache size: 100 queries
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub size: usize,
    pub max_size: usize,
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub hit_rate: f64,
    pub enabled: bool,
}

impl CacheStats {
    /// Get total requests
    pub fn total_requests(&self) -> u64 {
        self.hits + self.misses
    }

    /// Get cache efficiency score (0-100)
    pub fn efficiency_score(&self) -> f64 {
        self.hit_rate * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_creation() {
        let cache = QueryCache::new(10);
        assert_eq!(cache.max_size(), 10);
        assert!(cache.is_enabled());
        assert!(cache.is_empty());
    }

    #[test]
    fn test_cache_get_put() {
        let mut cache = QueryCache::new(10);
        let query = vec![1.0, 2.0, 3.0];
        let results = vec![CachedResult { id: 1, score: 0.9 }];

        // Miss on first get
        assert!(cache.get(&query, 5).is_none());

        // Put and get
        cache.put(&query, 5, results.clone());
        let cached = cache.get(&query, 5);
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().len(), 1);

        let stats = cache.stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
    }

    #[test]
    fn test_cache_eviction() {
        let mut cache = QueryCache::new(2);

        cache.put(&vec![1.0], 5, vec![CachedResult { id: 1, score: 0.9 }]);
        cache.put(&vec![2.0], 5, vec![CachedResult { id: 2, score: 0.8 }]);
        cache.put(&vec![3.0], 5, vec![CachedResult { id: 3, score: 0.7 }]);

        assert_eq!(cache.len(), 2);
        assert_eq!(cache.stats().evictions, 1);

        // First query should be evicted
        assert!(cache.get(&vec![1.0], 5).is_none());
        // Last two should be available
        assert!(cache.get(&vec![2.0], 5).is_some());
        assert!(cache.get(&vec![3.0], 5).is_some());
    }

    #[test]
    fn test_cache_clear() {
        let mut cache = QueryCache::new(10);
        cache.put(&vec![1.0], 5, vec![CachedResult { id: 1, score: 0.9 }]);
        cache.put(&vec![2.0], 5, vec![CachedResult { id: 2, score: 0.8 }]);

        assert_eq!(cache.len(), 2);
        cache.clear();
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_cache_disabled() {
        let mut cache = QueryCache::new(0); // Disabled
        assert!(!cache.is_enabled());

        cache.put(&vec![1.0], 5, vec![CachedResult { id: 1, score: 0.9 }]);
        assert!(cache.get(&vec![1.0], 5).is_none());
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_cache_hit_rate() {
        let mut cache = QueryCache::new(10);
        let query = vec![1.0, 2.0];
        let results = vec![CachedResult { id: 1, score: 0.9 }];

        cache.get(&query, 5); // Miss
        cache.put(&query, 5, results);
        cache.get(&query, 5); // Hit
        cache.get(&query, 5); // Hit

        let stats = cache.stats();
        assert_eq!(stats.hits, 2);
        assert_eq!(stats.misses, 1);
        assert!((stats.hit_rate - 0.666).abs() < 0.01);
    }

    #[test]
    fn test_different_k_values() {
        let mut cache = QueryCache::new(10);
        let query = vec![1.0, 2.0];

        cache.put(&query, 5, vec![CachedResult { id: 1, score: 0.9 }]);
        cache.put(&query, 10, vec![CachedResult { id: 2, score: 0.8 }]);

        // Different k values should be separate cache entries
        assert!(cache.get(&query, 5).is_some());
        assert!(cache.get(&query, 10).is_some());
        assert_eq!(cache.len(), 2);
    }
}
