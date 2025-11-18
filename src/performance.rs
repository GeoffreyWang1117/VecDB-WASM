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
    /// Search time samples for percentile calculation (limited to last N samples)
    #[serde(skip)]
    search_samples: Vec<f64>,
    /// Insert time samples for percentile calculation
    #[serde(skip)]
    insert_samples: Vec<f64>,
    /// Maximum samples to keep
    #[serde(skip)]
    max_samples: usize,
    /// Slow query threshold in milliseconds
    #[serde(skip)]
    slow_query_threshold: f64,
    /// Slow query log (limited to last N entries)
    #[serde(skip)]
    slow_queries: Vec<SlowQuery>,
}

/// Slow query entry
#[derive(Debug, Clone)]
pub struct SlowQuery {
    pub operation: String,
    pub duration_ms: f64,
    pub timestamp: u64,
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
            search_samples: Vec::with_capacity(1000),
            insert_samples: Vec::with_capacity(1000),
            max_samples: 1000,
            slow_query_threshold: 100.0, // 100ms default
            slow_queries: Vec::new(),
        }
    }
}

impl PerformanceMetrics {
    /// Create a new empty performance metrics tracker
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with custom configuration
    pub fn with_config(max_samples: usize, slow_query_threshold: f64) -> Self {
        Self {
            search_samples: Vec::with_capacity(max_samples),
            insert_samples: Vec::with_capacity(max_samples),
            max_samples,
            slow_query_threshold,
            ..Default::default()
        }
    }

    /// Set slow query threshold
    pub fn set_slow_query_threshold(&mut self, threshold_ms: f64) {
        self.slow_query_threshold = threshold_ms;
    }

    /// Record a search operation
    pub fn record_search(&mut self, duration_ms: f64) {
        self.total_searches += 1;
        self.total_search_time_ms += duration_ms;
        self.avg_search_time_ms = self.total_search_time_ms / self.total_searches as f64;

        if duration_ms > self.peak_search_time_ms {
            self.peak_search_time_ms = duration_ms;
        }

        // Store sample for percentile calculation
        self.add_search_sample(duration_ms);

        // Log slow query
        if duration_ms >= self.slow_query_threshold {
            self.log_slow_query("search".to_string(), duration_ms);
        }
    }

    /// Record an insert operation
    pub fn record_insert(&mut self, duration_ms: f64) {
        self.total_inserts += 1;
        self.total_insert_time_ms += duration_ms;
        self.avg_insert_time_ms = self.total_insert_time_ms / self.total_inserts as f64;

        // Store sample
        self.add_insert_sample(duration_ms);

        // Log slow query
        if duration_ms >= self.slow_query_threshold {
            self.log_slow_query("insert".to_string(), duration_ms);
        }
    }

    /// Record a batch operation
    pub fn record_batch(&mut self, vector_count: usize, duration_ms: f64) {
        self.total_batch_ops += 1;
        self.total_batch_vectors += vector_count as u64;
        self.total_batch_time_ms += duration_ms;
        self.avg_batch_time_ms = self.total_batch_time_ms / self.total_batch_ops as f64;

        // Log slow batch operation
        if duration_ms >= self.slow_query_threshold {
            self.log_slow_query(format!("batch({})", vector_count), duration_ms);
        }
    }

    /// Add search sample
    fn add_search_sample(&mut self, duration_ms: f64) {
        if self.search_samples.len() >= self.max_samples {
            self.search_samples.remove(0);
        }
        self.search_samples.push(duration_ms);
    }

    /// Add insert sample
    fn add_insert_sample(&mut self, duration_ms: f64) {
        if self.insert_samples.len() >= self.max_samples {
            self.insert_samples.remove(0);
        }
        self.insert_samples.push(duration_ms);
    }

    /// Log slow query
    fn log_slow_query(&mut self, operation: String, duration_ms: f64) {
        let query = SlowQuery {
            operation,
            duration_ms,
            timestamp: self.total_operations(),
        };

        if self.slow_queries.len() >= 100 {
            self.slow_queries.remove(0);
        }
        self.slow_queries.push(query);
    }

