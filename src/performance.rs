use serde::{Deserialize, Serialize};
use std::time::Instant;

/// Performance metrics for monitoring database operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Total number of search operations performed
    pub total_searches: u64,
    /// Total search time in milliseconds
    pub total_search_time_ms: f64,
    /// Average search time in milliseconds
    pub avg_search_time_ms: f64,
    /// Peak search time in milliseconds
    pub peak_search_time_ms: f64,
    /// Total number of insert operations
    pub total_inserts: u64,
    /// Total insert time in milliseconds
    pub total_insert_time_ms: f64,
    /// Average insert time in milliseconds
    pub avg_insert_time_ms: f64,
    /// Total number of batch operations
    pub total_batch_ops: u64,
    /// Total number of vectors in batch operations
    pub total_batch_vectors: u64,
    /// Total batch operation time in milliseconds
    pub total_batch_time_ms: f64,
    /// Average batch operation time in milliseconds
    pub avg_batch_time_ms: f64,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            total_searches: 0,
            total_search_time_ms: 0.0,
            avg_search_time_ms: 0.0,
            peak_search_time_ms: 0.0,
            total_inserts: 0,
            total_insert_time_ms: 0.0,
            avg_insert_time_ms: 0.0,
            total_batch_ops: 0,
            total_batch_vectors: 0,
            total_batch_time_ms: 0.0,
            avg_batch_time_ms: 0.0,
        }
    }
}

impl PerformanceMetrics {
    /// Create a new empty performance metrics tracker
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a search operation
    pub fn record_search(&mut self, duration_ms: f64) {
        self.total_searches += 1;
        self.total_search_time_ms += duration_ms;
        self.avg_search_time_ms = self.total_search_time_ms / self.total_searches as f64;

        if duration_ms > self.peak_search_time_ms {
            self.peak_search_time_ms = duration_ms;
        }
    }

    /// Record an insert operation
    pub fn record_insert(&mut self, duration_ms: f64) {
        self.total_inserts += 1;
        self.total_insert_time_ms += duration_ms;
        self.avg_insert_time_ms = self.total_insert_time_ms / self.total_inserts as f64;
    }

    /// Record a batch operation
    pub fn record_batch(&mut self, vector_count: usize, duration_ms: f64) {
        self.total_batch_ops += 1;
        self.total_batch_vectors += vector_count as u64;
        self.total_batch_time_ms += duration_ms;
        self.avg_batch_time_ms = self.total_batch_time_ms / self.total_batch_ops as f64;
    }

    /// Reset all metrics to zero
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Get total operations count
    pub fn total_operations(&self) -> u64 {
        self.total_searches + self.total_inserts + self.total_batch_ops
    }

    /// Get operations per second (requires elapsed time tracking)
    pub fn searches_per_second(&self, elapsed_seconds: f64) -> f64 {
        if elapsed_seconds > 0.0 {
            self.total_searches as f64 / elapsed_seconds
        } else {
            0.0
        }
    }
}

/// Timer for measuring operation duration
pub struct OperationTimer {
    start: Instant,
}

impl OperationTimer {
    /// Start a new timer
    pub fn start() -> Self {
        Self {
            start: Instant::now(),
        }
    }

    /// Get elapsed time in milliseconds
    pub fn elapsed_ms(&self) -> f64 {
        self.start.elapsed().as_secs_f64() * 1000.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_metrics_initialization() {
        let metrics = PerformanceMetrics::new();
        assert_eq!(metrics.total_searches, 0);
        assert_eq!(metrics.total_inserts, 0);
        assert_eq!(metrics.avg_search_time_ms, 0.0);
    }

    #[test]
    fn test_record_search() {
        let mut metrics = PerformanceMetrics::new();
        metrics.record_search(10.0);
        metrics.record_search(20.0);

        assert_eq!(metrics.total_searches, 2);
        assert_eq!(metrics.total_search_time_ms, 30.0);
        assert_eq!(metrics.avg_search_time_ms, 15.0);
        assert_eq!(metrics.peak_search_time_ms, 20.0);
    }

    #[test]
    fn test_record_insert() {
        let mut metrics = PerformanceMetrics::new();
        metrics.record_insert(5.0);
        metrics.record_insert(15.0);

        assert_eq!(metrics.total_inserts, 2);
        assert_eq!(metrics.total_insert_time_ms, 20.0);
        assert_eq!(metrics.avg_insert_time_ms, 10.0);
    }

    #[test]
    fn test_record_batch() {
        let mut metrics = PerformanceMetrics::new();
        metrics.record_batch(100, 50.0);
        metrics.record_batch(200, 100.0);

        assert_eq!(metrics.total_batch_ops, 2);
        assert_eq!(metrics.total_batch_vectors, 300);
        assert_eq!(metrics.total_batch_time_ms, 150.0);
        assert_eq!(metrics.avg_batch_time_ms, 75.0);
    }

    #[test]
    fn test_reset() {
        let mut metrics = PerformanceMetrics::new();
        metrics.record_search(10.0);
        metrics.record_insert(5.0);

        metrics.reset();

        assert_eq!(metrics.total_searches, 0);
        assert_eq!(metrics.total_inserts, 0);
        assert_eq!(metrics.total_search_time_ms, 0.0);
    }

    #[test]
    fn test_total_operations() {
        let mut metrics = PerformanceMetrics::new();
        metrics.record_search(10.0);
        metrics.record_insert(5.0);
        metrics.record_batch(100, 50.0);

        assert_eq!(metrics.total_operations(), 3);
    }

    #[test]
    fn test_operation_timer() {
        let timer = OperationTimer::start();
        thread::sleep(Duration::from_millis(10));
        let elapsed = timer.elapsed_ms();

        // Should be at least 10ms, but allow some tolerance
        assert!(elapsed >= 9.0);
    }
}
