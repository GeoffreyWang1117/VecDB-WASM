/**
 * VecDB-WASM TypeScript Type Definitions
 * Version: 0.2.0
 *
 * A production-grade vector database for browsers using WebAssembly
 */

declare module 'vecdb-wasm' {
    /**
     * Initialize the WASM module.
     * Must be called before using any other functions.
     *
     * @returns Promise that resolves when WASM is initialized
     *
     * @example
     * ```typescript
     * import init from 'vecdb-wasm';
     * await init();
     * ```
     */
    export default function init(module_or_path?: Promise<Response | BufferSource> | Response | BufferSource): Promise<void>;

    /**
     * Distance metric for vector similarity calculation
     */
    export enum Metric {
        /** Cosine similarity (1 - cosine distance). Best for text embeddings. Range: 0-2 */
        Cosine = 0,
        /** Euclidean (L2) distance. Best for general use. Range: 0-∞ */
        Euclidean = 1,
        /** Dot product. Best for recommendation systems. Range: -∞ to ∞ */
        DotProduct = 2,
    }

    /**
     * Index type for the vector database
     */
    export enum IndexType {
        /** Brute-force exact search. O(n) complexity. Best for <10K vectors. 100% recall. */
        Flat = 0,
        /** Hierarchical Navigable Small World graph. O(log n) complexity. Best for >10K vectors. ~95% recall. */
        HNSW = 1,
    }

    /**
     * Search result returned by search operations
     */
    export interface SearchResult {
        /** Unique vector ID */
        readonly id: number;
        /** Similarity score (lower is closer for distance metrics) */
        readonly score: number;
        /** Optional JSON string containing metadata */
        readonly metadata?: string;
    }

    /**
     * Vector data structure for batch operations
     */
    export interface VectorData {
        /** Unique vector ID */
        id: number;
        /** Vector data as array of numbers */
        vector: number[] | Float32Array;
        /** Optional JSON string containing metadata */
        metadata?: string;
    }

    /**
     * Database statistics
     */
    export interface DatabaseStats {
        /** Database version */
        version: string;
        /** Vector dimension */
        dimension: number;
        /** Number of vectors in database */
        count: number;
        /** Index type ("Flat" or "HNSW") */
        index_type: string;
        /** Distance metric used */
        metric: string;
        /** Estimated memory usage in bytes */
        memory_bytes: number;
        /** HNSW M parameter (only for HNSW index) */
        hnsw_m?: number;
        /** HNSW ef_construction parameter (only for HNSW index) */
        hnsw_ef?: number;
    }

    /**
     * Browser compatibility report
     */
    export interface CompatibilityReport {
        /** Browser name */
        browser_name: string;
        /** Full user agent string */
        user_agent: string;
        /** WebAssembly support */
        has_wasm: boolean;
        /** IndexedDB support */
        has_indexeddb: boolean;
        /** WASM SIMD support */
        has_simd: boolean;
        /** BigInt64Array support */
        has_bigint64array: boolean;
    }

    /**
     * Main vector database class
     */
    export class VectorDB {
        /**
         * Create a new vector database instance
         *
         * @param dimension - Vector dimension (must be positive integer)
         * @param metric - Distance metric to use
         * @param indexType - Index type (Flat or HNSW)
         * @throws Error if parameters are invalid
         *
         * @example
         * ```typescript
         * const db = new VectorDB(128, Metric.Cosine, IndexType.HNSW);
         * ```
         */
        constructor(dimension: number, metric: Metric, indexType: IndexType);

        /**
         * Create a new vector database with custom HNSW parameters
         *
         * @param dimension - Vector dimension
         * @param metric - Distance metric
         * @param m - Maximum connections per node (default: 16, recommended: 8-64)
         * @param ef_construction - Search quality during build (default: 200, recommended: 100-500)
         * @returns VectorDB instance
         *
         * @example
         * ```typescript
         * // High recall configuration
         * const db = VectorDB.new_with_hnsw_params(128, Metric.Cosine, 32, 400);
         * ```
         */
        static new_with_hnsw_params(
            dimension: number,
            metric: Metric,
            m: number,
            ef_construction: number
        ): VectorDB;

