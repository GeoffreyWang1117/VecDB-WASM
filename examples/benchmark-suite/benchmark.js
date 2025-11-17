import init, { VectorDB, Metric, IndexType } from '../../pkg/vecdb_wasm.js';

let wasmReady = false;
let allResults = [];

// Initialize WASM
async function initWasm() {
    if (!wasmReady) {
        await init();
        wasmReady = true;
        console.log('✓ WASM initialized');
    }
}

// Generate random vector
function randomVector(dim) {
    return Array(dim).fill(0).map(() => Math.random());
}

// Generate dataset
function generateDataset(count, dim) {
    const dataset = [];
    for (let i = 0; i < count; i++) {
        dataset.push({
            id: i,
            vector: randomVector(dim),
            metadata: { index: i, type: 'test' }
        });
    }
    return dataset;
}

// Format time
function formatTime(ms) {
    if (ms < 1) return `${(ms * 1000).toFixed(2)}µs`;
    if (ms < 1000) return `${ms.toFixed(2)}ms`;
    return `${(ms / 1000).toFixed(2)}s`;
}

// Format throughput
function formatThroughput(ops, timeMs) {
    const opsPerSec = (ops / timeMs) * 1000;
    if (opsPerSec > 1000000) return `${(opsPerSec / 1000000).toFixed(2)}M ops/s`;
    if (opsPerSec > 1000) return `${(opsPerSec / 1000).toFixed(2)}K ops/s`;
    return `${opsPerSec.toFixed(2)} ops/s`;
}

// Update result box
function updateResult(id, html, className = '') {
    const box = document.getElementById(id);
    box.innerHTML = html;
    box.className = `result-box ${className}`;
}

// Add result to table
function addResult(name, config, timeMs, throughput, status = '✓') {
    allResults.push({ name, config, timeMs, throughput, status });
    updateResultsTable();
}

// Update results table
function updateResultsTable() {
    const tbody = document.getElementById('resultsBody');
    tbody.innerHTML = allResults.map(r => `
        <tr>
            <td><strong>${r.name}</strong></td>
            <td>${r.config}</td>
            <td>${formatTime(r.timeMs)}</td>
            <td>${r.throughput}</td>
            <td>${r.status}</td>
        </tr>
    `).join('');

    document.getElementById('summarySection').style.display = 'block';
    document.getElementById('exportBtn').disabled = false;
}

// Run with progress
async function runWithProgress(fn, resultId, btnId) {
    const btn = document.getElementById(btnId);
    btn.disabled = true;
    updateResult(resultId, '⏳ Running benchmark...', 'running');

    try {
        await fn();
        btn.disabled = false;
    } catch (error) {
        updateResult(resultId, `❌ Error: ${error.message}`, 'error');
        btn.disabled = false;
        throw error;
    }
}

// Benchmark: Distance Functions
window.runDistanceBench = async function() {
    await initWasm();
    await runWithProgress(async () => {
        const dim = parseInt(document.getElementById('dimension').value);
        const warmup = parseInt(document.getElementById('warmupRuns').value);
        const iterations = 1000;

        const v1 = randomVector(dim);
        const v2 = randomVector(dim);

        const metrics = [
            { name: 'Cosine', metric: Metric.Cosine },
            { name: 'Euclidean', metric: Metric.Euclidean },
            { name: 'DotProduct', metric: Metric.DotProduct },
            { name: 'Manhattan', metric: Metric.Manhattan },
            { name: 'Chebyshev', metric: Metric.Chebyshev },
            { name: 'Hamming', metric: Metric.Hamming },
            { name: 'Angular', metric: Metric.Angular },
        ];

        let html = `<div class="metric-row"><div class="metric-label">Dimension</div><div class="metric-value">${dim}D</div></div>`;
        html += `<div class="metric-row"><div class="metric-label">Iterations</div><div class="metric-value">${iterations}</div></div><hr style="margin: 10px 0">`;

        for (const { name, metric } of metrics) {
            const db = new VectorDB(dim, metric, IndexType.Flat);
            db.insert(0, v1);
            db.insert(1, v2);

            // Warmup
            for (let i = 0; i < warmup; i++) {
                db.search(v1, 1);
            }

            // Measure
            const start = performance.now();
            for (let i = 0; i < iterations; i++) {
                db.search(v1, 1);
            }
            const elapsed = performance.now() - start;
            const avgTime = elapsed / iterations;

            html += `<div class="metric-row">
                <div class="metric-label">${name}</div>
                <div class="metric-value">${formatTime(avgTime)} per op</div>
            </div>`;

            addResult('Distance', `${name} ${dim}D`, avgTime, formatThroughput(1, avgTime));
        }

        updateResult('distanceResult', html, 'success');
    }, 'distanceResult', 'distanceBtn');
};

