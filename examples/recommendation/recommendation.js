import init, { VectorDB, Metric, IndexType } from '../../pkg/vecdb_wasm.js';

let db = null;
let selectedMovies = new Set();

// Sample movie database with genres and features
const movies = [
    { id: 0, title: "The Matrix", genres: ["Sci-Fi", "Action"], rating: 8.7, icon: "🤖", year: 1999 },
    { id: 1, title: "Inception", genres: ["Sci-Fi", "Thriller"], rating: 8.8, icon: "💭", year: 2010 },
    { id: 2, title: "Interstellar", genres: ["Sci-Fi", "Drama"], rating: 8.6, icon: "🚀", year: 2014 },
    { id: 3, title: "The Shawshank Redemption", genres: ["Drama"], rating: 9.3, icon: "🎭", year: 1994 },
    { id: 4, title: "The Godfather", genres: ["Crime", "Drama"], rating: 9.2, icon: "👔", year: 1972 },
    { id: 5, title: "Pulp Fiction", genres: ["Crime", "Drama"], rating: 8.9, icon: "🔫", year: 1994 },
    { id: 6, title: "The Dark Knight", genres: ["Action", "Crime"], rating: 9.0, icon: "🦇", year: 2008 },
    { id: 7, title: "Forrest Gump", genres: ["Drama", "Romance"], rating: 8.8, icon: "🏃", year: 1994 },
    { id: 8, title: "Fight Club", genres: ["Drama"], rating: 8.8, icon: "👊", year: 1999 },
    { id: 9, title: "The Lord of the Rings", genres: ["Fantasy", "Adventure"], rating: 8.9, icon: "💍", year: 2001 },
    { id: 10, title: "Star Wars", genres: ["Sci-Fi", "Adventure"], rating: 8.6, icon: "⭐", year: 1977 },
    { id: 11, title: "Gladiator", genres: ["Action", "Drama"], rating: 8.5, icon: "⚔️", year: 2000 },
    { id: 12, title: "Titanic", genres: ["Romance", "Drama"], rating: 7.9, icon: "🚢", year: 1997 },
    { id: 13, title: "Jurassic Park", genres: ["Sci-Fi", "Adventure"], rating: 8.2, icon: "🦖", year: 1993 },
    { id: 14, title: "Avatar", genres: ["Sci-Fi", "Action"], rating: 7.9, icon: "🌳", year: 2009 },
    { id: 15, title: "The Avengers", genres: ["Action", "Sci-Fi"], rating: 8.0, icon: "🦸", year: 2012 },
    { id: 16, title: "The Silence of the Lambs", genres: ["Crime", "Thriller"], rating: 8.6, icon: "🐑", year: 1991 },
    { id: 17, title: "Saving Private Ryan", genres: ["War", "Drama"], rating: 8.6, icon: "🎖️", year: 1998 },
    { id: 18, title: "The Green Mile", genres: ["Drama", "Fantasy"], rating: 8.6, icon: "💚", year: 1999 },
    { id: 19, title: "Schindler's List", genres: ["Drama", "History"], rating: 9.0, icon: "📜", year: 1993 },
    { id: 20, title: "Goodfellas", genres: ["Crime", "Drama"], rating: 8.7, icon: "🍝", year: 1990 },
    { id: 21, title: "The Departed", genres: ["Crime", "Thriller"], rating: 8.5, icon: "🚔", year: 2006 },
    { id: 22, title: "The Prestige", genres: ["Drama", "Mystery"], rating: 8.5, icon: "🎩", year: 2006 },
    { id: 23, title: "Memento", genres: ["Mystery", "Thriller"], rating: 8.4, icon: "🧠", year: 2000 },
    { id: 24, title: "The Lion King", genres: ["Animation", "Drama"], rating: 8.5, icon: "🦁", year: 1994 },
    { id: 25, title: "Toy Story", genres: ["Animation", "Comedy"], rating: 8.3, icon: "🤠", year: 1995 },
    { id: 26, title: "Finding Nemo", genres: ["Animation", "Adventure"], rating: 8.2, icon: "🐠", year: 2003 },
    { id: 27, title: "The Shining", genres: ["Horror", "Drama"], rating: 8.4, icon: "🏨", year: 1980 },
    { id: 28, title: "Alien", genres: ["Horror", "Sci-Fi"], rating: 8.5, icon: "👽", year: 1979 },
    { id: 29, title: "Terminator 2", genres: ["Sci-Fi", "Action"], rating: 8.6, icon: "🤖", year: 1991 },
    { id: 30, title: "Die Hard", genres: ["Action", "Thriller"], rating: 8.2, icon: "🔥", year: 1988 },
    { id: 31, title: "Mad Max: Fury Road", genres: ["Action", "Sci-Fi"], rating: 8.1, icon: "🏜️", year: 2015 },
    { id: 32, title: "Blade Runner", genres: ["Sci-Fi", "Thriller"], rating: 8.1, icon: "🌃", year: 1982 },
    { id: 33, title: "2001: A Space Odyssey", genres: ["Sci-Fi", "Mystery"], rating: 8.3, icon: "🌌", year: 1968 },
    { id: 34, title: "Eternal Sunshine", genres: ["Romance", "Drama"], rating: 8.3, icon: "💔", year: 2004 },
    { id: 35, title: "Her", genres: ["Romance", "Sci-Fi"], rating: 8.0, icon: "💕", year: 2013 },
    { id: 36, title: "La La Land", genres: ["Romance", "Musical"], rating: 8.0, icon: "🎵", year: 2016 },
    { id: 37, title: "Whiplash", genres: ["Drama", "Music"], rating: 8.5, icon: "🥁", year: 2014 },
    { id: 38, title: "The Social Network", genres: ["Drama", "Biography"], rating: 7.8, icon: "💻", year: 2010 },
    { id: 39, title: "Parasite", genres: ["Drama", "Thriller"], rating: 8.5, icon: "🏠", year: 2019 },
    { id: 40, title: "Get Out", genres: ["Horror", "Mystery"], rating: 7.7, icon: "😱", year: 2017 },
    { id: 41, title: "Joker", genres: ["Drama", "Thriller"], rating: 8.4, icon: "🃏", year: 2019 },
    { id: 42, title: "Django Unchained", genres: ["Western", "Drama"], rating: 8.5, icon: "🤠", year: 2012 },
    { id: 43, title: "Inglourious Basterds", genres: ["War", "Drama"], rating: 8.3, icon: "🎖️", year: 2009 },
    { id: 44, title: "The Grand Budapest Hotel", genres: ["Comedy", "Drama"], rating: 8.1, icon: "🏨", year: 2014 },
    { id: 45, title: "Moonrise Kingdom", genres: ["Comedy", "Drama"], rating: 7.8, icon: "🏕️", year: 2012 },
    { id: 46, title: "Amélie", genres: ["Romance", "Comedy"], rating: 8.3, icon: "☕", year: 2001 },
    { id: 47, title: "Life is Beautiful", genres: ["Drama", "Comedy"], rating: 8.6, icon: "❤️", year: 1997 },
    { id: 48, title: "Spirited Away", genres: ["Animation", "Fantasy"], rating: 8.6, icon: "🐉", year: 2001 },
    { id: 49, title: "Your Name", genres: ["Animation", "Romance"], rating: 8.4, icon: "☄️", year: 2016 }
];

