# VecDB-WASM Benchmark Suite

Comprehensive performance testing tool for VecDB-WASM vector database.

## Features

- **Distance Functions Benchmark**: Test all 7 distance metrics
- **Scaling Test**: Performance across dataset sizes (1K - 100K vectors)
- **Dimension Test**: Performance across dimensions (64D - 1024D)
- **Index Comparison**: Flat vs HNSW performance
- **Batch Operations**: Batch insert and search testing
- **Metric Comparison**: Compare all metrics on same dataset
- **Export Results**: Save benchmark results as JSON

## Quick Start

```bash
# From project root
npm run build:dev

# Serve examples
npm run serve:examples

# Open http://localhost:8080/benchmark-suite/
```

## Running Benchmarks

### Individual Benchmarks

Click any benchmark card to run that specific test:
- Distance Functions
- Scaling Test
- Dimension Test
- Index Comparison
- Batch Operations
- Metric Comparison

### Run All

Click "Run All Benchmarks" to execute all tests sequentially and generate a comprehensive report.

## Configuration

Customize test parameters:
- **Dataset Size**: 1K, 10K, 50K, 100K vectors
- **Vector Dimension**: 64D, 128D, 256D, 512D, 1024D
- **Search K**: Number of nearest neighbors
- **Warmup Runs**: Number of warmup iterations

## Results

Results include:
- Execution time (ms)
- Throughput (ops/s)
- Detailed metrics for each test
- Comparative analysis

## Export

Export results as JSON for further analysis or comparison across different environments/browsers.

## Rust Benchmarks

For native Rust benchmarks:

```bash
cargo bench
```

Results will be saved to `target/criterion/`.

## Performance Tips

1. Close other browser tabs for accurate results
2. Run multiple times and average results
3. Test in different browsers for comparison
4. Use same hardware for fair comparisons

## License

MIT