        /**
         * Insert a single vector into the database
         *
         * @param id - Unique vector ID
         * @param vector - Vector data
         * @param metadata - Optional JSON string containing metadata
         * @throws Error if vector dimension doesn't match or ID already exists
         *
         * @example
         * ```typescript
         * const vector = new Float32Array(128);
         * const metadata = JSON.stringify({ title: "Example", category: "A" });
         * db.insert(1, Array.from(vector), metadata);
         * ```
         */
        insert(id: number, vector: number[] | Float32Array, metadata?: string): void;

        /**
         * Insert multiple vectors efficiently
         *
         * @param vectors - Array of vector data objects
         * @returns Number of vectors inserted
         *
         * @example
         * ```typescript
         * const vectors = [
         *   { id: 1, vector: [...], metadata: '{"type":"A"}' },
         *   { id: 2, vector: [...], metadata: '{"type":"B"}' }
         * ];
         * const count = db.batch_insert(vectors);
         * ```
         */
        batch_insert(vectors: VectorData[]): number;

        /**
         * Search for k nearest neighbors
         *
         * @param query - Query vector
         * @param k - Number of results to return
         * @param include_metadata - Whether to include metadata in results (default: false)
         * @returns Array of search results
         *
         * @example
         * ```typescript
         * const results = db.search(queryVector, 10, true);
         * results.forEach(r => {
         *   console.log(`ID: ${r.id}, Score: ${r.score}`);
         * });
         * ```
         */
        search(query: number[] | Float32Array, k: number, include_metadata?: boolean): SearchResult[];

        /**
         * Search with metadata filtering
         *
         * @param query - Query vector
         * @param k - Number of results to return
         * @param filter - Optional JSON filter object (exact match)
         * @param include_metadata - Whether to include metadata in results
         * @returns Array of search results
         *
         * @example
         * ```typescript
         * const filter = JSON.stringify({ category: "tech", status: "active" });
         * const results = db.search_with_filter(query, 10, filter, true);
         * ```
         */
        search_with_filter(
            query: number[] | Float32Array,
            k: number,
            filter?: string,
            include_metadata?: boolean
        ): SearchResult[];

        /**
         * Remove a vector by ID
         *
         * @param id - Vector ID to remove
         * @returns true if removed, false if not found
         *
         * @example
         * ```typescript
         * if (db.remove(42)) {
         *   console.log('Vector removed');
         * }
         * ```
         */
        remove(id: number): boolean;

        /**
         * Remove all vectors from the database
         *
         * @example
         * ```typescript
         * db.clear();
         * console.log(`Database has ${db.len()} vectors`); // 0
         * ```
         */
        clear(): void;

        /**
         * Retrieve a vector by ID
         *
         * @param id - Vector ID
         * @returns Vector data or null if not found
         *
         * @example
         * ```typescript
         * const vector = db.get_vector(42);
         * if (vector) {
         *   console.log('Vector dimensions:', vector.length);
         * }
         * ```
         */
        get_vector(id: number): number[] | null;

        /**
         * Get the number of vectors in the database
         *
         * @returns Number of vectors
         *
         * @example
         * ```typescript
         * console.log(`Database contains ${db.len()} vectors`);
         * ```
         */
        len(): number;

        /**
         * Check if database is empty
         *
         * @returns true if empty, false otherwise
         */
        is_empty(): boolean;

        /**
         * Get the vector dimension
         *
         * @returns Vector dimension
         */
        dimension(): number;

        /**
         * Get comprehensive database statistics
         *
         * @returns Database statistics object
         *
         * @example
         * ```typescript
         * const stats = db.get_stats();
         * console.log(`DB: ${stats.count} vectors of ${stats.dimension}D`);
         * console.log(`Memory: ${(stats.memory_bytes / 1024 / 1024).toFixed(2)} MB`);
         * ```
         */
        get_stats(): DatabaseStats;

        /**
         * Export database to binary format
         *
         * @returns Binary snapshot as Uint8Array
         *
         * @example
         * ```typescript
         * const snapshot = db.export_snapshot();
         * // Save to file or localStorage
         * ```
         */
        export_snapshot(): Uint8Array;

        /**
         * Export database to JSON format
         *
         * @returns JSON snapshot as string
         *
         * @example
         * ```typescript
         * const json = db.export_snapshot_json();
         * const blob = new Blob([json], { type: 'application/json' });
         * ```
         */
        export_snapshot_json(): string;