// Benchmark: Scaling Test
window.runScalingBench = async function() {
    await initWasm();
    await runWithProgress(async () => {
        const dim = parseInt(document.getElementById('dimension').value);
        const k = parseInt(document.getElementById('searchK').value);
        const sizes = [1000, 5000, 10000, 20000, 50000];

        let html = `<div class="metric-row"><div class="metric-label">Dimension</div><div class="metric-value">${dim}D</div></div>`;
        html += `<div class="metric-row"><div class="metric-label">K</div><div class="metric-value">${k}</div></div><hr style="margin: 10px 0">`;

        for (const size of sizes) {
            const dataset = generateDataset(size, dim);
            const db = new VectorDB(dim, Metric.Cosine, IndexType.HNSW);

            // Measure insert
            const insertStart = performance.now();
            db.batch_insert(dataset);
            const insertTime = performance.now() - insertStart;

            // Measure search
            const query = randomVector(dim);
            const searchStart = performance.now();
            db.search(query, k);
            const searchTime = performance.now() - searchStart;

            html += `<div class="metric-row">
                <div class="metric-label">${(size/1000).toFixed(0)}K vectors</div>
                <div class="metric-value">Insert: ${formatTime(insertTime)} | Search: ${formatTime(searchTime)}</div>
            </div>`;

            addResult('Scaling', `${size} vectors ${dim}D`, searchTime, formatThroughput(size, insertTime));
        }

        updateResult('scalingResult', html, 'success');
    }, 'scalingResult', 'scalingBtn');
};

// Benchmark: Dimension Test
window.runDimensionBench = async function() {
    await initWasm();
    await runWithProgress(async () => {
        const size = parseInt(document.getElementById('datasetSize').value);
        const k = parseInt(document.getElementById('searchK').value);
        const dimensions = [64, 128, 256, 512, 1024];

        let html = `<div class="metric-row"><div class="metric-label">Dataset Size</div><div class="metric-value">${size}</div></div>`;
        html += `<div class="metric-row"><div class="metric-label">K</div><div class="metric-value">${k}</div></div><hr style="margin: 10px 0">`;

        for (const dim of dimensions) {
            const dataset = generateDataset(Math.min(size, 10000), dim); // Cap for higher dims
            const db = new VectorDB(dim, Metric.Cosine, IndexType.HNSW);

            const insertStart = performance.now();
            db.batch_insert(dataset);
            const insertTime = performance.now() - insertStart;

            const query = randomVector(dim);
            const searchStart = performance.now();
            db.search(query, k);
            const searchTime = performance.now() - searchStart;

            html += `<div class="metric-row">
                <div class="metric-label">${dim}D</div>
                <div class="metric-value">Search: ${formatTime(searchTime)} | Insert: ${formatTime(insertTime)}</div>
            </div>`;

            addResult('Dimension', `${dim}D ${dataset.length} vectors`, searchTime, formatThroughput(dataset.length, insertTime));
        }

        updateResult('dimensionResult', html, 'success');
    }, 'dimensionResult', 'dimensionBtn');
};

// Benchmark: Index Comparison
window.runIndexBench = async function() {
    await initWasm();
    await runWithProgress(async () => {
        const size = parseInt(document.getElementById('datasetSize').value);
        const dim = parseInt(document.getElementById('dimension').value);
        const k = parseInt(document.getElementById('searchK').value);

        const dataset = generateDataset(size, dim);
        const query = randomVector(dim);

        let html = `<div class="metric-row"><div class="metric-label">Configuration</div><div class="metric-value">${size} vectors, ${dim}D, k=${k}</div></div><hr style="margin: 10px 0">`;

        // Flat Index
        const flatDb = new VectorDB(dim, Metric.Cosine, IndexType.Flat);
        const flatInsertStart = performance.now();
        flatDb.batch_insert(dataset);
        const flatInsertTime = performance.now() - flatInsertStart;

        const flatSearchStart = performance.now();
        const flatResults = flatDb.search(query, k);
        const flatSearchTime = performance.now() - flatSearchStart;

        html += `<div class="metric-row">
            <div class="metric-label">Flat Index</div>
            <div class="metric-value">Insert: ${formatTime(flatInsertTime)} | Search: ${formatTime(flatSearchTime)}</div>
        </div>`;

        // HNSW Index
        const hnswDb = new VectorDB(dim, Metric.Cosine, IndexType.HNSW);
        const hnswInsertStart = performance.now();
        hnswDb.batch_insert(dataset);
        const hnswInsertTime = performance.now() - hnswInsertStart;

        const hnswSearchStart = performance.now();
        const hnswResults = hnswDb.search(query, k);
        const hnswSearchTime = performance.now() - hnswSearchStart;

        html += `<div class="metric-row">
            <div class="metric-label">HNSW Index</div>
            <div class="metric-value">Insert: ${formatTime(hnswInsertTime)} | Search: ${formatTime(hnswSearchTime)}</div>
        </div>`;

        const speedup = (flatSearchTime / hnswSearchTime).toFixed(2);
        html += `<hr style="margin: 10px 0"><div class="metric-row">
            <div class="metric-label">HNSW Speedup</div>
            <div class="metric-value">${speedup}x faster</div>
        </div>`;

        addResult('Index', `Flat ${size} vectors`, flatSearchTime, formatThroughput(size, flatInsertTime));
        addResult('Index', `HNSW ${size} vectors`, hnswSearchTime, formatThroughput(size, hnswInsertTime));

        updateResult('indexResult', html, 'success');
    }, 'indexResult', 'indexBtn');
};

