# Movie Recommendation System

A practical demonstration of building a recommendation engine using VecDB-WASM vector database.

## Overview

This example demonstrates content-based filtering using vector similarity. It showcases how to:
- Convert items (movies) to feature vectors
- Use cosine similarity for recommendations
- Build user profiles from selections
- Generate personalized recommendations in real-time

## Features

- 🎬 **50 Movies**: Diverse dataset spanning multiple genres
- ⚡ **Fast Recommendations**: Sub-5ms recommendation generation
- 🎯 **Content-Based Filtering**: Based on genres, ratings, and year
- 💡 **Explainable**: Shows why movies are recommended
- 🔍 **HNSW Index**: Efficient similarity search

## Quick Start

```bash
# Build WASM
npm run build:dev

# Serve examples
cd examples
python3 -m http.server 8080

# Open http://localhost:8080/recommendation/
```

## Usage

1. **Select Movies**: Click on movies you like (minimum 1)
2. **Get Recommendations**: Click the button to generate recommendations
3. **View Results**: See top 10 recommended movies with similarity scores

## How It Works

### Feature Engineering

Each movie is converted to a 32-dimensional vector:

```javascript
// Genre features (18 dimensions)
[1, 0, 1, 0, ...] // Binary encoding for each genre

// Rating feature (1 dimension)
0.87 // Normalized rating (8.7/10)

// Year feature (1 dimension)
0.5 // Normalized year

// Padding (12 dimensions)
[0, 0, 0, ...] // For better distribution
```

### Recommendation Algorithm

1. **Profile Building**:
   ```javascript
   // Average vectors of selected movies
   userProfile = mean(selectedMovies.map(m => movieToVector(m)))
   ```

2. **Similarity Search**:
   ```javascript
   // Find k most similar movies using cosine similarity
   recommendations = db.search(userProfile, k=10)
   ```

3. **Filtering**:
   ```javascript
   // Remove already selected movies
   filtered = recommendations.filter(r => !selected.has(r.id))
   ```

### Cosine Similarity

Measures angle between vectors, perfect for recommendation:
- Range: 0-1 (1 = identical, 0 = completely different)
- Normalized, handles different rating scales
- Fast computation with HNSW index

## Architecture

```
User Selections → Feature Vector → HNSW Search → Recommendations
    [Movies]        [32D Vector]    [Cosine Sim]    [Top-K Results]
```

## Performance

- **Index Build**: ~20ms for 50 movies
- **Recommendation Generation**: <5ms
- **Memory**: ~50KB for 50x32D vectors
- **Accuracy**: Content-based, deterministic

## Extending the Example

### Add More Features

```javascript
function movieToVector(movie) {
    return [
        ...genreEncoding(movie.genres),
        movie.rating / 10,
        (movie.year - 1960) / 65,
        movie.duration / 200,        // NEW: Duration
        movie.budget / 100000000,    // NEW: Budget
        directorEncoding(movie.director) // NEW: Director
    ];
}
```

### Hybrid Filtering

Combine content-based with collaborative filtering:

```javascript
// Content similarity
const contentSim = db.search(userProfile, k);

// User similarity (collaborative)
const userSim = db.search(userVector, k);

// Combine scores
const hybrid = contentSim.map((item, idx) => ({
    ...item,
    score: 0.7 * item.score + 0.3 * userSim[idx].score
}));
```

### Real-time Updates

```javascript
// Add new movies dynamically
function addMovie(movie) {
    const vector = movieToVector(movie);
    db.insert(movie.id, vector, JSON.stringify(movie.metadata));
}

// Update recommendations in real-time
watchSelections((selected) => {
    if (selected.length > 0) {
        updateRecommendations();
    }
});
```

## Production Use Cases

### E-commerce Product Recommendations

```javascript
// Product features
function productToVector(product) {
    return [
        ...categoryEncoding(product.category),
        ...priceRange(product.price),
        ...brandEncoding(product.brand),
        product.rating / 5,
        product.popularity
    ];
}
```

### Music Recommendations

```javascript
// Song features
function songToVector(song) {
    return [
        ...genreEncoding(song.genres),
        song.tempo / 200,
        song.energy,
        song.valence,
        song.danceability,
        ...artistEncoding(song.artist)
    ];
}
```

### Article Recommendations

```javascript
// Article features from embeddings
async function articleToVector(article) {
    // Use pre-trained text embeddings (e.g., Sentence-BERT)
    const embedding = await getEmbedding(article.content);
    return embedding; // 384D or 768D
}
```

## Integration Example

```javascript
import { VectorDB, Metric, IndexType } from 'vecdb-wasm';

class RecommendationEngine {
    constructor(dimension = 32) {
        this.db = new VectorDB(dimension, Metric.Cosine, IndexType.HNSW);
        this.items = new Map();
    }

    addItem(id, features, metadata) {
        const vector = this.featuresToVector(features);
        this.db.insert(id, vector, JSON.stringify(metadata));
        this.items.set(id, { features, metadata });
    }

    recommend(userProfile, k = 10, excludeIds = []) {
        const profileVector = this.featuresToVector(userProfile);
        const results = this.db.search(profileVector, k + excludeIds.length, true);

        return results
            .filter(r => !excludeIds.includes(r.id))
            .slice(0, k)
            .map(r => ({
                id: r.id,
                score: r.score,
                item: this.items.get(r.id)
            }));
    }

    featuresToVector(features) {
        // Implement your feature encoding
        return features;
    }
}

// Usage
const engine = new RecommendationEngine();

// Add items
products.forEach(p => {
    engine.addItem(p.id, extractFeatures(p), p);
});

// Get recommendations
const recommendations = engine.recommend(
    userPreferences,
    k = 10,
    excludeIds = purchasedIds
);
```

## Evaluation Metrics

### Precision@K
```javascript
function precisionAtK(recommended, relevant, k) {
    const topK = recommended.slice(0, k);
    const hits = topK.filter(id => relevant.has(id)).length;
    return hits / k;
}
```

### Diversity
```javascript
function diversity(recommendations) {
    const genres = recommendations.map(r => r.metadata.genres);
    const uniqueGenres = new Set(genres.flat());
    return uniqueGenres.size / genres.length;
}
```

## Best Practices

1. **Feature Normalization**: Normalize all features to [0, 1] range
2. **Dimensionality**: Keep dimensions reasonable (32-512)
3. **Index Selection**: Use HNSW for datasets >1K items
4. **Cold Start**: Provide diverse initial recommendations
5. **Explainability**: Always show why items are recommended

## Performance Tuning

```javascript
// Adjust HNSW parameters for accuracy/speed trade-off
const db = new VectorDB(
    dimension,
    Metric.Cosine,
    IndexType.HNSW,
    {
        m: 16,              // Higher = better accuracy, slower build
        ef_construction: 200 // Higher = better quality, slower build
    }
);
```

## License

MIT
