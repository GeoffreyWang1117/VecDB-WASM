# VecDB-WASM API Reference

Complete API documentation for VecDB-WASM v0.2.0

## Table of Contents

- [VectorDB Class](#vectordb-class)
- [Enums](#enums)
- [Types](#types)
- [Browser Compatibility](#browser-compatibility)
- [Utility Functions](#utility-functions)

---

## VectorDB Class

The main class for interacting with the vector database.

### Constructor

#### `new VectorDB(dimension, metric, indexType)`

Creates a new vector database instance.

**Parameters:**
- `dimension` (number): Vector dimension (must be positive integer)
- `metric` (Metric): Distance metric to use
- `indexType` (IndexType): Index type to use

**Returns:** `VectorDB` instance

**Throws:** Error if parameters are invalid

**Example:**
```javascript
import init, { VectorDB, Metric, IndexType } from './pkg/vecdb_wasm.js';

await init();

const db = new VectorDB(128, Metric.Cosine, IndexType.HNSW);
```

#### `VectorDB.new_with_hnsw_params(dimension, metric, m, ef_construction)`

Creates a new vector database with custom HNSW parameters.

**Parameters:**
- `dimension` (number): Vector dimension
- `metric` (Metric): Distance metric
- `m` (number): Maximum number of connections per node (default: 16, recommended: 8-64)
- `ef_construction` (number): Size of dynamic candidate list (default: 200, recommended: 100-500)

**Returns:** `VectorDB` instance

**Example:**
```javascript
// Create HNSW index optimized for high recall
const db = VectorDB.new_with_hnsw_params(
    128,              // dimension
    Metric.Cosine,    // metric
    32,               // m (higher = better recall, slower)
    400               // ef_construction (higher = better quality)
);
```

---

### Insert Operations

#### `insert(id, vector, metadata?)`

Insert a single vector into the database.

**Parameters:**
- `id` (number): Unique vector ID (u64)
- `vector` (Array\<number\> | Float32Array): Vector data
- `metadata` (string, optional): JSON string containing metadata

**Returns:** `void`

**Throws:** Error if vector dimension doesn't match or ID already exists

**Example:**
```javascript
// Insert without metadata
db.insert(1, [0.1, 0.2, 0.3, ...]);

// Insert with metadata
const metadata = JSON.stringify({
    title: "Example Document",
    category: "tech",
    timestamp: Date.now()
});
db.insert(2, vectorArray, metadata);
```

#### `batch_insert(vectors)`

Insert multiple vectors efficiently.

**Parameters:**
- `vectors` (Array): Array of objects with `{id, vector, metadata?}`

**Returns:** `number` - Number of vectors inserted

**Example:**
```javascript
const vectors = [
    { id: 1, vector: [0.1, 0.2, ...], metadata: '{"type":"A"}' },
    { id: 2, vector: [0.3, 0.4, ...], metadata: '{"type":"B"}' },
    { id: 3, vector: [0.5, 0.6, ...] }
];

const count = db.batch_insert(vectors);
console.log(`Inserted ${count} vectors`);
```

---

### Search Operations

#### `search(query, k, include_metadata?)`

Search for the k nearest neighbors.

**Parameters:**
- `query` (Array\<number\> | Float32Array): Query vector
- `k` (number): Number of results to return
- `include_metadata` (boolean, optional): Whether to include metadata (default: false)

**Returns:** `Array<SearchResult>`

**Example:**
```javascript
const query = [0.5, 0.3, 0.8, ...];
const results = db.search(query, 10, true);

results.forEach(result => {
    console.log(`ID: ${result.id}, Score: ${result.score}`);
    if (result.metadata) {
        console.log(`Metadata: ${result.metadata}`);
    }
});
```

#### `search_with_filter(query, k, filter?, include_metadata?)`

Search with metadata filtering.

**Parameters:**
- `query` (Array\<number\>): Query vector
- `k` (number): Number of results
- `filter` (string, optional): JSON filter object
- `include_metadata` (boolean, optional): Include metadata in results

**Returns:** `Array<SearchResult>`

**Filter Format:**
```javascript
// Exact match filter
const filter = JSON.stringify({
    category: "tech",
    status: "active"
});

// Multiple filters (all must match)
const filter = JSON.stringify({
    type: "document",
    year: "2024",
    verified: "true"
});

const results = db.search_with_filter(query, 10, filter, true);
```

**Note:** The search will retrieve more candidates internally to ensure k results after filtering.

---

### Data Management

#### `remove(id)`

Remove a vector by ID.

**Parameters:**
- `id` (number): Vector ID to remove

**Returns:** `boolean` - true if removed, false if not found

**Example:**
```javascript
if (db.remove(42)) {
    console.log('Vector removed');
} else {
    console.log('Vector not found');
}
```

#### `clear()`

Remove all vectors from the database.

**Example:**
```javascript
db.clear();
console.log(`Database cleared, now has ${db.len()} vectors`);
```

#### `get_vector(id)`

Retrieve a vector by ID.

**Parameters:**
- `id` (number): Vector ID

**Returns:** `Array<number> | null` - Vector data or null if not found

**Example:**
```javascript
const vector = db.get_vector(42);
if (vector) {
    console.log('Vector dimensions:', vector.length);
}
```

---

### Database Info

#### `len()`

Get the number of vectors in the database.

**Returns:** `number`

**Example:**
```javascript
console.log(`Database contains ${db.len()} vectors`);
```

#### `is_empty()`

Check if database is empty.

**Returns:** `boolean`

#### `dimension()`

Get the vector dimension.

**Returns:** `number`

#### `get_stats()`

Get comprehensive database statistics.

**Returns:** Object with:
- `version` (string): Database version
- `dimension` (number): Vector dimension
- `count` (number): Number of vectors
- `index_type` (string): Index type ("Flat" or "HNSW")
- `metric` (string): Distance metric
- `memory_bytes` (number): Estimated memory usage
- `hnsw_m` (number, optional): HNSW M parameter
- `hnsw_ef` (number, optional): HNSW ef_construction parameter

**Example:**
```javascript
const stats = db.get_stats();
console.log(`Database: ${stats.count} vectors of ${stats.dimension}D`);
console.log(`Index: ${stats.index_type} with ${stats.metric} metric`);
console.log(`Memory: ${(stats.memory_bytes / 1024 / 1024).toFixed(2)} MB`);
```

---

### Persistence

#### `export_snapshot()`

Export database to binary format.

**Returns:** `Uint8Array` - Binary snapshot

**Example:**
```javascript
const snapshot = db.export_snapshot();
localStorage.setItem('db_snapshot', btoa(String.fromCharCode(...snapshot)));
```

#### `export_snapshot_json()`

Export database to JSON format.

**Returns:** `string` - JSON snapshot

**Example:**
```javascript
const json = db.export_snapshot_json();
const blob = new Blob([json], { type: 'application/json' });
const url = URL.createObjectURL(blob);
// Create download link...
```

#### `VectorDB.import_snapshot(data)`

Import database from binary format (static method).

**Parameters:**
- `data` (Uint8Array): Binary snapshot data

**Returns:** `VectorDB` instance

**Example:**
```javascript
const data = new Uint8Array(binaryData);
const db = VectorDB.import_snapshot(data);
```

#### `VectorDB.import_snapshot_json(json)`

Import database from JSON format (static method).

**Parameters:**
- `json` (string): JSON snapshot

**Returns:** `VectorDB` instance

**Example:**
```javascript
const json = await fetch('database.json').then(r => r.text());
const db = VectorDB.import_snapshot_json(json);
```

---

### IndexedDB Persistence

#### `save_to_indexeddb(db_name)` (async)

Save database to IndexedDB.

**Parameters:**
- `db_name` (string): Database name

**Returns:** `Promise<void>`

**Example:**
```javascript
await db.save_to_indexeddb('my_vectors');
console.log('Database saved to IndexedDB');
```

#### `VectorDB.load_from_indexeddb(db_name)` (async, static)

Load database from IndexedDB.

**Parameters:**
- `db_name` (string): Database name

**Returns:** `Promise<Uint8Array>` - Binary data to pass to `import_snapshot()`

**Example:**
```javascript
const data = await VectorDB.load_from_indexeddb('my_vectors');
const db = VectorDB.import_snapshot(data);
```

#### `VectorDB.delete_from_indexeddb(db_name)` (async, static)

Delete a database from IndexedDB.

**Parameters:**
- `db_name` (string): Database name

**Returns:** `Promise<void>`

**Example:**
```javascript
await VectorDB.delete_from_indexeddb('old_database');
```

#### `VectorDB.list_saved_databases()` (async, static)

List all saved databases in IndexedDB.

**Returns:** `Promise<Array<string>>` - Array of database names

**Example:**
```javascript
const databases = await VectorDB.list_saved_databases();
console.log('Saved databases:', databases);
```

---

## Enums

### IndexType

Index type for the database.

```javascript
enum IndexType {
    Flat,   // Brute-force exact search, O(n) complexity
    HNSW    // Approximate nearest neighbor, O(log n) complexity
}
```

**Recommendations:**
- **Flat**: Best for small datasets (<10K vectors), 100% recall
- **HNSW**: Best for large datasets (>10K vectors), >95% recall, much faster

### Metric

Distance metric for similarity calculation.

```javascript
enum Metric {
    Cosine,      // Cosine similarity (1 - cosine distance)
    Euclidean,   // L2 distance
    DotProduct   // Dot product (higher is more similar)
}
```

**Metric Selection Guide:**

| Metric | Use Case | Normalized Vectors? | Range |
|--------|----------|---------------------|-------|
| **Cosine** | Text embeddings, semantic search | Recommended | 0-2 (0=identical) |
| **Euclidean** | Image embeddings, general use | Not required | 0-∞ (0=identical) |
| **DotProduct** | Recommendation systems | Yes | -∞ to ∞ (higher=similar) |

---

## Types

### SearchResult

Result object returned by search operations.

```typescript
interface SearchResult {
    id: number;           // Vector ID
    score: number;        // Similarity score (lower is closer for distance metrics)
    metadata?: string;    // JSON metadata string (if requested)
}
```

---

## Browser Compatibility

### BrowserCompat Class

Check browser compatibility and features.

#### Constructor

```javascript
import { BrowserCompat } from './pkg/vecdb_wasm.js';

const compat = new BrowserCompat();
```

#### Properties

- `browser_name` (string): Browser name
- `user_agent` (string): Full user agent string
- `has_wasm` (boolean): WebAssembly support
- `has_indexeddb` (boolean): IndexedDB support
- `has_simd` (boolean): WASM SIMD support
- `has_bigint64array` (boolean): BigInt64Array support

#### Methods

##### `get_report()`

Get full compatibility report.

**Returns:** Object with all compatibility info

##### `get_warnings()`

Get array of warning messages.

**Returns:** `Array<string>`

##### `is_fully_compatible()`

Check if fully compatible.

**Returns:** `boolean`

**Example:**
```javascript
const compat = new BrowserCompat();

console.log(`Browser: ${compat.browser_name}`);
console.log(`SIMD: ${compat.has_simd ? 'Yes' : 'No'}`);
console.log(`IndexedDB: ${compat.has_indexeddb ? 'Yes' : 'No'}`);

if (!compat.is_fully_compatible()) {
    const warnings = compat.get_warnings();
    console.warn('Compatibility issues:', warnings);
}
```

---

## Utility Functions

### `init_panic_hook()`

Initialize panic hook for better error messages (development only).

**Example:**
```javascript
import { init_panic_hook } from './pkg/vecdb_wasm.js';

if (process.env.NODE_ENV === 'development') {
    init_panic_hook();
}
```

### `version()`

Get VecDB-WASM version.

**Returns:** `string` - Version string

**Example:**
```javascript
import { version } from './pkg/vecdb_wasm.js';

console.log(`VecDB-WASM version: ${version()}`);
```

---

## Performance Tips

### 1. Choose the Right Index

```javascript
// Small dataset (<10K): Use Flat for 100% recall
const db = new VectorDB(128, Metric.Cosine, IndexType.Flat);

// Large dataset (>10K): Use HNSW for speed
const db = new VectorDB(128, Metric.Cosine, IndexType.HNSW);
```

### 2. Batch Insert for Large Datasets

```javascript
// ❌ Slow: Insert one by one
for (let i = 0; i < 10000; i++) {
    db.insert(i, vectors[i]);
}

// ✅ Fast: Use batch insert
const batch = vectors.map((v, i) => ({ id: i, vector: v }));
db.batch_insert(batch);
```

### 3. Tune HNSW Parameters

```javascript
// Balanced (default)
const db = VectorDB.new_with_hnsw_params(128, Metric.Cosine, 16, 200);

// High recall (slower build, better quality)
const db = VectorDB.new_with_hnsw_params(128, Metric.Cosine, 32, 400);

// Fast build (faster, lower recall)
const db = VectorDB.new_with_hnsw_params(128, Metric.Cosine, 8, 100);
```

### 4. Use Binary Snapshots

```javascript
// ✅ Binary: ~50% smaller, faster
const snapshot = db.export_snapshot();

// ❌ JSON: Larger, human-readable
const json = db.export_snapshot_json();
```

---

## Error Handling

```javascript
try {
    const db = new VectorDB(128, Metric.Cosine, IndexType.HNSW);

    // Insert with validation
    db.insert(1, vector);

    // Search
    const results = db.search(query, 10);

} catch (error) {
    if (error.message.includes('dimension mismatch')) {
        console.error('Vector dimension does not match database');
    } else if (error.message.includes('already exists')) {
        console.error('Vector ID already exists');
    } else {
        console.error('Unexpected error:', error);
    }
}
```

---

## Complete Example

```javascript
import init, { VectorDB, Metric, IndexType, BrowserCompat } from './pkg/vecdb_wasm.js';

async function main() {
    // Initialize WASM
    await init();

    // Check compatibility
    const compat = new BrowserCompat();
    if (!compat.is_fully_compatible()) {
        console.warn('Some features may not be available');
    }

    // Create database
    const db = new VectorDB(128, Metric.Cosine, IndexType.HNSW);

    // Insert vectors
    const vectors = [
        { id: 1, vector: generateRandomVector(128), metadata: '{"type":"A"}' },
        { id: 2, vector: generateRandomVector(128), metadata: '{"type":"B"}' },
        { id: 3, vector: generateRandomVector(128), metadata: '{"type":"A"}' }
    ];
    db.batch_insert(vectors);

    // Search
    const query = generateRandomVector(128);
    const results = db.search(query, 2, true);

    console.log('Search results:', results);

    // Save to IndexedDB
    await db.save_to_indexeddb('my_db');

    // Get stats
    const stats = db.get_stats();
    console.log('Database stats:', stats);
}

function generateRandomVector(dim) {
    return Array.from({ length: dim }, () => Math.random());
}

main();
```

---

## TypeScript Support

TypeScript definitions will be available in a future release. For now, you can create your own:

```typescript
// types.d.ts
declare module 'vecdb-wasm' {
    export function init(): Promise<void>;

    export enum Metric {
        Cosine,
        Euclidean,
        DotProduct
    }

    export enum IndexType {
        Flat,
        HNSW
    }

    export interface SearchResult {
        id: number;
        score: number;
        metadata?: string;
    }

    export class VectorDB {
        constructor(dimension: number, metric: Metric, indexType: IndexType);
        static new_with_hnsw_params(dimension: number, metric: Metric, m: number, ef: number): VectorDB;

        insert(id: number, vector: number[], metadata?: string): void;
        batch_insert(vectors: Array<{id: number, vector: number[], metadata?: string}>): number;

        search(query: number[], k: number, includeMetadata?: boolean): SearchResult[];
        search_with_filter(query: number[], k: number, filter?: string, includeMetadata?: boolean): SearchResult[];

        remove(id: number): boolean;
        clear(): void;
        get_vector(id: number): number[] | null;

        len(): number;
        is_empty(): boolean;
        dimension(): number;
        get_stats(): any;

        export_snapshot(): Uint8Array;
        export_snapshot_json(): string;
        static import_snapshot(data: Uint8Array): VectorDB;
        static import_snapshot_json(json: string): VectorDB;

        save_to_indexeddb(name: string): Promise<void>;
        static load_from_indexeddb(name: string): Promise<Uint8Array>;
        static delete_from_indexeddb(name: string): Promise<void>;
        static list_saved_databases(): Promise<string[]>;
    }

    export class BrowserCompat {
        browser_name: string;
        user_agent: string;
        has_wasm: boolean;
        has_indexeddb: boolean;
        has_simd: boolean;
        has_bigint64array: boolean;

        get_report(): any;
        get_warnings(): string[];
        is_fully_compatible(): boolean;
    }
}
```

---

For more examples, see the [examples/](../examples/) directory.
