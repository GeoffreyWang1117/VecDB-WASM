use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{IdbDatabase, IdbFactory, IdbOpenDbRequest, IdbTransactionMode, IdbVersionChangeEvent};
use js_sys::{Uint8Array, Array, Promise};

const DB_NAME: &str = "VecDB";
const DB_VERSION: u32 = 1;
const STORE_NAME: &str = "databases";

/// IndexedDB wrapper for persistent storage
pub struct IndexedDBStore {
    db: Option<IdbDatabase>,
}

impl IndexedDBStore {
    pub fn new() -> Self {
        Self { db: None }
    }

    /// Open or create the database
    pub async fn open(&mut self) -> Result<(), JsValue> {
        let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window object"))?;

        // Check if IndexedDB is supported
        let idb_factory: IdbFactory = match window.indexed_db() {
            Ok(Some(factory)) => factory,
            Ok(None) => {
                return Err(JsValue::from_str(
                    "IndexedDB not available in this browser. Please use a modern browser like Chrome, Firefox, or Edge."
                ));
            }
            Err(_) => {
                return Err(JsValue::from_str(
                    "IndexedDB not supported in this browser. Persistence features are disabled."
                ));
            }
        };

        let open_request: IdbOpenDbRequest = idb_factory
            .open_with_u32(DB_NAME, DB_VERSION)
            .map_err(|e| {
                JsValue::from_str(&format!(
                    "Failed to open IndexedDB. This may be due to browser privacy settings or incognito mode. Error: {:?}",
                    e
                ))
            })?;

        // Set up onupgradeneeded callback
        let onupgradeneeded = Closure::once(move |event: &IdbVersionChangeEvent| {
            if let Some(target) = event.target() {
                if let Ok(request) = target.dyn_into::<IdbOpenDbRequest>() {
                    if let Ok(result) = request.result() {
                        if let Ok(db) = result.dyn_into::<IdbDatabase>() {
                            // Always try to create the store - will fail silently if exists
                            let _ = db.create_object_store(STORE_NAME);
                        }
                    }
                }
            }
        });

        open_request.set_onupgradeneeded(Some(onupgradeneeded.as_ref().unchecked_ref()));
        onupgradeneeded.forget();

        // Create promise for open request
        let promise = Promise::new(&mut |resolve, reject| {
            let request_clone = open_request.clone();
            let success = Closure::once(move || {
                if let Ok(result) = request_clone.result() {
                    resolve.call1(&JsValue::NULL, &result).unwrap();
                }
            });
            let error = Closure::once(move |e: JsValue| {
                reject.call1(&JsValue::NULL, &e).unwrap();
            });

            open_request.set_onsuccess(Some(success.as_ref().unchecked_ref()));
            open_request.set_onerror(Some(error.as_ref().unchecked_ref()));

            success.forget();
            error.forget();
        });

        let db_result = JsFuture::from(promise).await?;
        self.db = Some(db_result.dyn_into::<IdbDatabase>()?);

        Ok(())
    }

    /// Save data to IndexedDB
    pub async fn save(&self, key: &str, data: &[u8]) -> Result<(), JsValue> {
        let db = self.db.as_ref().ok_or_else(|| JsValue::from_str("Database not opened"))?;

        let transaction = db
            .transaction_with_str_and_mode(STORE_NAME, IdbTransactionMode::Readwrite)
            .map_err(|_| JsValue::from_str("Failed to create transaction"))?;

        let store = transaction
            .object_store(STORE_NAME)
            .map_err(|_| JsValue::from_str("Failed to get object store"))?;

        // Convert data to Uint8Array
        let array = Uint8Array::new_with_length(data.len() as u32);
        array.copy_from(data);

        let request = store
            .put_with_key(&array, &JsValue::from_str(key))
            .map_err(|_| JsValue::from_str("Failed to put data"))?;

        let promise = Promise::new(&mut |resolve, reject| {
            let success = Closure::once(move || {
                resolve.call0(&JsValue::NULL).unwrap();
            });
            let error = Closure::once(move |e: JsValue| {
                reject.call1(&JsValue::NULL, &e).unwrap();
            });

            request.set_onsuccess(Some(success.as_ref().unchecked_ref()));
            request.set_onerror(Some(error.as_ref().unchecked_ref()));

            success.forget();
            error.forget();
        });

        JsFuture::from(promise).await?;
        Ok(())
    }

