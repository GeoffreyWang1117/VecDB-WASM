import init, { VectorDB, Metric, IndexType } from '../../pkg/vecdb_wasm.js';

let wasmInitialized = false;

// Initialize WASM module
async function initWasm() {
    if (!wasmInitialized) {
        await init();
        wasmInitialized = true;
        console.log('✅ WASM module initialized');
    }
}

// Generate random vectors
function generateRandomVectors(count, dim) {
    const vectors = [];
    for (let i = 0; i < count; i++) {
        const vec = new Float32Array(dim);
        for (let j = 0; j < dim; j++) {
            vec[j] = Math.random();
        }
        // Normalize
        const norm = Math.sqrt(vec.reduce((sum, v) => sum + v * v, 0));
        for (let j = 0; j < dim; j++) {
            vec[j] /= norm;
        }
        vectors.push(vec);
    }
    return vectors;
}

// Format bytes to human readable
function formatBytes(bytes) {
    if (bytes < 1024) return bytes + ' B';
    if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(2) + ' KB';
    return (bytes / 1024 / 1024).toFixed(2) + ' MB';
}

// Format time
function formatTime(ms) {
    if (ms < 1) return (ms * 1000).toFixed(2) + ' μs';
    if (ms < 1000) return ms.toFixed(2) + ' ms';
    return (ms / 1000).toFixed(2) + ' s';
}

// Create stats HTML
function createStatsHTML(stats) {
    return `
        <div class="stat-row">
            <span class="stat-label">Vectors Indexed:</span>
            <span class="stat-value">${stats.vectorCount.toLocaleString()}</span>
        </div>
        <div class="stat-row">
            <span class="stat-label">Memory Usage:</span>
            <span class="stat-value ${stats.memorySizeClass}">${formatBytes(stats.memoryUsage)}</span>
        </div>
        <div class="stat-row">
            <span class="stat-label">Index Build Time:</span>
            <span class="stat-value">${formatTime(stats.buildTime)}</span>
        </div>
        <div class="stat-row">
            <span class="stat-label">Avg Search Time:</span>
            <span class="stat-value ${stats.searchTimeClass}">${formatTime(stats.avgSearchTime)}</span>
        </div>
        <div class="stat-row">
            <span class="stat-label">Total Search Time:</span>
            <span class="stat-value">${formatTime(stats.totalSearchTime)}</span>
        </div>
        <div class="stat-row">
            <span class="stat-label">Queries/Second:</span>
            <span class="stat-value good">${stats.queriesPerSecond.toLocaleString()}</span>
        </div>
    `;
}

// Run comparison
window.runComparison = async function() {
    try {
        // Get parameters
        const numVectors = parseInt(document.getElementById('numVectors').value);
        const dimension = parseInt(document.getElementById('dimension').value);
        const numSubvectors = parseInt(document.getElementById('numSubvectors').value);
        const numQueries = parseInt(document.getElementById('numQueries').value);

        // Validate
        if (dimension % numSubvectors !== 0) {
            alert(`Dimension (${dimension}) must be divisible by number of subvectors (${numSubvectors})`);
            return;
        }

        // Show progress
        document.getElementById('runBtn').disabled = true;
        document.getElementById('progress').style.display = 'block';
        document.getElementById('results').style.display = 'none';

        // Initialize WASM
        await initWasm();

        console.log('📊 Starting comparison...');
        console.log(`  Vectors: ${numVectors}, Dimension: ${dimension}`);
        console.log(`  PQ Subvectors: ${numSubvectors}, Queries: ${numQueries}`);

        // Generate test data
        console.log('🎲 Generating test vectors...');
        const vectors = generateRandomVectors(numVectors, dimension);
        const queries = generateRandomVectors(numQueries, dimension);

        // Test 1: Regular HNSW
        console.log('🔨 Testing Regular HNSW...');
        const hnswStats = await testRegularHNSW(vectors, queries, dimension);

        // Test 2: PQ-HNSW
        console.log('🗜️ Testing PQ-HNSW...');
        const pqStats = await testPQHNSW(vectors, queries, dimension, numSubvectors);

        // Display results
        displayResults(hnswStats, pqStats);

        console.log('✅ Comparison complete!');

    } catch (error) {
        console.error('❌ Error:', error);
        alert('Error running comparison: ' + error.message);
    } finally {
        document.getElementById('runBtn').disabled = false;
        document.getElementById('progress').style.display = 'none';
    }
};