// All possible genres
const allGenres = ["Sci-Fi", "Action", "Drama", "Crime", "Thriller", "Fantasy", "Adventure",
                   "Romance", "War", "History", "Mystery", "Animation", "Comedy", "Horror",
                   "Music", "Biography", "Western", "Musical"];

// Convert movie to feature vector
function movieToVector(movie) {
    const vector = [];

    // Genre features (18 dimensions)
    allGenres.forEach(genre => {
        vector.push(movie.genres.includes(genre) ? 1.0 : 0.0);
    });

    // Rating feature (1 dimension, normalized)
    vector.push(movie.rating / 10.0);

    // Year feature (1 dimension, normalized to 0-1)
    const yearNorm = (movie.year - 1960) / (2025 - 1960);
    vector.push(yearNorm);

    // Pad to reach 32 dimensions (for better distribution)
    while (vector.length < 32) {
        vector.push(0.0);
    }

    return vector;
}

// Initialize database
async function initDatabase() {
    try {
        await init();
        db = new VectorDB(32, Metric.Cosine, IndexType.HNSW);

        // Insert all movies
        const vectors = movies.map(movie => ({
            id: movie.id,
            vector: movieToVector(movie),
            metadata: {
                title: movie.title,
                genres: movie.genres.join(', '),
                rating: movie.rating,
                year: movie.year,
                icon: movie.icon
            }
        }));

        db.batch_insert(vectors);
        console.log('✓ Database initialized with', movies.length, 'movies');

        renderMovieGrid();
    } catch (error) {
        console.error('Failed to initialize:', error);
    }
}

