# VecDB-WASM Development Guide

## 🏗️ Architecture Overview

VecDB-WASM is structured in three main layers:

### 1. **Core Rust Layer** (`src/`)

```
src/
├── lib.rs              # Main entry point
├── storage.rs          # Vector storage and metadata
├── distance/           # Distance calculations
│   ├── mod.rs         # Scalar implementations
│   └── simd.rs        # SIMD-optimized versions
├── index/             # Index implementations
│   ├── mod.rs         # Index trait
│   ├── flat.rs        # Flat (brute-force) index
│   └── hnsw.rs        # HNSW index
└── bindings.rs        # WASM bindings for JavaScript
```

### 2. **WASM Bindings Layer**

The `bindings.rs` module exposes the following classes to JavaScript:

- `VectorDB` - Main database interface
- `IndexType` - Enum for Flat/HNSW
- `Metric` - Enum for distance metrics
- `SearchResult` - Search result structure

### 3. **Web Interface Layer** (`examples/`)

- `index.html` - Interactive demo
- `benchmark.html` - Performance benchmarks

## 🚀 Building the Project

### Prerequisites

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install wasm-pack
cargo install wasm-pack

# Add wasm32 target
rustup target add wasm32-unknown-unknown
```

### Build Commands

```bash
# Development build (faster compilation)
wasm-pack build --target web --dev

# Release build (optimized, SIMD enabled)
wasm-pack build --target web --release

# Or use the build script
chmod +x build.sh
./build.sh
```

### Enable SIMD Optimizations

SIMD is automatically enabled in release builds. To verify:

```bash
# The release build enables simd128 target feature
RUSTFLAGS='-C target-feature=+simd128' wasm-pack build --target web --release
```

## 🧪 Testing

### Run Rust Unit Tests

```bash
# Run all tests
cargo test

# Run specific test module
cargo test distance::
cargo test index::hnsw::

# Run with output
cargo test -- --nocapture
```

### Run WASM Tests

```bash
# Headless browser tests
wasm-pack test --headless --firefox

# Chrome
wasm-pack test --headless --chrome
```

### Manual Testing

```bash
# Start local server
cd examples
python3 -m http.server 8080

# Open in browser
# http://localhost:8080/index.html
# http://localhost:8080/benchmark.html
```

## 📊 Performance Benchmarks

Expected performance on modern hardware:

| Operation | Flat Index | HNSW Index | Note |
|-----------|-----------|-----------|------|
| Insert (10K vectors, 128D) | ~200ms | ~500ms | HNSW builds graph |
| Search (10K vectors, k=10) | ~5ms | ~1ms | HNSW is 5x faster |
| Search (100K vectors, k=10) | ~50ms | ~2ms | HNSW is 25x faster |
| SIMD Speedup | ~2-4x | ~2-4x | For dimension ≥ 4 |

### SIMD Performance

SIMD provides significant speedups for distance calculations:

- **Cosine Similarity**: 2-3x faster
- **Euclidean Distance**: 2-4x faster
- **Dot Product**: 2-3x faster

## 🎯 Index Types

### Flat Index

**Pros:**
- 100% recall (exact search)
- Simple implementation
- No build time
- Good for small datasets (<10K vectors)

**Cons:**
- O(n) search complexity
- Slow for large datasets

**Use cases:**
- Small datasets
- When exact results are critical
- As baseline for benchmarking

### HNSW Index

**Pros:**
- Fast approximate search
- Excellent recall (>95%)
- Scalable to millions of vectors
- O(log n) search complexity

**Cons:**
- Higher memory usage
- Build time required
- Approximate results

**Parameters:**
- `M` (default: 16) - Max connections per layer
- `ef_construction` (default: 200) - Build-time search depth

**Use cases:**
- Large datasets (>10K vectors)
- When speed matters more than perfect recall
- Production deployments

## 🔧 API Usage

### Basic Example

```javascript
import init, { VectorDB, IndexType, Metric } from './pkg/vecdb_wasm.js';

await init();

// Create database
const db = new VectorDB(128, Metric.Cosine, IndexType.HNSW);

// Insert vectors
const vector = new Float32Array(128);
for (let i = 0; i < 128; i++) {
    vector[i] = Math.random() * 2 - 1;
}