    /// Calculate percentile from samples
    fn calculate_percentile(samples: &[f64], percentile: f64) -> f64 {
        if samples.is_empty() {
            return 0.0;
        }

        let mut sorted = samples.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let index = (percentile / 100.0 * (sorted.len() - 1) as f64).round() as usize;
        sorted[index.min(sorted.len() - 1)]
    }

    /// Get search p50 (median)
    pub fn search_p50(&self) -> f64 {
        Self::calculate_percentile(&self.search_samples, 50.0)
    }

    /// Get search p95
    pub fn search_p95(&self) -> f64 {
        Self::calculate_percentile(&self.search_samples, 95.0)
    }

    /// Get search p99
    pub fn search_p99(&self) -> f64 {
        Self::calculate_percentile(&self.search_samples, 99.0)
    }

    /// Get insert p50 (median)
    pub fn insert_p50(&self) -> f64 {
        Self::calculate_percentile(&self.insert_samples, 50.0)
    }

    /// Get insert p95
    pub fn insert_p95(&self) -> f64 {
        Self::calculate_percentile(&self.insert_samples, 95.0)
    }

    /// Get insert p99
    pub fn insert_p99(&self) -> f64 {
        Self::calculate_percentile(&self.insert_samples, 99.0)
    }

    /// Get slow queries
    pub fn get_slow_queries(&self) -> &[SlowQuery] {
        &self.slow_queries
    }

    /// Get slow query count
    pub fn slow_query_count(&self) -> usize {
        self.slow_queries.len()
    }

    /// Clear slow query log
    pub fn clear_slow_queries(&mut self) {
        self.slow_queries.clear();
    }

    /// Reset all metrics to zero
    pub fn reset(&mut self) {
        *self = Self::with_config(self.max_samples, self.slow_query_threshold);
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

    /// Export metrics in Prometheus format
    pub fn to_prometheus(&self, prefix: &str) -> String {
        let mut output = String::new();

        // Counters
        output.push_str(&format!("# HELP {}_searches_total Total number of search operations\n", prefix));
        output.push_str(&format!("# TYPE {}_searches_total counter\n", prefix));
        output.push_str(&format!("{}_searches_total {}\n\n", prefix, self.total_searches));

        output.push_str(&format!("# HELP {}_inserts_total Total number of insert operations\n", prefix));
        output.push_str(&format!("# TYPE {}_inserts_total counter\n", prefix));
        output.push_str(&format!("{}_inserts_total {}\n\n", prefix, self.total_inserts));

        // Gauges - averages
        output.push_str(&format!("# HELP {}_search_duration_ms_avg Average search duration in milliseconds\n", prefix));
        output.push_str(&format!("# TYPE {}_search_duration_ms_avg gauge\n", prefix));
        output.push_str(&format!("{}_search_duration_ms_avg {}\n\n", prefix, self.avg_search_time_ms));

        // Histograms - percentiles
        output.push_str(&format!("# HELP {}_search_duration_ms Search duration percentiles in milliseconds\n", prefix));
        output.push_str(&format!("# TYPE {}_search_duration_ms summary\n", prefix));
        output.push_str(&format!("{}_search_duration_ms{{quantile=\"0.5\"}} {}\n", prefix, self.search_p50()));
        output.push_str(&format!("{}_search_duration_ms{{quantile=\"0.95\"}} {}\n", prefix, self.search_p95()));
        output.push_str(&format!("{}_search_duration_ms{{quantile=\"0.99\"}} {}\n", prefix, self.search_p99()));
        output.push_str(&format!("{}_search_duration_ms_sum {}\n", prefix, self.total_search_time_ms));
        output.push_str(&format!("{}_search_duration_ms_count {}\n\n", prefix, self.total_searches));

        // Slow queries
        output.push_str(&format!("# HELP {}_slow_queries_total Total number of slow queries\n", prefix));
        output.push_str(&format!("# TYPE {}_slow_queries_total counter\n", prefix));
        output.push_str(&format!("{}_slow_queries_total {}\n", prefix, self.slow_query_count()));

        output
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