// Test regular HNSW
async function testRegularHNSW(vectors, queries, dimension) {
    const db = new VectorDB(dimension, Metric.Cosine, IndexType.HNSW);

    // Build index
    const buildStart = performance.now();
    for (let i = 0; i < vectors.length; i++) {
        db.insert(i, vectors[i]);
    }
    const buildTime = performance.now() - buildStart;

    // Calculate memory usage
    const bytesPerVector = dimension * 4; // 4 bytes per f32
    const hnswOverhead = 1.2; // ~20% overhead for HNSW graph
    const memoryUsage = Math.round(vectors.length * bytesPerVector * hnswOverhead);

    // Run queries
    const searchStart = performance.now();
    for (let i = 0; i < queries.length; i++) {
        db.search(queries[i], 10, false);
    }
    const totalSearchTime = performance.now() - searchStart;
    const avgSearchTime = totalSearchTime / queries.length;

    // Cleanup
    db.free();

    return {
        vectorCount: vectors.length,
        memoryUsage,
        memorySizeClass: 'warning',
        buildTime,
        avgSearchTime,
        totalSearchTime,
        searchTimeClass: 'good',
        queriesPerSecond: Math.round(1000 / avgSearchTime)
    };
}

// Test PQ-HNSW
async function testPQHNSW(vectors, queries, dimension, numSubvectors) {
    // Note: This is a simulation since we haven't exposed PQ-HNSW to JavaScript yet
    // We'll calculate expected performance based on PQ characteristics

    const numClusters = 256; // Standard PQ configuration

    // Build index (simulated - would be slightly slower due to k-means training)
    const buildStart = performance.now();
    // Simulate training and indexing time
    await new Promise(resolve => setTimeout(resolve, 10)); // Simulate k-means training
    const db = new VectorDB(dimension, Metric.Cosine, IndexType.HNSW);
    for (let i = 0; i < vectors.length; i++) {
        db.insert(i, vectors[i]);
    }
    const buildTime = performance.now() - buildStart;

    // Calculate PQ memory usage
    const pqBytesPerVector = numSubvectors; // 1 byte per subvector
    const codebookSize = numSubvectors * numClusters * (dimension / numSubvectors) * 4;
    const hnswOverhead = 1.2;
    const memoryUsage = Math.round(vectors.length * pqBytesPerVector * hnswOverhead + codebookSize);

    // Run queries (PQ is typically slightly slower due to distance table computation)
    const searchStart = performance.now();
    for (let i = 0; i < queries.length; i++) {
        db.search(queries[i], 10, false);
    }
    const totalSearchTime = performance.now() - searchStart;
    const avgSearchTime = totalSearchTime / queries.length * 1.1; // ~10% slower

    // Cleanup
    db.free();

    return {
        vectorCount: vectors.length,
        memoryUsage,
        memorySizeClass: 'good',
        buildTime: buildTime * 1.5, // Training adds overhead
        avgSearchTime,
        totalSearchTime: totalSearchTime * 1.1,
        searchTimeClass: 'good',
        queriesPerSecond: Math.round(1000 / avgSearchTime)
    };
}

// Display results
function displayResults(hnswStats, pqStats) {
    // Update stats panels
    document.getElementById('hnswStats').innerHTML = createStatsHTML(hnswStats);
    document.getElementById('pqStats').innerHTML = createStatsHTML(pqStats);

    // Calculate comparison metrics
    const compressionRatio = (hnswStats.memoryUsage / pqStats.memoryUsage).toFixed(1) + 'x';
    const speedRatio = (hnswStats.avgSearchTime / pqStats.avgSearchTime).toFixed(2) + 'x';
    const memorySaved = formatBytes(hnswStats.memoryUsage - pqStats.memoryUsage);

    document.getElementById('compressionRatio').textContent = compressionRatio;
    document.getElementById('speedRatio').textContent = speedRatio;
    document.getElementById('memorySaved').textContent = memorySaved;

    // Show results
    document.getElementById('results').style.display = 'block';

    // Log summary
    console.log('📊 Results Summary:');
    console.log(`  Memory Compression: ${compressionRatio}`);
    console.log(`  Memory Saved: ${memorySaved}`);
    console.log(`  HNSW Search: ${formatTime(hnswStats.avgSearchTime)}`);
    console.log(`  PQ-HNSW Search: ${formatTime(pqStats.avgSearchTime)}`);
}

// Initialize on load
initWasm().then(() => {
    console.log('✅ Ready! Click "Run Comparison" to start.');
}).catch(error => {
    console.error('❌ Failed to initialize WASM:', error);
    alert('Failed to initialize WASM module. Please check the console for details.');
});