// Render movie grid
function renderMovieGrid() {
    const grid = document.getElementById('movieGrid');
    grid.innerHTML = movies.map(movie => `
        <div class="movie-card" onclick="toggleMovie(${movie.id})">
            <div class="movie-icon">${movie.icon}</div>
            <div class="movie-title">${movie.title}</div>
            <div class="movie-genre">${movie.genres.join(', ')}</div>
            <div class="movie-rating">⭐ ${movie.rating}</div>
        </div>
    `).join('');
}

// Toggle movie selection
window.toggleMovie = function(movieId) {
    const card = event.currentTarget;

    if (selectedMovies.has(movieId)) {
        selectedMovies.delete(movieId);
        card.classList.remove('selected');
    } else {
        selectedMovies.add(movieId);
        card.classList.add('selected');
    }

    document.getElementById('selectedCount').textContent = selectedMovies.size;
    document.getElementById('recBtn').disabled = selectedMovies.size === 0;
};

// Get recommendations
window.getRecommendations = async function() {
    if (selectedMovies.size === 0) return;

    const recDiv = document.getElementById('recommendations');
    recDiv.innerHTML = '<div style="text-align: center; padding: 40px;"><div style="font-size: 2em;">⏳</div>Finding recommendations...</div>';

    try {
        // Calculate average vector of selected movies
        const selectedVectors = Array.from(selectedMovies).map(id => {
            const movie = movies.find(m => m.id === id);
            return movieToVector(movie);
        });

        const avgVector = selectedVectors[0].map((_, idx) => {
            const sum = selectedVectors.reduce((acc, vec) => acc + vec[idx], 0);
            return sum / selectedVectors.length;
        });

        // Search for similar movies
        const startTime = performance.now();
        const k = Math.min(10, movies.length - selectedMovies.size);
        const results = db.search(avgVector, k + selectedMovies.size, true);
        const searchTime = performance.now() - startTime;

        document.getElementById('searchTime').textContent = searchTime.toFixed(2) + 'ms';

        // Filter out already selected movies
        const recommendations = results
            .filter(r => !selectedMovies.has(r.id))
            .slice(0, 10);

        if (recommendations.length === 0) {
            recDiv.innerHTML = `
                <div class="empty-state">
                    <div class="empty-icon">😅</div>
                    <h3>You've seen all similar movies!</h3>
                    <p>Try selecting different movies</p>
                </div>
            `;
            return;
        }

        // Display recommendations
        recDiv.innerHTML = recommendations.map(rec => {
            const metadata = JSON.parse(rec.metadata);
            const scorePercent = (rec.score * 100).toFixed(1);

            // Generate reason
            const selectedGenres = new Set();
            selectedMovies.forEach(id => {
                const movie = movies.find(m => m.id === id);
                movie.genres.forEach(g => selectedGenres.add(g));
            });

            const matchingGenres = metadata.genres.split(', ')
                .filter(g => selectedGenres.has(g));

            const reason = matchingGenres.length > 0
                ? `Similar genres: ${matchingGenres.join(', ')}`
                : `High overall similarity (${scorePercent}%)`;

            return `
                <div class="recommendation-card">
                    <div class="rec-header">
                        <div>
                            <div style="font-size: 2em; margin-bottom: 5px;">${metadata.icon}</div>
                            <div class="rec-title">${metadata.title}</div>
                        </div>
                        <div class="rec-score">${scorePercent}%</div>
                    </div>
                    <div class="rec-details">
                        <span class="tag">${metadata.genres}</span>
                        <span class="tag">⭐ ${metadata.rating}</span>
                        <span class="tag">${metadata.year}</span>
                    </div>
                    <div class="rec-reason">
                        💡 ${reason}
                    </div>
                </div>
            `;
        }).join('');

    } catch (error) {
        console.error('Recommendation error:', error);
        recDiv.innerHTML = `
            <div class="empty-state">
                <div class="empty-icon">❌</div>
                <h3>Error</h3>
                <p>${error.message}</p>
            </div>
        `;
    }
};

// Initialize on load
initDatabase().then(() => {
    console.log('✓ Recommendation system ready');
});
