# VecDB-WASM Optimization Guide

Comprehensive guide to optimizing VecDB-WASM for production use.

## Package Size Optimization

### Current Size Analysis

| Build Type | Size | Gzipped |
|------------|------|---------|
| Dev Build | ~500KB | ~150KB |
| Release Build | ~200KB | ~60KB |
| Optimized Build | ~150KB | ~45KB |

### Optimization Levels

#### 1. Basic Optimization (Default)
```bash
npm run build
```
- Uses `--release` flag
- LTO enabled
- `opt-level = 3`

#### 2. Advanced Optimization
```bash
npm run build:optimized
```
- Release build + wasm-opt
- `-Oz` flag (optimize for size)
- Reduces size by ~25%

#### 3. Ultra Optimization
```toml
# Cargo.toml
[profile.release]
opt-level = "z"          # Optimize for size
lto = true               # Link-time optimization
codegen-units = 1        # Better optimization
panic = 'abort'          # Remove panic machinery
strip = true             # Strip debug symbols
```

### Size Reduction Techniques

#### 1. Remove Unused Features
```bash
cargo build --release --no-default-features
```

#### 2. Use wasm-opt
```bash
wasm-opt -Oz -o output.wasm input.wasm
```

#### 3. Enable Brotli Compression
```javascript
// Serve with Brotli compression
Content-Encoding: br
// Can reduce size by additional 20-30%
```

## Performance Optimization

### Memory Usage

#### Current Memory Profile

| Operation | Memory | Notes |
|-----------|--------|-------|
| Empty DB | ~1MB | Baseline |
| 10K vectors (128D) | ~6MB | 5MB for vectors |
| 100K vectors (128D) | ~52MB | 50MB for vectors |
| HNSW overhead | ~20% | Graph structure |

#### Optimization Tips

1. **Use Appropriate Dimensions**
   - Lower dimensions = less memory
   - 128D is sweet spot for most use cases
   - Avoid unnecessary high dimensions

2. **Batch Operations**
   - Use `batch_insert()` instead of individual inserts
   - 10-100x faster for large datasets
   - Less memory fragmentation

3. **Clear Unused Data**
   - Call `clear()` when done
   - Remove unused vectors with `remove()`

### Search Performance

#### Current Performance

| Configuration | Time | Throughput |
|--------------|------|------------|
| Flat 10K, k=10 | 2-5ms | 200-500 ops/s |
| HNSW 10K, k=10 | 0.5-1ms | 1000-2000 ops/s |
| HNSW 100K, k=10 | 1-2ms | 500-1000 ops/s |

#### Optimization Strategies

1. **Use HNSW Index**
   - 4-10x faster than Flat
   - Scales better with dataset size
   - Small accuracy trade-off (>95% recall)

2. **Tune HNSW Parameters**
   ```javascript
   // Faster build, slightly lower accuracy
   new VectorDB(dim, metric, IndexType.HNSW, {
       m: 8,              // Default: 16
       ef_construction: 100  // Default: 200
   });

   // Higher accuracy, slower build
   new VectorDB(dim, metric, IndexType.HNSW, {
       m: 32,
       ef_construction: 400
   });
   ```

3. **Choose Right Metric**
   - Cosine: Best for normalized vectors (text embeddings)
   - Euclidean: General purpose
   - Manhattan/Chebyshev: Faster computation
   - Dot Product: Unnormalized vectors

4. **Use Web Workers**
   - Prevents UI blocking
   - Parallel processing
   - See `examples/worker/`

### CPU Optimization

#### SIMD Acceleration

Automatic SIMD optimization for:
- Distance calculations
- Vector normalization
- Batch operations

Requirements:
- Chrome 91+, Firefox 89+, Safari 16.4+
- Vectors dimension ≥ 4

Performance gain: **2-4x speedup**

#### Compiler Optimizations

```toml
[profile.release]
opt-level = 3            # Maximum optimization
lto = "fat"             # Full LTO
codegen-units = 1       # Better optimization
```

## Browser Optimization

### Loading Strategy

