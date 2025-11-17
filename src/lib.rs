pub mod bindings;
pub mod compat;
pub mod distance;
pub mod index;
pub mod integrity;
pub mod performance;
pub mod persistence;
pub mod storage;

pub use bindings::{IndexType, Metric, VectorDB};
pub use compat::BrowserCompat;
pub use distance::{cosine_similarity, euclidean_distance, DistanceMetric};
pub use index::{FlatIndex, Index, SearchResult};
pub use integrity::{IntegrityChecker, IntegrityReport};
pub use performance::PerformanceMetrics;
pub use persistence::{DatabaseSnapshot, IndexParams};
pub use storage::{Vector, VectorMetadata};

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