        /**
         * Import database from binary format (static method)
         *
         * @param data - Binary snapshot data
         * @returns VectorDB instance
         *
         * @example
         * ```typescript
         * const data = new Uint8Array(binaryData);
         * const db = VectorDB.import_snapshot(data);
         * ```
         */
        static import_snapshot(data: Uint8Array): VectorDB;

        /**
         * Import database from JSON format (static method)
         *
         * @param json - JSON snapshot string
         * @returns VectorDB instance
         *
         * @example
         * ```typescript
         * const json = await fetch('database.json').then(r => r.text());
         * const db = VectorDB.import_snapshot_json(json);
         * ```
         */
        static import_snapshot_json(json: string): VectorDB;

        /**
         * Save database to IndexedDB
         *
         * @param db_name - Database name
         * @returns Promise that resolves when saved
         *
         * @example
         * ```typescript
         * await db.save_to_indexeddb('my_vectors');
         * console.log('Database saved');
         * ```
         */
        save_to_indexeddb(db_name: string): Promise<void>;

        /**
         * Load database from IndexedDB (static method)
         *
         * @param db_name - Database name
         * @returns Promise with binary data to pass to import_snapshot()
         *
         * @example
         * ```typescript
         * const data = await VectorDB.load_from_indexeddb('my_vectors');
         * const db = VectorDB.import_snapshot(data);
         * ```
         */
        static load_from_indexeddb(db_name: string): Promise<Uint8Array>;

        /**
         * Delete a database from IndexedDB (static method)
         *
         * @param db_name - Database name
         * @returns Promise that resolves when deleted
         *
         * @example
         * ```typescript
         * await VectorDB.delete_from_indexeddb('old_database');
         * ```
         */
        static delete_from_indexeddb(db_name: string): Promise<void>;

        /**
         * List all saved databases in IndexedDB (static method)
         *
         * @returns Promise with array of database names
         *
         * @example
         * ```typescript
         * const databases = await VectorDB.list_saved_databases();
         * console.log('Saved databases:', databases);
         * ```
         */
        static list_saved_databases(): Promise<string[]>;
    }

    /**
     * Browser compatibility checker
     */
    export class BrowserCompat {
        /** Browser name (e.g., "Chrome", "Firefox", "Safari") */
        readonly browser_name: string;
        /** Full user agent string */
        readonly user_agent: string;
        /** WebAssembly support */
        readonly has_wasm: boolean;
        /** IndexedDB support */
        readonly has_indexeddb: boolean;
        /** WASM SIMD support */
        readonly has_simd: boolean;
        /** BigInt64Array support */
        readonly has_bigint64array: boolean;

        /**
         * Create a new browser compatibility checker
         *
         * @example
         * ```typescript
         * const compat = new BrowserCompat();
         * console.log('Browser:', compat.browser_name);
         * console.log('SIMD:', compat.has_simd);
         * ```
         */
        constructor();

        /**
         * Get full compatibility report
         *
         * @returns Compatibility report object
         */
        get_report(): CompatibilityReport;

        /**
         * Get array of warning messages
         *
         * @returns Array of warning strings
         *
         * @example
         * ```typescript
         * const warnings = compat.get_warnings();
         * if (warnings.length > 0) {
         *   console.warn('Compatibility issues:', warnings);
         * }
         * ```
         */
        get_warnings(): string[];

        /**
         * Check if fully compatible
         *
         * @returns true if all features are supported
         *
         * @example
         * ```typescript
         * if (!compat.is_fully_compatible()) {
         *   alert('Some features may not be available');
         * }
         * ```
         */
        is_fully_compatible(): boolean;
    }

    /**
     * Initialize panic hook for better error messages (development only)
     *
     * @example
     * ```typescript
     * if (process.env.NODE_ENV === 'development') {
     *   init_panic_hook();
     * }
     * ```
     */
    export function init_panic_hook(): void;

    /**
     * Get VecDB-WASM version
     *
     * @returns Version string
     *
     * @example
     * ```typescript
     * import { version } from 'vecdb-wasm';
     * console.log(`VecDB-WASM version: ${version()}`);
     * ```
     */
    export function version(): string;
}
