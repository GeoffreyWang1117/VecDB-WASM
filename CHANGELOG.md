# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2025-11-16

### Added

#### Core Infrastructure
- Initial project setup with Rust + WASM architecture
- Modular codebase structure (storage, distance, index, bindings)
- Comprehensive test suite with 12+ unit tests
- MIT License

#### Data Structures
- `VectorStorage` for managing vectors and metadata
- `VectorMetadata` with flexible key-value metadata
- Type-safe dimension validation
- Efficient HashMap-based storage

#### Distance Metrics
- Cosine similarity calculation
- Euclidean distance calculation
- Dot product calculation
- Generic `calculate_distance` with metric selection
- SIMD-optimized versions for all distance functions
- Automatic SIMD dispatch for vectors with dimension ≥ 4
- 2-4x speedup with WASM SIMD128

#### Index Implementations
- **Flat Index** (baseline)
  - Brute-force exact search
  - O(n) complexity
  - 100% recall
  - Efficient for small datasets (<10K vectors)

- **HNSW Index** (advanced)
  - Hierarchical navigable small world graph
  - Approximate nearest neighbor search
  - O(log n) search complexity
  - Configurable M and ef_construction parameters
  - Excellent recall (>95%) with fast search

#### WASM Bindings
- `VectorDB` class for JavaScript
- `IndexType` enum (Flat, HNSW)
- `Metric` enum (Cosine, Euclidean, DotProduct)
- `SearchResult` structure with scores and metadata
- Batch insert API for efficient data loading
- Error handling with descriptive messages
- Optional metadata inclusion in search results

#### Web Interface
- Interactive demo UI (`examples/index.html`)
  - Beautiful gradient design
  - Real-time statistics display
  - Vector generation and search
  - Metadata visualization
  - Console logging

- Performance benchmark suite (`examples/benchmark.html`)
  - Insert performance testing
  - Search latency measurement
  - Scalability analysis
  - Visual performance charts
  - Detailed metrics table
  - QPS (Queries Per Second) tracking

#### Build & Development
- `package.json` with NPM scripts
- `build.sh` for easy compilation
- Cargo.toml with optimized release profile
- LTO and single codegen unit for size optimization
- WASM-pack integration
- Development and release build configurations

#### Documentation
- Comprehensive README with architecture diagram
- Development guide (DEVELOPMENT.md)
- Inline code documentation
- API usage examples
- Performance benchmarks
- Testing instructions

### Performance

#### Benchmarks (Release Build)
- Insert: ~2000 vectors/sec (128D, HNSW)
- Search: 1-5ms for 10K vectors (k=10)
- SIMD Speedup: 2-4x for distance calculations
- Memory: Efficient storage with minimal overhead

### Technical Details

#### Dependencies
- wasm-bindgen ^0.2
- serde ^1.0 (with derive)
- serde-wasm-bindgen ^0.6
- js-sys ^0.3
- web-sys ^0.3
- rand ^0.8
- getrandom ^0.2 (with js feature)

#### Optimizations
- SIMD128 for parallel distance calculations
- Max-heap for efficient top-k selection
- Release profile with LTO enabled
- Single codegen unit for size reduction
- Lazy metadata serialization

### Testing
- 12 unit tests covering all modules
- Distance calculation tests
- Index operation tests (insert, search, remove)
- Storage validation tests
- SIMD correctness verification
- WASM browser tests ready

## [Unreleased]

### Planned for v0.2.0
- IndexedDB persistence layer
- Serialize/deserialize index state
- Web Workers for parallel search
- Enhanced metadata filtering
- Range search queries

### Future Enhancements
- IVF (Inverted File) index
- Product Quantization for compression
- GPU acceleration via WebGPU
- Incremental index updates
- Multi-vector batch search