    /// Load data from IndexedDB
    pub async fn load(&self, key: &str) -> Result<Vec<u8>, JsValue> {
        let db = self.db.as_ref().ok_or_else(|| JsValue::from_str("Database not opened"))?;

        let transaction = db
            .transaction_with_str(STORE_NAME)
            .map_err(|_| JsValue::from_str("Failed to create transaction"))?;

        let store = transaction
            .object_store(STORE_NAME)
            .map_err(|_| JsValue::from_str("Failed to get object store"))?;

        let request = store
            .get(&JsValue::from_str(key))
            .map_err(|_| JsValue::from_str("Failed to get data"))?;

        let promise = Promise::new(&mut |resolve, reject| {
            let req_clone = request.clone();
            let success = Closure::once(move || {
                if let Ok(result) = req_clone.result() {
                    resolve.call1(&JsValue::NULL, &result).unwrap();
                }
            });
            let error = Closure::once(move |e: JsValue| {
                reject.call1(&JsValue::NULL, &e).unwrap();
            });

            request.set_onsuccess(Some(success.as_ref().unchecked_ref()));
            request.set_onerror(Some(error.as_ref().unchecked_ref()));

            success.forget();
            error.forget();
        });

        let result = JsFuture::from(promise).await?;

        if result.is_undefined() || result.is_null() {
            return Err(JsValue::from_str("Key not found"));
        }

        // Convert from Uint8Array back to Vec<u8>
        let array = result.dyn_into::<Uint8Array>()?;
        Ok(array.to_vec())
    }

    /// Delete data from IndexedDB
    pub async fn delete(&self, key: &str) -> Result<(), JsValue> {
        let db = self.db.as_ref().ok_or_else(|| JsValue::from_str("Database not opened"))?;

        let transaction = db
            .transaction_with_str_and_mode(STORE_NAME, IdbTransactionMode::Readwrite)
            .map_err(|_| JsValue::from_str("Failed to create transaction"))?;

        let store = transaction
            .object_store(STORE_NAME)
            .map_err(|_| JsValue::from_str("Failed to get object store"))?;

        let request = store
            .delete(&JsValue::from_str(key))
            .map_err(|_| JsValue::from_str("Failed to delete data"))?;

        let promise = Promise::new(&mut |resolve, reject| {
            let success = Closure::once(move || {
                resolve.call0(&JsValue::NULL).unwrap();
            });
            let error = Closure::once(move |e: JsValue| {
                reject.call1(&JsValue::NULL, &e).unwrap();
            });

            request.set_onsuccess(Some(success.as_ref().unchecked_ref()));
            request.set_onerror(Some(error.as_ref().unchecked_ref()));

            success.forget();
            error.forget();
        });

        JsFuture::from(promise).await?;
        Ok(())
    }

    /// List all keys in the database
    pub async fn list_keys(&self) -> Result<Vec<String>, JsValue> {
        let db = self.db.as_ref().ok_or_else(|| JsValue::from_str("Database not opened"))?;

        let transaction = db
            .transaction_with_str(STORE_NAME)
            .map_err(|_| JsValue::from_str("Failed to create transaction"))?;

        let store = transaction
            .object_store(STORE_NAME)
            .map_err(|_| JsValue::from_str("Failed to get object store"))?;

        let request = store
            .get_all_keys()
            .map_err(|_| JsValue::from_str("Failed to get keys"))?;

        let promise = Promise::new(&mut |resolve, reject| {
            let req_clone = request.clone();
            let success = Closure::once(move || {
                if let Ok(result) = req_clone.result() {
                    resolve.call1(&JsValue::NULL, &result).unwrap();
                }
            });
            let error = Closure::once(move |e: JsValue| {
                reject.call1(&JsValue::NULL, &e).unwrap();
            });

            request.set_onsuccess(Some(success.as_ref().unchecked_ref()));
            request.set_onerror(Some(error.as_ref().unchecked_ref()));

            success.forget();
            error.forget();
        });

        let result = JsFuture::from(promise).await?;
        let array = result.dyn_into::<Array>()?;

        let mut keys = Vec::new();
        for i in 0..array.length() {
            if let Some(key) = array.get(i).as_string() {
                keys.push(key);
            }
        }

        Ok(keys)
    }

    /// Check if a key exists
    pub async fn exists(&self, key: &str) -> Result<bool, JsValue> {
        match self.load(key).await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Clear all data from the store
    pub async fn clear(&self) -> Result<(), JsValue> {
        let db = self.db.as_ref().ok_or_else(|| JsValue::from_str("Database not opened"))?;

        let transaction = db
            .transaction_with_str_and_mode(STORE_NAME, IdbTransactionMode::Readwrite)
            .map_err(|_| JsValue::from_str("Failed to create transaction"))?;

        let store = transaction
            .object_store(STORE_NAME)
            .map_err(|_| JsValue::from_str("Failed to get object store"))?;

        let request = store
            .clear()
            .map_err(|_| JsValue::from_str("Failed to clear store"))?;

        let promise = Promise::new(&mut |resolve, reject| {
            let success = Closure::once(move || {
                resolve.call0(&JsValue::NULL).unwrap();
            });
            let error = Closure::once(move |e: JsValue| {
                reject.call1(&JsValue::NULL, &e).unwrap();
            });

            request.set_onsuccess(Some(success.as_ref().unchecked_ref()));
            request.set_onerror(Some(error.as_ref().unchecked_ref()));

            success.forget();
            error.forget();
        });

        JsFuture::from(promise).await?;
        Ok(())
    }
}

impl Default for IndexedDBStore {
    fn default() -> Self {
        Self::new()
    }
}
