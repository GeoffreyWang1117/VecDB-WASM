/**
 * VecDB-WASM Worker Client
 *
 * Promise-based wrapper for VecDB Web Worker
 * Provides the same API as VectorDB but runs operations in a background thread
 */

export class VectorDBWorker {
    constructor(workerPath = './vecdb.worker.js') {
        this.worker = new Worker(workerPath);
        this.nextId = 0;
        this.pending = new Map();

        // Handle messages from worker
        this.worker.onmessage = (event) => {
            const { id, success, result, error } = event.data;
            const { resolve, reject } = this.pending.get(id);

            if (success) {
                resolve(result);
            } else {
                reject(new Error(error));
            }

            this.pending.delete(id);
        };

        // Handle worker errors
        this.worker.onerror = (error) => {
            console.error('Worker error:', error);
            // Reject all pending promises
            for (const { reject } of this.pending.values()) {
                reject(error);
            }
            this.pending.clear();
        };
    }

    /**
     * Send a message to the worker and wait for response
     */
    async call(method, params = {}) {
        return new Promise((resolve, reject) => {
            const id = this.nextId++;
            this.pending.set(id, { resolve, reject });

            this.worker.postMessage({
                id,
                method,
                params
            });
        });
    }

    /**
     * Create a new vector database
     */
    async create(dimension, metric, indexType) {
        return this.call('create', { dimension, metric, indexType });
    }

    /**
     * Create a new vector database with custom HNSW parameters
     */
    async createWithHNSWParams(dimension, metric, m, ef_construction) {
        return this.call('create_with_params', {
            dimension,
            metric,
            m,
            ef_construction
        });
    }

    /**
     * Insert a vector
     */
    async insert(id, vector, metadata) {
        return this.call('insert', { id, vector, metadata });
    }

    /**
     * Batch insert multiple vectors
     */
    async batchInsert(vectors) {
        const result = await this.call('batch_insert', { vectors });
        return result.count;
    }

    /**
     * Search for k nearest neighbors
     */
    async search(query, k, includeMetadata = false) {
        return this.call('search', { query, k, include_metadata: includeMetadata });
    }

    /**
     * Search with metadata filter
     */
    async searchWithFilter(query, k, filter, includeMetadata = false) {
        return this.call('search_with_filter', {
            query,
            k,
            filter,
            include_metadata: includeMetadata
        });
    }

    /**
     * Batch search multiple queries
     */
    async batchSearch(queries, k, includeMetadata = false) {
        return this.call('batch_search', {
            queries,
            k,
            include_metadata: includeMetadata
        });
    }

    /**
     * Search within a radius
     */
    async searchRadius(query, radius, maxResults, includeMetadata = false) {
        return this.call('search_radius', {
            query,
            radius,
            max_results: maxResults,
            include_metadata: includeMetadata
        });
    }

    /**
     * Remove a vector by ID
     */
    async remove(id) {
        const result = await this.call('remove', { id });
        return result.removed;
    }

    /**
     * Clear all vectors
     */
    async clear() {
        return this.call('clear');
    }

    /**
     * Get a vector by ID
     */
    async getVector(id) {
        return this.call('get_vector', { id });
    }

    /**
     * Get number of vectors
     */
    async len() {
        const result = await this.call('len');
        return result.length;
    }

    /**
     * Check if database is empty
     */
    async isEmpty() {
        const result = await this.call('is_empty');
        return result.empty;
    }

    /**
     * Get vector dimension
     */
    async dimension() {
        const result = await this.call('dimension');
        return result.dimension;
    }

    /**
     * Get database statistics
     */
    async getStats() {
        return this.call('get_stats');
    }

    /**
     * Get performance metrics
     */
    async getPerformanceMetrics() {
        return this.call('get_performance_metrics');
    }

    /**
     * Reset performance metrics
     */
    async resetPerformanceMetrics() {
        return this.call('reset_performance_metrics');
    }

    /**
     * Export database to binary format
     */
    async exportSnapshot() {
        return this.call('export_snapshot');
    }

    /**
     * Export database to JSON format
     */
    async exportSnapshotJson() {
        return this.call('export_snapshot_json');
    }

    /**
     * Import database from binary format
     */
    async importSnapshot(data) {
        return this.call('import_snapshot', { data });
    }

    /**
     * Import database from JSON format
     */
    async importSnapshotJson(json) {
        return this.call('import_snapshot_json', { json });
    }

    /**
     * Save to IndexedDB
     */
    async saveToIndexedDB(dbName) {
        return this.call('save_to_indexeddb', { db_name: dbName });
    }

    /**
     * Load from IndexedDB
     */
    async loadFromIndexedDB(dbName) {
        return this.call('load_from_indexeddb', { db_name: dbName });
    }

    /**
     * Delete from IndexedDB
     */
    async deleteFromIndexedDB(dbName) {
        return this.call('delete_from_indexeddb', { db_name: dbName });
    }

    /**
     * List saved databases
     */
    async listSavedDatabases() {
        return this.call('list_saved_databases');
    }

    /**
     * Terminate the worker
     */
    terminate() {
        this.worker.terminate();
        // Reject all pending promises
        for (const { reject } of this.pending.values()) {
            reject(new Error('Worker terminated'));
        }
        this.pending.clear();
    }
}
