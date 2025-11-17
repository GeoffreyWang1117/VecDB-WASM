/**
 * VecDB-WASM Web Worker
 *
 * This worker script enables non-blocking vector database operations
 * by running them in a background thread.
 */

// Import the WASM module
importScripts('../pkg/vecdb_wasm.js');

let db = null;
let initialized = false;

// Initialize WASM
async function initWasm() {
    if (!initialized) {
        await wasm_bindgen('../pkg/vecdb_wasm_bg.wasm');
        initialized = true;
    }
}

// Message handler
self.onmessage = async (event) => {
    const { id, method, params } = event.data;

    try {
        // Initialize on first message
        if (!initialized) {
            await initWasm();
        }

        let result;

        switch (method) {
            case 'create':
                db = new wasm_bindgen.VectorDB(
                    params.dimension,
                    params.metric,
                    params.indexType
                );
                result = { success: true };
                break;

            case 'create_with_params':
                db = wasm_bindgen.VectorDB.new_with_hnsw_params(
                    params.dimension,
                    params.metric,
                    params.m,
                    params.ef_construction
                );
                result = { success: true };
                break;

            case 'insert':
                db.insert(params.id, params.vector, params.metadata);
                result = { success: true };
                break;

            case 'batch_insert':
                const count = db.batch_insert(params.vectors);
                result = { count };
                break;

            case 'search':
                const results = db.search(
                    params.query,
                    params.k,
                    params.include_metadata || false
                );
                result = results;
                break;

            case 'search_with_filter':
                const filtered = db.search_with_filter(
                    params.query,
                    params.k,
                    params.filter,
                    params.include_metadata || false
                );
                result = filtered;
                break;

            case 'batch_search':
                const batchResults = db.batch_search(
                    params.queries,
                    params.k,
                    params.include_metadata || false
                );
                result = batchResults;
                break;

            case 'search_radius':
                const radiusResults = db.search_radius(
                    params.query,
                    params.radius,
                    params.max_results,
                    params.include_metadata || false
                );
                result = radiusResults;
                break;

            case 'remove':
                const removed = db.remove(params.id);
                result = { removed };
                break;

            case 'clear':
                db.clear();
                result = { success: true };
                break;

            case 'get_vector':
                const vector = db.get_vector(params.id);
                result = vector;
                break;

            case 'len':
                result = { length: db.len() };
                break;

            case 'is_empty':
                result = { empty: db.is_empty() };
                break;

            case 'dimension':
                result = { dimension: db.dimension() };
                break;

            case 'get_stats':
                result = db.get_stats();
                break;

            case 'get_performance_metrics':
                result = db.get_performance_metrics();
                break;

            case 'reset_performance_metrics':
                db.reset_performance_metrics();
                result = { success: true };
                break;

            case 'export_snapshot':
                const snapshot = db.export_snapshot();
                result = snapshot;
                break;

            case 'export_snapshot_json':
                const json = db.export_snapshot_json();
                result = json;
                break;

            case 'import_snapshot':
                db = wasm_bindgen.VectorDB.import_snapshot(params.data);
                result = { success: true };
                break;

            case 'import_snapshot_json':
                db = wasm_bindgen.VectorDB.import_snapshot_json(params.json);
                result = { success: true };
                break;

            case 'save_to_indexeddb':
                await db.save_to_indexeddb(params.db_name);
                result = { success: true };
                break;

            case 'load_from_indexeddb':
                const data = await wasm_bindgen.VectorDB.load_from_indexeddb(params.db_name);
                db = wasm_bindgen.VectorDB.import_snapshot(data);
                result = { success: true };
                break;

            case 'delete_from_indexeddb':
                await wasm_bindgen.VectorDB.delete_from_indexeddb(params.db_name);
                result = { success: true };
                break;

            case 'list_saved_databases':
                const databases = await wasm_bindgen.VectorDB.list_saved_databases();
                result = databases;
                break;

            default:
                throw new Error(`Unknown method: ${method}`);
        }

        // Send success response
        self.postMessage({
            id,
            success: true,
            result
        });

    } catch (error) {
        // Send error response
        self.postMessage({
            id,
            success: false,
            error: error.message || String(error)
        });
    }
};

// Handle errors
self.onerror = (error) => {
    console.error('Worker error:', error);
};
