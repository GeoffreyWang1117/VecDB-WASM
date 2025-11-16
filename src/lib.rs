pub mod distance;
pub mod index;
pub mod storage;
pub mod bindings;
pub mod persistence;

pub use distance::{DistanceMetric, cosine_similarity, euclidean_distance};
pub use index::{Index, FlatIndex, SearchResult};
pub use storage::{Vector, VectorMetadata};
pub use bindings::VectorDB;
pub use persistence::{DatabaseSnapshot, IndexParams};

// WASM initialization
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn main() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[allow(unused_macros)]
macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}