// Benchmark: Batch Operations
window.runBatchBench = async function() {
    await initWasm();
    await runWithProgress(async () => {
        const dim = parseInt(document.getElementById('dimension').value);
        const k = parseInt(document.getElementById('searchK').value);
        const batchSizes = [100, 500, 1000, 5000, 10000];

        let html = `<div class="metric-row"><div class="metric-label">Dimension</div><div class="metric-value">${dim}D</div></div><hr style="margin: 10px 0">`;

        for (const batchSize of batchSizes) {
            const dataset = generateDataset(batchSize, dim);
            const db = new VectorDB(dim, Metric.Cosine, IndexType.HNSW);

            const start = performance.now();
            db.batch_insert(dataset);
            const elapsed = performance.now() - start;

            const perVector = elapsed / batchSize;

            html += `<div class="metric-row">
                <div class="metric-label">${batchSize} vectors</div>
                <div class="metric-value">${formatTime(elapsed)} (${formatTime(perVector)}/vec)</div>
            </div>`;

            addResult('Batch', `${batchSize} vectors ${dim}D`, elapsed, formatThroughput(batchSize, elapsed));
        }

        updateResult('batchResult', html, 'success');
    }, 'batchResult', 'batchBtn');
};

// Benchmark: Metric Comparison
window.runMetricComparison = async function() {
    await initWasm();
    await runWithProgress(async () => {
        const size = parseInt(document.getElementById('datasetSize').value);
        const dim = parseInt(document.getElementById('dimension').value);
        const k = parseInt(document.getElementById('searchK').value);

        const dataset = generateDataset(size, dim);
        const query = randomVector(dim);

        const metrics = [
            { name: 'Cosine', metric: Metric.Cosine },
            { name: 'Euclidean', metric: Metric.Euclidean },
            { name: 'Manhattan', metric: Metric.Manhattan },
            { name: 'Chebyshev', metric: Metric.Chebyshev },
            { name: 'Angular', metric: Metric.Angular },
        ];

        let html = `<div class="metric-row"><div class="metric-label">Configuration</div><div class="metric-value">${size} vectors, ${dim}D, k=${k}</div></div><hr style="margin: 10px 0">`;

        for (const { name, metric } of metrics) {
            const db = new VectorDB(dim, metric, IndexType.HNSW);

            const insertStart = performance.now();
            db.batch_insert(dataset);
            const insertTime = performance.now() - insertStart;

            const searchStart = performance.now();
            db.search(query, k);
            const searchTime = performance.now() - searchStart;

            html += `<div class="metric-row">
                <div class="metric-label">${name}</div>
                <div class="metric-value">Search: ${formatTime(searchTime)} | Insert: ${formatTime(insertTime)}</div>
            </div>`;

            addResult('Metric', `${name} ${size} vectors`, searchTime, formatThroughput(size, insertTime));
        }

        updateResult('metricResult', html, 'success');
    }, 'metricResult', 'metricBtn');
};

// Run all benchmarks
window.runAllBenchmarks = async function() {
    allResults = [];
    document.getElementById('resultsBody').innerHTML = '';
    document.getElementById('summarySection').style.display = 'none';

    await runDistanceBench();
    await runScalingBench();
    await runDimensionBench();
    await runIndexBench();
    await runBatchBench();
    await runMetricComparison();

    updateSummary();
};

// Update summary
function updateSummary() {
    const totalBenchmarks = allResults.length;
    const avgTime = allResults.reduce((sum, r) => sum + r.timeMs, 0) / totalBenchmarks;
    const fastest = Math.min(...allResults.map(r => r.timeMs));
    const slowest = Math.max(...allResults.map(r => r.timeMs));

    document.getElementById('summaryGrid').innerHTML = `
        <div class="summary-card">
            <div class="label">Total Benchmarks</div>
            <div class="value">${totalBenchmarks}</div>
        </div>
        <div class="summary-card">
            <div class="label">Average Time</div>
            <div class="value">${formatTime(avgTime)}</div>
        </div>
        <div class="summary-card">
            <div class="label">Fastest</div>
            <div class="value">${formatTime(fastest)}</div>
        </div>
        <div class="summary-card">
            <div class="label">Slowest</div>
            <div class="value">${formatTime(slowest)}</div>
        </div>
    `;
}

// Export results
window.exportResults = function() {
    const data = {
        timestamp: new Date().toISOString(),
        browser: navigator.userAgent,
        results: allResults
    };

    const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `vecdb-benchmark-${Date.now()}.json`;
    a.click();
    URL.revokeObjectURL(url);
};

// Clear results
window.clearResults = function() {
    allResults = [];
    document.getElementById('resultsBody').innerHTML = '';
    document.getElementById('summarySection').style.display = 'none';
    document.querySelectorAll('.result-box').forEach(box => box.innerHTML = '');
};

// Initialize on load
initWasm().then(() => {
    console.log('✓ Benchmark suite ready');
});
