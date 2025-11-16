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

## [0.2.0] - 2025-11-16

### Added

#### Persistence & Data Management
- **IndexedDB Integration**
  - Save databases directly to browser storage with `save_to_indexeddb()`
  - Load databases from IndexedDB with `load_from_indexeddb()`
  - List all saved databases with `list_saved_databases()`
  - Delete databases with `delete_from_indexeddb()`
  - Asynchronous Promise-based API
  - Enhanced error handling for browser compatibility

- **Snapshot Import/Export**
  - Binary snapshot format using bincode serialization
  - JSON snapshot format for human-readable exports
  - `export_snapshot()` and `import_snapshot()` for binary format
  - `export_snapshot_json()` and `import_snapshot_json()` for JSON format
  - Efficient serialization with minimal overhead
  - Version tracking in snapshots

#### Advanced Search Features
- **Metadata Filtering**
  - `search_with_filter()` method for filtered searches
  - JSON-based filter format
  - Automatic candidate expansion for filtered queries
  - Supports arbitrary metadata key-value matching
  - Works with both HNSW and Flat indices

- **Customizable HNSW Parameters**
  - `new_with_hnsw_params()` constructor
  - Configure M (connections per node)
  - Configure ef_construction (search quality during build)
  - Fine-tune recall vs performance tradeoff

#### Browser Compatibility
- **Feature Detection System**
  - `BrowserCompat` class for runtime capability detection
  - Automatic browser identification (Chrome, Firefox, Safari, Edge, Opera)
  - IndexedDB availability checking
  - WASM support verification
  - SIMD feature detection
  - BigInt64Array support checking

- **Compatibility Reporting**
  - `get_report()` method for full compatibility status
  - `get_warnings()` for browser-specific warnings
  - `is_fully_compatible()` quick check
  - User-agent parsing and analysis

- **Enhanced Error Messages**
  - Clear messages when IndexedDB unavailable
  - Guidance for private/incognito mode
  - Browser-specific recommendations
  - Graceful degradation suggestions

#### Statistics & Monitoring
- **Database Statistics**
  - `get_stats()` method returning comprehensive info
  - Version tracking
  - Vector count and dimension
  - Index type and metric information
  - HNSW parameters (M, ef_construction)
  - Estimated memory usage in bytes

#### Web Interface
- **Persistence Demo** (`examples/persistence.html`)
  - Interactive IndexedDB save/load testing
  - Export/import demonstration
  - Metadata filtering examples
  - Database listing and management
  - Real-time statistics display

- **Compatibility Checker** (`examples/compatibility.html`)
  - Browser feature detection UI
  - Visual compatibility matrix
  - Performance testing suite
  - Warning and recommendation display
  - Real-time capability reporting

#### Documentation
- **Browser Compatibility Guide** (BROWSER_COMPATIBILITY.md)
  - Comprehensive browser support matrix
  - SIMD enablement instructions for Chrome/Edge/Firefox
  - Feature detection examples
  - Graceful degradation patterns
  - Private browsing mode handling
  - Performance expectations by browser
  - Known issues and workarounds
  - Mobile browser considerations

- **Updated README**
  - Browser compatibility table
  - Persistence API documentation
  - Advanced search examples
  - Statistics API reference
  - Updated feature count (15+ tests)

### Technical Details

#### New Dependencies
- bincode ^1.3 - Binary serialization
- base64 ^0.21 - Base64 encoding for snapshots
- wasm-bindgen-futures ^0.4 - Async/await support

#### New Modules
- `persistence/snapshot.rs` - Serialization logic
- `persistence/indexeddb.rs` - IndexedDB wrapper
- `compat.rs` - Browser compatibility detection

#### Web-sys Features Added
- Navigator - User-agent detection
- IdbFactory, IdbDatabase - IndexedDB core
- IdbObjectStore, IdbTransaction - Storage operations
- IdbRequest, IdbOpenDbRequest - Async operations
- IdbVersionChangeEvent - Database versioning
- DomStringList, DomException - Error handling

### Testing
- Increased test coverage to 15+ unit tests
- New tests for persistence layer
- Snapshot serialization tests
- Metadata filtering tests
- Browser compatibility detection tests

### Performance
- Binary snapshots: ~50% smaller than JSON
- IndexedDB operations: <100ms for typical databases
- Metadata filtering: Minimal performance impact (<10%)
- SIMD speedup maintained across all browsers with support

### Browser Support

| Browser | Version | SIMD | IndexedDB | Status |
|---------|---------|------|-----------|--------|
| Chrome | 91+ | ✅ | ✅ | ✅ Fully Supported |
| Edge | 91+ | ✅ | ✅ | ✅ Fully Supported |
| Firefox | 89+ | ✅ | ✅ | ✅ Fully Supported |
| Safari | 15+ | ⚠️ | ✅ | ⚠️ Partial (no SIMD) |
| Opera | 77+ | ✅ | ✅ | ✅ Fully Supported |

**Note**: SIMD provides 2-4x performance boost. Browsers without SIMD use scalar fallback.

## [Unreleased]

### Planned for v0.3.0
- IVF (Inverted File) index implementation
- Product Quantization for compression
- Web Workers parallelism for batch operations
- React-based full UI component
- Range search queries

### Future Enhancements
- GPU acceleration via WebGPU
- Incremental index updates
- Multi-vector batch search
- Hybrid search (vector + text)
