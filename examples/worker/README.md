# VecDB-WASM Web Worker Example

This example demonstrates how to use VecDB-WASM in a Web Worker for non-blocking, high-performance vector database operations.

## Why Use Web Workers?

Web Workers enable you to run VecDB operations in a background thread, preventing UI freezes during:
- Large batch inserts (thousands of vectors)
- Complex searches on large databases
- Building HNSW indexes
- Database import/export operations

## Files

- `vecdb.worker.js` - The worker script that runs VecDB operations
- `vecdb-worker-client.js` - Promise-based client wrapper for easy usage
- `index.html` - Interactive demo showing worker capabilities

## Quick Start

### 1. Build VecDB-WASM

```bash
cd ../..
npm run build:dev
```

### 2. Serve the Example

```bash
# From project root
npm run serve:examples
```

### 3. Open in Browser

Navigate to: `http://localhost:8080/examples/worker/`

## Usage

### Basic Example

```javascript
import { VectorDBWorker } from './vecdb-worker-client.js';

// Create worker instance
const db = new VectorDBWorker('./vecdb.worker.js');

// Initialize database (128D vectors, Cosine similarity, HNSW index)
await db.create(128, 0, 1);

// Insert vectors (non-blocking!)
await db.insert(1, new Float32Array(128), JSON.stringify({ tag: 'example' }));

// Search (non-blocking!)
const results = await db.search(queryVector, 10, true);
console.log('Found:', results);

// Get performance metrics
const metrics = await db.getPerformanceMetrics();
console.log('Avg search time:', metrics.avg_search_time_ms, 'ms');

// Clean up when done
db.terminate();
```

### Batch Operations

```javascript
// Batch insert (great for large datasets)
const vectors = [];
for (let i = 0; i < 10000; i++) {
    vectors.push({
        id: i,
        vector: generateRandomVector(128),
        metadata: { index: i }
    });
}

// This won't block the UI!
const count = await db.batchInsert(vectors);
console.log(`Inserted ${count} vectors`);
```

### Advanced Search

```javascript
// Search with metadata filter
const results = await db.searchWithFilter(
    queryVector,
    10,
    JSON.stringify({ category: 'tech', status: 'active' }),
    true
);

// Radius search
const nearby = await db.searchRadius(
    queryVector,
    0.5,  // radius
    100,  // max results
    true
);

// Batch search multiple queries
const queries = [vector1, vector2, vector3];
const allResults = await db.batchSearch(queries, 10);
```

### Persistence

```javascript
// Save to IndexedDB
await db.saveToIndexedDB('my_vectors');

// Later... load from IndexedDB
await db.loadFromIndexedDB('my_vectors');

// List saved databases
const databases = await db.listSavedDatabases();
console.log('Saved databases:', databases);
```

## API

The `VectorDBWorker` class provides a Promise-based API that mirrors the main VectorDB API:

### Database Creation
- `create(dimension, metric, indexType)` - Create database
- `createWithHNSWParams(dimension, metric, m, ef_construction)` - Create with custom HNSW params

### Vector Operations
- `insert(id, vector, metadata)` - Insert single vector
- `batchInsert(vectors)` - Insert multiple vectors
- `remove(id)` - Remove vector
- `getVector(id)` - Retrieve vector
- `clear()` - Remove all vectors

### Search Operations
- `search(query, k, includeMetadata)` - k-NN search
- `searchWithFilter(query, k, filter, includeMetadata)` - Filtered search
- `batchSearch(queries, k, includeMetadata)` - Search multiple queries
- `searchRadius(query, radius, maxResults, includeMetadata)` - Radius search

### Database Info
- `len()` - Get vector count
- `isEmpty()` - Check if empty
- `dimension()` - Get dimension
- `getStats()` - Get database statistics
- `getPerformanceMetrics()` - Get performance metrics
- `resetPerformanceMetrics()` - Reset metrics

### Persistence
- `exportSnapshot()` - Export to binary
- `exportSnapshotJson()` - Export to JSON
- `importSnapshot(data)` - Import from binary
- `importSnapshotJson(json)` - Import from JSON
- `saveToIndexedDB(dbName)` - Save to browser storage
- `loadFromIndexedDB(dbName)` - Load from browser storage
- `deleteFromIndexedDB(dbName)` - Delete from browser storage
- `listSavedDatabases()` - List saved databases

### Worker Management
- `terminate()` - Terminate the worker

## Performance Tips

1. **Use batch operations** when inserting many vectors
2. **Enable HNSW index** for databases with >10K vectors
3. **Monitor metrics** to identify bottlenecks
4. **Use workers** for operations on large datasets (>1K vectors)
5. **Adjust HNSW parameters** (`m`, `ef_construction`) for your use case

## Browser Compatibility

- Chrome 80+
- Firefox 79+
- Safari 15.4+
- Edge 80+

All browsers must support:
- Web Workers
- WebAssembly
- IndexedDB (for persistence)

## Error Handling

```javascript
try {
    await db.search(query, 10);
} catch (error) {
    console.error('Search failed:', error);
}
```

All methods return Promises that reject on error.

## Cleanup

Always terminate the worker when you're done to free resources:

```javascript
// When component unmounts or page unloads
db.terminate();
```

## Integration with React

```javascript
import { useEffect, useState } from 'react';
import { VectorDBWorker } from './vecdb-worker-client';

function useVectorDB() {
    const [db, setDb] = useState(null);

    useEffect(() => {
        const worker = new VectorDBWorker();
        worker.create(128, 0, 1).then(() => {
            setDb(worker);
        });

        return () => {
            worker.terminate();
        };
    }, []);

    return db;
}

// Usage in component
function MyComponent() {
    const db = useVectorDB();

    const handleSearch = async () => {
        if (db) {
            const results = await db.search(query, 10);
            // ...
        }
    };

    return <button onClick={handleSearch}>Search</button>;
}
```

## License

MIT
