import init, { VectorDB, Metric, IndexType } from '../../pkg/vecdb_wasm.js';

let db = null;
let vocabulary = new Map();
let documentCount = 0;
const VECTOR_DIM = 128;

// Sample documents
const sampleDocuments = [
    {
        title: "Introduction to Machine Learning",
        category: "AI/ML",
        content: "Machine learning is a subset of artificial intelligence that enables systems to learn and improve from experience without being explicitly programmed. It focuses on developing computer programs that can access data and use it to learn for themselves.",
        tags: ["machine learning", "AI", "data science", "algorithms"]
    },
    {
        title: "Web Development Best Practices",
        category: "Programming",
        content: "Modern web development involves responsive design, performance optimization, security considerations, and accessibility. Developers should follow established patterns, use version control, and implement continuous integration and deployment pipelines.",
        tags: ["web dev", "frontend", "backend", "best practices"]
    },
    {
        title: "Database Design Principles",
        category: "Databases",
        content: "Effective database design involves normalization, indexing strategies, query optimization, and proper schema design. Understanding relationships between entities and choosing the right database type for your use case is crucial.",
        tags: ["databases", "SQL", "NoSQL", "design"]
    },
    {
        title: "Cloud Computing Architecture",
        category: "Infrastructure",
        content: "Cloud computing provides on-demand access to computing resources including servers, storage, databases, networking, software, and analytics. It offers scalability, reliability, and cost-effectiveness for modern applications.",
        tags: ["cloud", "AWS", "Azure", "infrastructure"]
    },
    {
        title: "Cybersecurity Fundamentals",
        category: "Security",
        content: "Cybersecurity involves protecting computer systems, networks, and data from digital attacks, unauthorized access, and damage. Key concepts include encryption, authentication, firewalls, and security best practices.",
        tags: ["security", "encryption", "authentication", "protection"]
    },
    {
        title: "Data Science and Analytics",
        category: "Data",
        content: "Data science combines statistics, mathematics, and programming to extract insights from data. It involves data collection, cleaning, analysis, visualization, and machine learning to solve complex problems and make data-driven decisions.",
        tags: ["data science", "analytics", "statistics", "visualization"]
    },
    {
        title: "Mobile App Development",
        category: "Programming",
        content: "Mobile development encompasses creating applications for iOS and Android platforms. Modern approaches include native development, cross-platform frameworks like React Native and Flutter, and progressive web apps.",
        tags: ["mobile", "iOS", "Android", "apps"]
    },
    {
        title: "DevOps and CI/CD",
        category: "Infrastructure",
        content: "DevOps practices combine software development and IT operations to shorten development cycles and provide continuous delivery. CI/CD pipelines automate testing, building, and deployment processes.",
        tags: ["DevOps", "CI/CD", "automation", "deployment"]
    },
    {
        title: "Blockchain Technology",
        category: "Technology",
        content: "Blockchain is a distributed ledger technology that maintains a secure and decentralized record of transactions. It's the foundation of cryptocurrencies and has applications in supply chain, healthcare, and finance.",
        tags: ["blockchain", "cryptocurrency", "distributed systems", "security"]
    },
    {
        title: "Natural Language Processing",
        category: "AI/ML",
        content: "NLP enables computers to understand, interpret, and generate human language. Applications include chatbots, sentiment analysis, machine translation, and text summarization using deep learning models.",
        tags: ["NLP", "AI", "text processing", "language models"]
    },
    {
        title: "Computer Vision Applications",
        category: "AI/ML",
        content: "Computer vision enables machines to interpret and understand visual information from images and videos. Applications include facial recognition, object detection, autonomous vehicles, and medical image analysis.",
        tags: ["computer vision", "image processing", "deep learning", "AI"]
    },
    {
        title: "Microservices Architecture",
        category: "Architecture",
        content: "Microservices architecture structures an application as a collection of loosely coupled services. Each service is independently deployable, scalable, and maintainable, improving system resilience and development velocity.",
        tags: ["microservices", "architecture", "scalability", "distributed systems"]
    },
    {
        title: "Agile Software Development",
        category: "Methodology",
        content: "Agile methodology emphasizes iterative development, collaboration, and flexibility. Teams work in sprints, deliver incremental value, and adapt to changing requirements through continuous feedback and improvement.",
        tags: ["agile", "scrum", "project management", "software development"]
    },
    {
        title: "API Design and RESTful Services",
        category: "Programming",
        content: "RESTful APIs follow principles of representational state transfer, using HTTP methods for CRUD operations. Good API design includes versioning, authentication, rate limiting, and comprehensive documentation.",
        tags: ["API", "REST", "HTTP", "web services"]
    },
    {
        title: "Quantum Computing Basics",
        category: "Technology",
        content: "Quantum computing leverages quantum mechanics principles like superposition and entanglement to process information. It promises exponential speedup for certain computational problems compared to classical computers.",
        tags: ["quantum computing", "physics", "algorithms", "future tech"]
    },
    {
        title: "Containerization with Docker",
        category: "Infrastructure",
        content: "Docker containers package applications with their dependencies, ensuring consistency across environments. Container orchestration platforms like Kubernetes manage deployment, scaling, and operations of containerized applications.",
        tags: ["Docker", "containers", "Kubernetes", "deployment"]
    },
    {
        title: "Git Version Control",
        category: "Tools",
        content: "Git is a distributed version control system that tracks changes in source code. It enables collaboration through branching, merging, and pull requests, and is essential for modern software development workflows.",
        tags: ["Git", "version control", "collaboration", "development"]
    },
    {
        title: "UI/UX Design Principles",
        category: "Design",
        content: "User interface and experience design focuses on creating intuitive, accessible, and aesthetically pleasing digital products. Key principles include consistency, feedback, simplicity, and user-centered design.",
        tags: ["UI", "UX", "design", "usability"]
    },
    {
        title: "Software Testing Strategies",
        category: "Quality Assurance",
        content: "Comprehensive testing includes unit tests, integration tests, end-to-end tests, and performance testing. Test-driven development and automated testing improve code quality and reduce bugs.",
        tags: ["testing", "QA", "automation", "quality"]
    },
    {
        title: "Big Data Processing",
        category: "Data",
        content: "Big data technologies like Hadoop, Spark, and distributed databases handle massive datasets that traditional systems can't process. They enable parallel processing, real-time analytics, and scalable storage.",
        tags: ["big data", "Hadoop", "Spark", "distributed computing"]
    }
];

