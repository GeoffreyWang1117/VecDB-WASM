# Semantic Text Search Example

A practical demonstration of using VecDB-WASM for semantic text search.

## Overview

This example shows how to build a semantic search engine using VecDB-WASM. It includes:
- Text-to-vector conversion using simple hashing
- HNSW index for fast approximate search
- Real-time search with sub-millisecond response times
- 20 sample tech documents

## Features

- 📚 **Sample Dataset**: 20 technology-related documents
- 🔍 **Fast Search**: Sub-5ms search times
- 🎯 **Semantic Matching**: Find conceptually similar documents
- 📊 **Relevance Scores**: Cosine similarity-based scoring
- 🏷️ **Category Filtering**: Documents organized by topic

## Quick Start

```bash
# Build WASM
npm run build:dev

# Serve examples
cd examples
python3 -m http.server 8080

# Open http://localhost:8080/semantic-search/
```

## Usage

1. **Load Documents**: Click "Load Sample Documents"
2. **Search**: Enter a query (e.g., "machine learning")
3. **View Results**: See ranked results with relevance scores

## Example Queries

- "artificial intelligence and machine learning"
- "web development and programming"
- "cloud infrastructure and deployment"
- "data analysis and visualization"
- "security and encryption"
- "mobile app development"

## How It Works

### Text Vectorization

Simple hash-based vectorization:
1. Tokenize text into words
2. Hash each word to vector indices
3. Increment vector values at hashed positions
4. Add to neighboring indices for distribution
5. L2 normalize the vector

### Search Process

1. Convert query to 128D vector
2. Search HNSW index with cosine similarity
3. Return top-k most similar documents
4. Display with relevance scores

## Production Use

For production applications:
- Use pre-trained embeddings (BERT, Sentence Transformers)
- Generate embeddings server-side or client-side
- Store embeddings with documents
- Index large document collections

## Integration Example

```javascript
import { VectorDB, Metric, IndexType } from 'vecdb-wasm';

// Initialize
const db = new VectorDB(384, Metric.Cosine, IndexType.HNSW);

// Add documents
documents.forEach((doc, id) => {
    const embedding = await getEmbedding(doc.text);
    db.insert(id, embedding, JSON.stringify(doc.metadata));
});

// Search
const query Embedding = await getEmbedding(searchQuery);
const results = db.search(queryEmbedding, 10, true);
```

## Performance

- **Index Build**: ~50ms for 20 documents
- **Search Time**: <5ms per query
- **Memory**: ~100KB for 20x128D vectors

## License

MIT
