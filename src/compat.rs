use wasm_bindgen::prelude::*;
use web_sys;
use js_sys;

/// Browser compatibility information
#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct BrowserCompat {
    has_indexeddb: bool,
    has_wasm: bool,
    has_simd: bool,
    has_bigint64array: bool,
    browser_name: String,
    user_agent: String,
}

#[wasm_bindgen]
impl BrowserCompat {
    /// Check browser compatibility
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<BrowserCompat, JsValue> {
        let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window object"))?;
        let navigator = window.navigator();
        let user_agent = navigator.user_agent().unwrap_or_else(|_| "Unknown".to_string());

        // Detect browser name
        let browser_name = detect_browser(&user_agent);

        // Check IndexedDB support
        let has_indexeddb = window.indexed_db().is_ok();

        // WASM is always available if we're running this code
        let has_wasm = true;

        // Check SIMD support (this is a best-effort check)
        let has_simd = check_simd_support();

        // Check BigInt64Array support
        let has_bigint64array = js_sys::Reflect::has(
            &js_sys::global(),
            &JsValue::from_str("BigInt64Array"),
        )
        .unwrap_or(false);

        Ok(BrowserCompat {
            has_indexeddb,
            has_wasm,
            has_simd,
            has_bigint64array,
            browser_name,
            user_agent,
        })
    }

    #[wasm_bindgen(getter)]
    pub fn has_indexeddb(&self) -> bool {
        self.has_indexeddb
    }

    #[wasm_bindgen(getter)]
    pub fn has_wasm(&self) -> bool {
        self.has_wasm
    }

    #[wasm_bindgen(getter)]
    pub fn has_simd(&self) -> bool {
        self.has_simd
    }

    #[wasm_bindgen(getter)]
    pub fn has_bigint64array(&self) -> bool {
        self.has_bigint64array
    }

    #[wasm_bindgen(getter)]
    pub fn browser_name(&self) -> String {
        self.browser_name.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn user_agent(&self) -> String {
        self.user_agent.clone()
    }

    /// Check if all required features are supported
    #[wasm_bindgen]
    pub fn is_fully_compatible(&self) -> bool {
        self.has_wasm && self.has_indexeddb
    }

    /// Get compatibility report as JSON
    #[wasm_bindgen]
    pub fn get_report(&self) -> Result<JsValue, JsValue> {
        use serde::Serialize;

        #[derive(Serialize)]
        struct CompatReport {
            browser_name: String,
            has_indexeddb: bool,
            has_wasm: bool,
            has_simd: bool,
            has_bigint64array: bool,
            is_fully_compatible: bool,
            recommended_features: Vec<String>,
            missing_features: Vec<String>,
        }

        let mut recommended = Vec::new();
        let mut missing = Vec::new();

        if self.has_indexeddb {
            recommended.push("IndexedDB persistence".to_string());
        } else {
            missing.push("IndexedDB (persistence disabled)".to_string());
        }

        if self.has_simd {
            recommended.push("WASM SIMD (2-4x faster distance calculations)".to_string());
        } else {
            missing.push("WASM SIMD (will use scalar fallback)".to_string());
        }

        if self.has_bigint64array {
            recommended.push("BigInt64Array".to_string());
        }

        let report = CompatReport {
            browser_name: self.browser_name.clone(),
            has_indexeddb: self.has_indexeddb,
            has_wasm: self.has_wasm,
            has_simd: self.has_simd,
            has_bigint64array: self.has_bigint64array,
            is_fully_compatible: self.is_fully_compatible(),
            recommended_features: recommended,
            missing_features: missing,
        };

        serde_wasm_bindgen::to_value(&report)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Get warnings for the current browser
    #[wasm_bindgen]
    pub fn get_warnings(&self) -> Vec<JsValue> {
        let mut warnings = Vec::new();

        if !self.has_indexeddb {
            warnings.push(JsValue::from_str(
                "IndexedDB not supported. Persistence features will be disabled.",
            ));
        }

        if !self.has_simd {
            warnings.push(JsValue::from_str(
                "WASM SIMD not supported. Distance calculations will be slower (using scalar fallback).",
            ));
        }

        if self.browser_name.contains("Safari") && !self.browser_name.contains("Chrome") {
            warnings.push(JsValue::from_str(
                "Safari detected. Some IndexedDB features may have limitations.",
            ));
        }

        if self.browser_name == "IE" {
            warnings.push(JsValue::from_str(
                "Internet Explorer is not supported. Please use a modern browser.",
            ));
        }

        warnings
    }
}

/// Detect browser from user agent
fn detect_browser(user_agent: &str) -> String {
    let ua = user_agent.to_lowercase();

    if ua.contains("edg/") || ua.contains("edge/") {
        "Edge".to_string()
    } else if ua.contains("chrome") && !ua.contains("edg") {
        "Chrome".to_string()
    } else if ua.contains("firefox") {
        "Firefox".to_string()
    } else if ua.contains("safari") && !ua.contains("chrome") {
        "Safari".to_string()
    } else if ua.contains("opera") || ua.contains("opr/") {
        "Opera".to_string()
    } else if ua.contains("trident") || ua.contains("msie") {
        "IE".to_string()
    } else {
        "Unknown".to_string()
    }
}

/// Check if WASM SIMD is supported
fn check_simd_support() -> bool {
    // This is a runtime check - SIMD support is determined at compile time
    // For WASM, we check if the target feature is enabled
    #[cfg(all(target_arch = "wasm32", target_feature = "simd128"))]
    {
        true
    }
    #[cfg(not(all(target_arch = "wasm32", target_feature = "simd128")))]
    {
        false
    }
}

/// Check specific feature support
#[wasm_bindgen]
pub fn check_feature_support(feature: &str) -> bool {
    match feature {
        "indexeddb" => {
            if let Some(window) = web_sys::window() {
                window.indexed_db().is_ok()
            } else {
                false
            }
        }
        "wasm" => true, // If we're running, WASM is supported
        "simd" => check_simd_support(),
        "bigint" => js_sys::Reflect::has(
            &js_sys::global(),
            &JsValue::from_str("BigInt64Array"),
        )
        .unwrap_or(false),
        "crypto" => {
            if let Some(window) = web_sys::window() {
                js_sys::Reflect::has(&window, &JsValue::from_str("crypto")).unwrap_or(false)
            } else {
                false
            }
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_browser() {
        assert_eq!(
            detect_browser("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36"),
            "Chrome"
        );

        assert_eq!(
            detect_browser("Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:89.0) Gecko/20100101 Firefox/89.0"),
            "Firefox"
        );

        assert_eq!(
            detect_browser("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/14.1.1 Safari/605.1.15"),
            "Safari"
        );

        assert_eq!(
            detect_browser("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36 Edg/91.0.864.59"),
            "Edge"
        );
    }
}