// Simple text vectorization using TF-IDF-like approach
function textToVector(text, dim = VECTOR_DIM) {
    const words = text.toLowerCase()
        .replace(/[^\w\s]/g, '')
        .split(/\s+/)
        .filter(w => w.length > 3);

    // Create a simple hash-based vector
    const vector = new Array(dim).fill(0);

    words.forEach(word => {
        // Simple hash function
        let hash = 0;
        for (let i = 0; i < word.length; i++) {
            hash = ((hash << 5) - hash) + word.charCodeAt(i);
            hash = hash & hash;
        }

        const index = Math.abs(hash) % dim;
        vector[index] += 1.0;

        // Also add to neighboring indices for better distribution
        vector[(index + 1) % dim] += 0.5;
        vector[(index - 1 + dim) % dim] += 0.5;
    });

    // Normalize vector
    const magnitude = Math.sqrt(vector.reduce((sum, val) => sum + val * val, 0));
    if (magnitude > 0) {
        for (let i = 0; i < vector.length; i++) {
            vector[i] /= magnitude;
        }
    }

    return vector;
}

// Initialize WASM and database
async function initDatabase() {
    try {
        await init();
        db = new VectorDB(VECTOR_DIM, Metric.Cosine, IndexType.HNSW);
        console.log('✓ Database initialized');
        updateStats();
    } catch (error) {
        console.error('Failed to initialize:', error);
    }
}