#### Lazy Loading
```javascript
// Load WASM only when needed
async function initWhenNeeded() {
    if (!dbInitialized) {
        await init();
        db = new VectorDB(dim, metric, indexType);
        dbInitialized = true;
    }
}
```

#### Preloading
```html
<!-- Preload WASM module -->
<link rel="preload" href="vecdb_wasm_bg.wasm" as="fetch" crossorigin>
```

### Caching Strategy

```javascript
// Service Worker caching
self.addEventListener('install', (event) => {
    event.waitUntil(
        caches.open('vecdb-v1').then((cache) => {
            return cache.addAll([
                '/pkg/vecdb_wasm.js',
                '/pkg/vecdb_wasm_bg.wasm'
            ]);
        })
    );
});
```

## Production Checklist

### Build Configuration

- [ ] Use release build: `npm run build:optimized`
- [ ] Enable LTO and size optimization
- [ ] Strip debug symbols
- [ ] Run wasm-opt with `-Oz`

### Compression

- [ ] Enable Brotli compression (server)
- [ ] Enable Gzip as fallback
- [ ] Set proper Cache-Control headers
- [ ] Use CDN for static assets

### Performance

- [ ] Use HNSW index for datasets >1K
- [ ] Implement Web Workers for large operations
- [ ] Add performance monitoring
- [ ] Test on target browsers

### Memory Management

- [ ] Monitor memory usage
- [ ] Implement cleanup on page unload
- [ ] Use batch operations
- [ ] Clear unused data

## Benchmarking

### Run Benchmarks

```bash
# Rust benchmarks
cargo bench

# Browser benchmarks
npm run serve:examples
# Open http://localhost:8080/benchmark-suite/
```

### Measure Real Performance

```javascript
const metrics = db.get_performance_metrics();
console.log(`Avg search time: ${metrics.avg_search_time_ms}ms`);
console.log(`Total searches: ${metrics.total_searches}`);
```

## Monitoring

### Performance Metrics

```javascript
// Track performance
setInterval(() => {
    const metrics = db.get_performance_metrics();
    sendToAnalytics({
        avgSearchTime: metrics.avg_search_time_ms,
        totalSearches: metrics.total_searches,
        peakTime: metrics.peak_search_time_ms
    });
}, 60000); // Every minute
```

### Memory Monitoring

```javascript
if (performance.memory) {
    console.log('Used JS heap:',
        (performance.memory.usedJSHeapSize / 1024 / 1024).toFixed(2), 'MB');
}
```

## Common Issues

### Large Package Size

**Problem**: WASM bundle is too large

**Solutions**:
1. Use `build:optimized` script
2. Enable Brotli compression
3. Remove unused features
4. Consider code splitting

### Slow Search

**Problem**: Search takes >10ms

**Solutions**:
1. Use HNSW index instead of Flat
2. Reduce dataset size or dimension
3. Use Web Workers
4. Check browser SIMD support

### High Memory Usage

**Problem**: Memory usage grows over time

**Solutions**:
1. Call `clear()` when resetting
2. Remove unused vectors
3. Avoid storing large metadata
4. Monitor with performance APIs

## Advanced Optimizations

### Custom Allocator

For extreme performance, consider custom allocator:
```toml
[dependencies]
wee_alloc = "0.4"

[features]
default = ["wee_alloc"]
```

### Profile-Guided Optimization

```bash
# Generate profile data
cargo pgo build

# Use profile for optimization
cargo pgo optimize
```

### Link-Time Optimization

Already enabled by default, but can tune:
```toml
[profile.release]
lto = "fat"  # vs "thin" - slower build, better optimization
```

## Resources

- [Rust WASM Book](https://rustwasm.github.io/book/)
- [wasm-opt Documentation](https://github.com/WebAssembly/binaryen)
- [Web Performance](https://web.dev/performance/)

## Performance Targets

| Metric | Target | Current |
|--------|--------|---------|
| Package size (gzipped) | <50KB | ~45KB |
| Initialization time | <100ms | ~50ms |
| Search 10K vectors | <2ms | ~1ms |
| Memory (10K vectors) | <10MB | ~6MB |
| SIMD speedup | 2-4x | 2-4x |

---

Last updated: 2025-11-17
Version: 0.3.0
