# Browser Compatibility Guide

## Supported Browsers

### ✅ Fully Supported (Recommended)

| Browser | Minimum Version | SIMD Support | IndexedDB | Notes |
|---------|----------------|--------------|-----------|-------|
| **Chrome** | 91+ | ✅ Yes (flag) | ✅ Yes | Best performance |
| **Edge** | 91+ | ✅ Yes (flag) | ✅ Yes | Best performance |
| **Firefox** | 89+ | ✅ Yes (default) | ✅ Yes | Best performance |
| **Safari** | 15+ | ⚠️ Partial | ✅ Yes | Good performance |
| **Opera** | 77+ | ✅ Yes (flag) | ✅ Yes | Good performance |

### ⚠️ Partially Supported

| Browser | Limitation | Workaround |
|---------|-----------|------------|
| **Safari < 15** | No SIMD | Uses scalar fallback (slower) |
| **Mobile Safari** | Limited IndexedDB | May have storage limitations |
| **Private/Incognito Mode** | IndexedDB disabled | In-memory only, no persistence |

### ❌ Not Supported

- **Internet Explorer** (all versions) - No WebAssembly support
- **Very old browsers** (Chrome <57, Firefox <52, Safari <11)

---

## Feature Detection

VecDB-WASM includes automatic feature detection:

```javascript
import { BrowserCompat } from './pkg/vecdb_wasm.js';

const compat = new BrowserCompat();

// Check specific features
console.log(compat.has_indexeddb);  // true/false
console.log(compat.has_simd);       // true/false
console.log(compat.has_wasm);       // true/false

// Get full compatibility report
const report = compat.get_report();
console.log(report);

// Get warnings
const warnings = compat.get_warnings();
warnings.forEach(w => console.warn(w));
```

---

## Enabling SIMD (for 2-4x Performance Boost)

### Chrome/Edge (91+)
1. Navigate to `chrome://flags` or `edge://flags`
2. Search for "WebAssembly SIMD"
3. Enable the flag
4. Restart browser

**OR** use Chrome 92+ where SIMD is enabled by default

### Firefox (89+)
SIMD is **enabled by default** in Firefox 89+. No configuration needed!

### Safari (15+)
SIMD support is limited. Check Apple's release notes for updates.

---

## Handling Browser Differences

### Graceful Degradation

VecDB-WASM automatically falls back to scalar implementations when SIMD is unavailable:

```javascript
// No code changes needed - automatic fallback
const db = new VectorDB(128, Metric.Cosine, IndexType.HNSW);
// Works in all supported browsers
```

### IndexedDB Availability

Check before using persistence features:

```javascript
const compat = new BrowserCompat();

if (compat.has_indexeddb) {
    // Safe to use persistence
    await db.save_to_indexeddb("myDB");
} else {
    // Use in-memory only or export/import
    console.warn("IndexedDB not available - using in-memory storage");
    const snapshot = db.export_snapshot();
    // Save snapshot to server or localStorage
}
```

### Private/Incognito Mode

IndexedDB is typically disabled in private browsing:

```javascript
try {
    await db.save_to_indexeddb("myDB");
} catch (error) {
    if (error.toString().includes("IndexedDB")) {
        // Fallback to export/download
        const json = db.export_snapshot_json();
        downloadFile(json, "vecdb_backup.json");
    }
}
```

---

## Performance Expectations

### With SIMD (Optimal)
- Insert: 2,000-3,000 vectors/sec (128D)
- Search: 0.5-2ms (10K vectors, k=10)
- Distance calculations: 2-4x faster

### Without SIMD (Fallback)
- Insert: 1,000-1,500 vectors/sec (128D)
- Search: 2-8ms (10K vectors, k=10)
- Distance calculations: Standard performance

### Mobile Browsers
- Performance: 30-50% slower than desktop
- Memory: More constrained
- IndexedDB: May have storage quotas

---

## Testing Compatibility

Use the built-in compatibility checker:

1. Open `examples/compatibility.html`
2. View feature detection results
3. Run performance test
4. See browser-specific recommendations

Or programmatically:

```javascript
import { BrowserCompat, check_feature_support } from './pkg/vecdb_wasm.js';

const compat = new BrowserCompat();

if (!compat.is_fully_compatible()) {
    const warnings = compat.get_warnings();
    warnings.forEach(w => {
        // Display warning to user
        alert(w);
    });
}

// Check individual features
if (!check_feature_support("simd")) {
    console.warn("SIMD not available - performance will be reduced");
}
```

---

## Known Issues

### Safari
- **Issue**: SIMD support limited
- **Impact**: 2-4x slower distance calculations
- **Workaround**: None currently, wait for Safari updates

### Private Browsing
- **Issue**: IndexedDB disabled
- **Impact**: No persistence
- **Workaround**: Use export/import with localStorage or downloads

### Mobile Browsers
- **Issue**: Memory constraints
- **Impact**: Lower maximum vector count
- **Workaround**: Use smaller batches, implement pagination

### Older Chrome (< 92)
- **Issue**: SIMD requires manual flag enable
- **Impact**: Users may not have SIMD enabled
- **Workaround**: Detect and show instructions to enable

---

## Recommended Setup

### For Development
```javascript
// Always check compatibility first
const compat = new BrowserCompat();
console.log(compat.get_report());

// Enable all warnings
if (!compat.is_fully_compatible()) {
    const warnings = compat.get_warnings();
    warnings.forEach(w => console.warn(w));
}
```

### For Production
```javascript
// Show user-friendly messages
const compat = new BrowserCompat();

if (!compat.has_wasm) {
    showError("Your browser doesn't support WebAssembly. Please upgrade to a modern browser.");
    return;
}

if (!compat.has_indexeddb) {
    showWarning("Persistence features disabled. Data will be lost on page refresh.");
}

if (!compat.has_simd) {
    showInfo("For best performance, enable WASM SIMD in your browser.");
}
```

---

## Browser Update Recommendations

### For End Users
1. **Chrome/Edge**: Update to latest (auto-updates)
2. **Firefox**: Update to 89+ (auto-updates)
3. **Safari**: Update to macOS 12+ / iOS 15+
4. **Mobile**: Keep OS and browser updated

### For Developers
- Test in **multiple browsers**
- Use **feature detection**, not browser detection
- Provide **clear error messages**
- Implement **graceful degradation**

---

## Debugging Compatibility Issues

### Check Browser Console

```javascript
// Detailed diagnostics
import { BrowserCompat } from './pkg/vecdb_wasm.js';

const compat = new BrowserCompat();
const report = compat.get_report();

console.table({
    Browser: report.browser_name,
    'Has WASM': report.has_wasm,
    'Has IndexedDB': report.has_indexeddb,
    'Has SIMD': report.has_simd,
    'Fully Compatible': report.is_fully_compatible
});
```

### Common Error Messages

| Error | Cause | Solution |
|-------|-------|----------|
| "IndexedDB not available" | Private mode or unsupported browser | Use export/import or update browser |
| "WASM initialization failed" | Very old browser | Update to modern browser |
| "Quota exceeded" | IndexedDB storage full | Clear browser data or use smaller datasets |

---

## Future Compatibility

We actively monitor browser developments:

- **WebGPU**: Planned for v0.4.0 (Chrome 113+)
- **Shared Array Buffer**: Being evaluated for multi-threading
- **File System Access API**: Considered for v0.5.0

Stay updated by checking our [CHANGELOG](CHANGELOG.md).