// Load sample documents
window.loadSampleData = async function() {
    if (!db) await initDatabase();

    const loadBtn = document.getElementById('loadBtn');
    loadBtn.disabled = true;
    loadBtn.textContent = '⏳ Loading...';

    try {
        const vectors = sampleDocuments.map((doc, idx) => {
            const fullText = `${doc.title} ${doc.content} ${doc.tags.join(' ')}`;
            const vector = textToVector(fullText);

            return {
                id: idx,
                vector: vector,
                metadata: {
                    title: doc.title,
                    category: doc.category,
                    content: doc.content,
                    tags: doc.tags.join(', ')
                }
            };
        });

        const count = db.batch_insert(vectors);
        documentCount = count;

        console.log(`✓ Loaded ${count} documents`);
        updateStats();

        loadBtn.textContent = '✓ Loaded!';
        setTimeout(() => {
            loadBtn.textContent = '📚 Reload Sample Documents';
            loadBtn.disabled = false;
        }, 2000);

        // Show example queries
        const exampleDiv = document.getElementById('exampleQueries');
        exampleDiv.style.display = 'block';

        const exampleList = document.getElementById('exampleList');
        const examples = [
            "artificial intelligence and machine learning",
            "web development and programming",
            "cloud infrastructure and deployment",
            "data analysis and visualization",
            "security and encryption",
            "mobile app development"
        ];

        exampleList.innerHTML = examples.map(q =>
            `<span class="example-query" onclick="document.getElementById('searchInput').value='${q}'; performSearch();">${q}</span>`
        ).join('');

    } catch (error) {
        console.error('Error loading data:', error);
        loadBtn.disabled = false;
        loadBtn.textContent = '📚 Load Sample Documents';
    }
};

// Perform search
window.performSearch = async function() {
    const query = document.getElementById('searchInput').value.trim();
    if (!query) return;
    if (!db || documentCount === 0) {
        alert('Please load sample documents first!');
        return;
    }

    const resultsContainer = document.getElementById('resultsContainer');
    resultsContainer.innerHTML = '<div class="loading"><div class="spinner"></div>Searching...</div>';

    try {
        const queryVector = textToVector(query);
        const k = Math.min(10, documentCount);

        const startTime = performance.now();
        const results = db.search(queryVector, k, true);
        const searchTime = performance.now() - startTime;

        document.getElementById('searchTime').textContent = `${searchTime.toFixed(2)}ms`;

        if (results.length === 0) {
            resultsContainer.innerHTML = `
                <div class="no-results">
                    <div class="no-results-icon">😕</div>
                    <h3>No results found</h3>
                    <p>Try a different search query</p>
                </div>
            `;
            return;
        }

        resultsContainer.innerHTML = results.map(result => {
            const metadata = JSON.parse(result.metadata);
            const scorePercent = (result.score * 100).toFixed(1);

            return `
                <div class="result-card">
                    <div class="result-header">
                        <div>
                            <div class="result-title">${metadata.title}</div>
                            <span class="result-category">${metadata.category}</span>
                        </div>
                        <div class="result-score">${scorePercent}%</div>
                    </div>
                    <div class="result-content">${metadata.content}</div>
                    <div class="result-meta">
                        <div>🏷️ ${metadata.tags}</div>
                        <div>📄 ID: ${result.id}</div>
                    </div>
                </div>
            `;
        }).join('');

    } catch (error) {
        console.error('Search error:', error);
        resultsContainer.innerHTML = `
            <div class="no-results">
                <div class="no-results-icon">❌</div>
                <h3>Search error</h3>
                <p>${error.message}</p>
            </div>
        `;
    }
};

// Clear database
window.clearDatabase = function() {
    if (!db) return;

    db.clear();
    documentCount = 0;
    updateStats();

    document.getElementById('resultsContainer').innerHTML = `
        <div class="no-results">
            <div class="no-results-icon">📚</div>
            <h3>Database cleared</h3>
            <p>Load sample documents to start searching!</p>
        </div>
    `;

    document.getElementById('exampleQueries').style.display = 'none';
    document.getElementById('searchInput').value = '';
};

// Update statistics
function updateStats() {
    if (!db) {
        document.getElementById('docCount').textContent = '0';
        document.getElementById('vectorDim').textContent = '-';
        return;
    }

    document.getElementById('docCount').textContent = db.len();
    document.getElementById('vectorDim').textContent = `${VECTOR_DIM}D`;
}

// Initialize on load
initDatabase().then(() => {
    console.log('✓ Semantic search ready');
});
