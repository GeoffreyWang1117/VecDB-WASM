# VecDB-WASM Quick Start Guide

Get up and running with VecDB-WASM in 5 minutes! 🚀

## Prerequisites

- Modern browser (Chrome 91+, Firefox 89+, Edge 91+, or Safari 15+)
- Basic knowledge of JavaScript
- Local web server (for development)

## Installation

### Option 1: Build from Source (Recommended for Development)

```bash
# Clone the repository
git clone https://github.com/your-org/VecDB-WASM.git
cd VecDB-WASM

# Install Rust and wasm-pack (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
cargo install wasm-pack

# Build the WASM package
wasm-pack build --target web --release

# The compiled package is now in ./pkg/
```

### Option 2: Use Pre-built Package

Download the latest release from GitHub and extract to your project:

```bash
# Your project structure
my-project/
├── index.html
└── pkg/              # Extract here
    ├── vecdb_wasm.js
    ├── vecdb_wasm_bg.wasm
    └── ...
```

## Your First Vector Database

### Step 1: Create HTML File

Create `index.html`:

```html
<!DOCTYPE html>
<html>
<head>
    <title>VecDB-WASM Quick Start</title>
</head>
<body>
    <h1>VecDB-WASM Demo</h1>
    <div id="output"></div>

    <script type="module">
        import init, { VectorDB, Metric, IndexType } from './pkg/vecdb_wasm.js';

        async function main() {
            // 1. Initialize WASM module
            await init();
            console.log('✅ WASM initialized');

            // 2. Create a vector database
            const db = new VectorDB(
                3,                 // 3-dimensional vectors
                Metric.Cosine,     // Use cosine similarity
                IndexType.Flat     // Use flat index for small datasets
            );
            console.log('✅ Database created');

            // 3. Insert some vectors
            db.insert(1, [1.0, 0.0, 0.0], '{"label": "red"}');
            db.insert(2, [0.0, 1.0, 0.0], '{"label": "green"}');
            db.insert(3, [0.0, 0.0, 1.0], '{"label": "blue"}');
            db.insert(4, [0.5, 0.5, 0.0], '{"label": "yellow"}');
            console.log(`✅ Inserted ${db.len()} vectors`);

            // 4. Search for similar vectors
            const query = [1.0, 0.1, 0.0];  // Close to red
            const results = db.search(query, 2, true);

            console.log('🔍 Search results:');
            results.forEach((result, i) => {
                const meta = JSON.parse(result.metadata);
                console.log(`  ${i + 1}. ID=${result.id}, Score=${result.score.toFixed(4)}, Label=${meta.label}`);
            });

            // Display results
            const output = document.getElementById('output');
            output.innerHTML = `
                <h2>Search Results</h2>
                <p>Query: [${query.join(', ')}]</p>
                <ul>
                    ${results.map(r => {
                        const meta = JSON.parse(r.metadata);
                        return `<li>ID: ${r.id}, Label: ${meta.label}, Score: ${r.score.toFixed(4)}</li>`;
                    }).join('')}
                </ul>
            `;
        }

        main().catch(console.error);
    </script>
</body>
</html>
```

### Step 2: Serve Locally

```bash
# Using Python 3
python3 -m http.server 8080

# Or using Node.js
npx serve .

# Or using PHP
php -S localhost:8080
```

### Step 3: Open in Browser

Navigate to http://localhost:8080

You should see:
```
Search Results
Query: [1, 0.1, 0]

• ID: 1, Label: red, Score: 0.0050
• ID: 4, Label: yellow, Score: 0.2929
```

🎉 **Congratulations!** You've created your first vector database!

---

## Common Use Cases

### Use Case 1: Semantic Search

Search text documents by meaning:

```javascript
import init, { VectorDB, Metric, IndexType } from './pkg/vecdb_wasm.js';

await init();

// Create database for 384-dimensional embeddings (e.g., from sentence-transformers)
const db = new VectorDB(384, Metric.Cosine, IndexType.HNSW);

// Insert document embeddings
const documents = [
    { id: 1, text: "The cat sat on the mat", embedding: getEmbedding("...") },
    { id: 2, text: "A dog played in the park", embedding: getEmbedding("...") },
    { id: 3, text: "The kitten slept on the rug", embedding: getEmbedding("...") }
];

documents.forEach(doc => {
    db.insert(doc.id, doc.embedding, JSON.stringify({ text: doc.text }));
});

// Search with query
const queryEmbedding = getEmbedding("feline on carpet");
const results = db.search(queryEmbedding, 3, true);

// Results will prioritize documents about cats on floor coverings
```

### Use Case 2: Image Similarity

Find similar images:

```javascript
// Create database for image embeddings (e.g., 512D from ResNet)
const db = new VectorDB(512, Metric.Euclidean, IndexType.HNSW);

// Insert image embeddings
async function addImage(id, imageUrl) {
    const embedding = await extractImageEmbedding(imageUrl);
    db.insert(id, embedding, JSON.stringify({ url: imageUrl }));
}

await addImage(1, 'cat1.jpg');
await addImage(2, 'dog1.jpg');
await addImage(3, 'cat2.jpg');

// Find similar images
const queryEmbedding = await extractImageEmbedding('query_cat.jpg');
const similar = db.search(queryEmbedding, 5, true);
```

