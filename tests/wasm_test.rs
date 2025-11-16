use wasm_bindgen_test::*;
use vecdb_wasm::{VectorDB, IndexType, Metric};

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_create_db() {
    let db = VectorDB::new(128, Metric::Cosine, IndexType::Flat);
    assert!(db.is_ok());
    let db = db.unwrap();
    assert_eq!(db.dimension(), 128);
    assert_eq!(db.len(), 0);
}

#[wasm_bindgen_test]
fn test_insert_and_search() {
    let mut db = VectorDB::new(3, Metric::Cosine, IndexType::Flat).unwrap();

    let v1 = vec![1.0, 0.0, 0.0];
    let v2 = vec![0.0, 1.0, 0.0];

    assert!(db.insert(1, v1.clone(), None).is_ok());
    assert!(db.insert(2, v2, None).is_ok());

    assert_eq!(db.len(), 2);

    let results = db.search(v1, 1, false);
    assert!(results.is_ok());
}

#[wasm_bindgen_test]
fn test_remove() {
    let mut db = VectorDB::new(2, Metric::Euclidean, IndexType::Flat).unwrap();

    db.insert(1, vec![1.0, 2.0], None).unwrap();
    assert_eq!(db.len(), 1);

    assert!(db.remove(1));
    assert_eq!(db.len(), 0);
}

#[wasm_bindgen_test]
fn test_hnsw_index() {
    let mut db = VectorDB::new(4, Metric::Cosine, IndexType::HNSW).unwrap();

    for i in 0..100 {
        let vec = vec![
            (i as f32) / 100.0,
            ((i + 1) as f32) / 100.0,
            ((i + 2) as f32) / 100.0,
            ((i + 3) as f32) / 100.0,
        ];
        db.insert(i, vec, None).unwrap();
    }

    assert_eq!(db.len(), 100);

    let query = vec![0.5, 0.5, 0.5, 0.5];
    let results = db.search(query, 5, false);
    assert!(results.is_ok());
}