const metadata = JSON.stringify({ name: "example", tag: "test" });
db.insert(1, Array.from(vector), metadata);

// Search
const results = db.search(Array.from(vector), 10, true);
console.log(results);
// [{id: 1, score: 1.0, metadata: "{...}"}, ...]
```

### Batch Insert

```javascript
const vectors = [];
for (let i = 0; i < 1000; i++) {
    vectors.push({
        id: i,
        vector: Array.from({ length: 128 }, () => Math.random()),
        metadata: { index: i }
    });
}

const count = db.batch_insert(vectors);
console.log(`Inserted ${count} vectors`);
```

## 🐛 Debugging

### Enable Console Logging

```javascript
import { init_panic_hook } from './pkg/vecdb_wasm.js';
init_panic_hook(); // Better error messages in browser console
```

### Common Issues

**Issue: "RuntimeError: unreachable"**
- Solution: Check vector dimensions match
- Verify WASM module loaded correctly

**Issue: Slow performance**
- Solution: Use release build, not dev build
- Enable SIMD (automatic in release)
- Use HNSW for large datasets

**Issue: Memory errors**
- Solution: Reduce batch size
- Clear old data with `db.clear()`

## 📈 Optimization Tips

### 1. Choose Right Index Type

```javascript
// Small dataset (<10K): Use Flat
const db = new VectorDB(dim, metric, IndexType.Flat);

// Large dataset (>10K): Use HNSW
const db = new VectorDB(dim, metric, IndexType.HNSW);
```

### 2. Batch Operations

```javascript
// ❌ Slow: Individual inserts
for (const vec of vectors) {
    db.insert(vec.id, vec.data, null);
}

// ✅ Fast: Batch insert
db.batch_insert(vectors);
```

### 3. Vector Dimensions

- Prefer dimensions that are multiples of 4 for SIMD (e.g., 128, 256, 384)
- Common dimensions: 128, 384, 768, 1536

### 4. Search Parameters

```javascript
// Adjust k based on needs
const k = 10; // Good default
const results = db.search(query, k, false); // Set false if metadata not needed
```

## 🔬 Advanced Features

### Distance Metrics

```javascript
// Cosine similarity (default for embeddings)
Metric.Cosine

// Euclidean distance (for spatial data)
Metric.Euclidean

// Dot product (for normalized vectors)
Metric.DotProduct
```

### Performance Monitoring

```javascript
const start = performance.now();
const results = db.search(query, 10, false);
const elapsed = performance.now() - start;
console.log(`Search took ${elapsed.toFixed(2)}ms`);
```

## 📦 Distribution

### NPM Package (Future)

```bash
# Build package
wasm-pack build --target bundler --release

# Publish
wasm-pack publish
```

### CDN Usage (Future)

```html
<script type="module">
  import init, { VectorDB } from 'https://cdn.example.com/vecdb-wasm/pkg/vecdb_wasm.js';
  await init();
  // Use VectorDB...
</script>
```

## 🤝 Contributing

### Code Style

- Use `rustfmt` for Rust code: `cargo fmt`
- Use `clippy` for linting: `cargo clippy`
- Add tests for new features
- Update documentation

### Pull Request Process

1. Create feature branch
2. Implement changes with tests
3. Run `cargo test` and `cargo clippy`
4. Update CHANGELOG.md
5. Submit PR with description

## 📝 Roadmap

### Phase 4: Persistence (Next)
- [ ] IndexedDB integration
- [ ] Serialize/deserialize index
- [ ] Auto-save on close

### Phase 5: Advanced Features
- [ ] IVF (Inverted File) index
- [ ] Product Quantization
- [ ] Multi-threaded search (Web Workers)
- [ ] Streaming inserts

### Future Enhancements
- [ ] GPU acceleration (WebGPU)
- [ ] Compressed vectors
- [ ] Incremental index updates
- [ ] Distributed search

## 📚 Resources

- [HNSW Paper](https://arxiv.org/abs/1603.09320)
- [WASM SIMD Proposal](https://github.com/WebAssembly/simd)
- [wasm-bindgen Book](https://rustwasm.github.io/wasm-bindgen/)
- [Rust WASM Book](https://rustwasm.github.io/book/)