### Use Case 3: Recommendation System

Product recommendations:

```javascript
// Create database for user/product embeddings
const db = new VectorDB(128, Metric.DotProduct, IndexType.HNSW);

// Insert product embeddings
products.forEach(product => {
    db.insert(product.id, product.embedding, JSON.stringify({
        name: product.name,
        category: product.category,
        price: product.price
    }));
});

// Get recommendations for a user
const userEmbedding = getUserEmbedding(userId);
const recommendations = db.search(userEmbedding, 10, true);

// Filter by category
const filtered = db.search_with_filter(
    userEmbedding,
    10,
    JSON.stringify({ category: "electronics" }),
    true
);
```

---

## Persistence

### Save to Browser Storage

```javascript
// Save database to IndexedDB
await db.save_to_indexeddb('my_database');
console.log('💾 Database saved');

// Later, load it back
const data = await VectorDB.load_from_indexeddb('my_database');
const db = VectorDB.import_snapshot(data);
console.log('📂 Database loaded');
```

### Export/Import Files

```javascript
// Export to binary file
const snapshot = db.export_snapshot();
const blob = new Blob([snapshot], { type: 'application/octet-stream' });
const url = URL.createObjectURL(blob);

// Create download link
const a = document.createElement('a');
a.href = url;
a.download = 'database.vecdb';
a.click();

// Import from file
const fileInput = document.createElement('input');
fileInput.type = 'file';
fileInput.onchange = async (e) => {
    const file = e.target.files[0];
    const arrayBuffer = await file.arrayBuffer();
    const data = new Uint8Array(arrayBuffer);
    const db = VectorDB.import_snapshot(data);
    console.log('📥 Database imported');
};
fileInput.click();
```

---

## Performance Optimization

### 1. Choose the Right Index

```javascript
// Small dataset (<10K vectors): Use Flat
const small_db = new VectorDB(128, Metric.Cosine, IndexType.Flat);

// Large dataset (>10K vectors): Use HNSW
const large_db = new VectorDB(128, Metric.Cosine, IndexType.HNSW);
```

### 2. Batch Insert for Speed

```javascript
// ❌ Slow
for (let i = 0; i < 10000; i++) {
    db.insert(i, vectors[i]);
}

// ✅ Fast (10-100x faster)
const batch = vectors.map((v, i) => ({ id: i, vector: v }));
db.batch_insert(batch);
```

### 3. Tune HNSW Parameters

```javascript
// Default: Good balance
const db = new VectorDB(128, Metric.Cosine, IndexType.HNSW);

// High accuracy: Better recall, slower build
const accurate_db = VectorDB.new_with_hnsw_params(128, Metric.Cosine, 32, 400);

// Fast build: Faster, lower recall
const fast_db = VectorDB.new_with_hnsw_params(128, Metric.Cosine, 8, 100);
```

### 4. Monitor Performance

```javascript
const stats = db.get_stats();
console.log('Database stats:', {
    vectors: stats.count,
    dimension: stats.dimension,
    memory_mb: (stats.memory_bytes / 1024 / 1024).toFixed(2),
    index_type: stats.index_type
});

// Benchmark search
const start = performance.now();
const results = db.search(query, 10);
const elapsed = performance.now() - start;
console.log(`Search took ${elapsed.toFixed(2)}ms`);
```

---

## Debugging Tips

### Enable Panic Hook (Development)

```javascript
import { init_panic_hook } from './pkg/vecdb_wasm.js';

// Better error messages in development
init_panic_hook();
```

### Check Browser Compatibility

```javascript
import { BrowserCompat } from './pkg/vecdb_wasm.js';

const compat = new BrowserCompat();
console.log('Browser:', compat.browser_name);
console.log('SIMD support:', compat.has_simd);
console.log('IndexedDB support:', compat.has_indexeddb);

if (!compat.is_fully_compatible()) {
    const warnings = compat.get_warnings();
    console.warn('Compatibility warnings:', warnings);
}
```

### Common Errors

**Error: "dimension mismatch"**
```javascript
// Make sure all vectors have the same dimension
const db = new VectorDB(128, Metric.Cosine, IndexType.Flat);
db.insert(1, [1, 2, 3]);  // ❌ Wrong! Expected 128D
db.insert(1, new Array(128).fill(0));  // ✅ Correct
```

**Error: "ID already exists"**
```javascript
// Each vector needs a unique ID
db.insert(1, vector1);
db.insert(1, vector2);  // ❌ Error: ID 1 already exists

// Solution: Use different IDs or remove first
db.remove(1);
db.insert(1, vector2);  // ✅ OK
```

---

## Next Steps

1. 📖 **Read the [API Documentation](API.md)** for complete reference
2. 🎨 **Explore [Examples](../examples/)** for interactive demos
3. 🚀 **Check [Deployment Guide](DEPLOYMENT.md)** for production tips
4. 🔧 **Review [DEVELOPMENT.md](../DEVELOPMENT.md)** to contribute

## Need Help?

- 📚 [Full API Reference](API.md)
- 🐛 [Report Issues](https://github.com/your-org/VecDB-WASM/issues)
- 💬 [Discussions](https://github.com/your-org/VecDB-WASM/discussions)
- 📧 Email: support@vecdb-wasm.dev

---

Happy vector searching! 🎯
