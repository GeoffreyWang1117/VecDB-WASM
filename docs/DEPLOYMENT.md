# VecDB-WASM Deployment Guide

Production deployment guide for VecDB-WASM applications.

## Table of Contents

- [Build for Production](#build-for-production)
- [Hosting Options](#hosting-options)
- [CDN Integration](#cdn-integration)
- [Performance Optimization](#performance-optimization)
- [Security Considerations](#security-considerations)
- [Monitoring](#monitoring)
- [Troubleshooting](#troubleshooting)

---

## Build for Production

### 1. Build Optimized WASM

```bash
# Build with maximum optimizations
wasm-pack build --target web --release

# The output is in ./pkg/
# Key files:
#   - vecdb_wasm.js       (~20KB gzipped)
#   - vecdb_wasm_bg.wasm  (~150KB gzipped)
```

### 2. Further Optimize (Optional)

```bash
# Install wasm-opt (from binaryen)
# Ubuntu/Debian
sudo apt-get install binaryen

# macOS
brew install binaryen

# Optimize WASM file
wasm-opt -Oz -o pkg/vecdb_wasm_bg.wasm pkg/vecdb_wasm_bg.wasm

# Can reduce file size by additional 10-20%
```

### 3. Verify Build

```bash
# Check file sizes
ls -lh pkg/

# Expected output (approximate):
# vecdb_wasm.js         ~60KB (20KB gzipped)
# vecdb_wasm_bg.wasm    ~450KB (150KB gzipped)
# vecdb_wasm_bg.wasm.d.ts
# vecdb_wasm.d.ts
# package.json
```

---

## Hosting Options

### Option 1: Static Site Hosting

Perfect for Jamstack deployments.

#### Netlify

```toml
# netlify.toml
[build]
  publish = "dist"
  command = "npm run build"

[[headers]]
  for = "/*.wasm"
  [headers.values]
    Content-Type = "application/wasm"
    Cache-Control = "public, max-age=31536000, immutable"

[[headers]]
  for = "/*.js"
  [headers.values]
    Cache-Control = "public, max-age=31536000, immutable"
```

Deploy:
```bash
# Install Netlify CLI
npm install -g netlify-cli

# Deploy
netlify deploy --prod
```

#### Vercel

```json
// vercel.json
{
  "headers": [
    {
      "source": "/(.*).wasm",
      "headers": [
        {
          "key": "Content-Type",
          "value": "application/wasm"
        },
        {
          "key": "Cache-Control",
          "value": "public, max-age=31536000, immutable"
        }
      ]
    }
  ]
}
```

Deploy:
```bash
# Install Vercel CLI
npm install -g vercel

# Deploy
vercel --prod
```

#### GitHub Pages

```yaml
# .github/workflows/deploy.yml
name: Deploy to GitHub Pages

on:
  push:
    branches: [main]

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Install wasm-pack
        run: cargo install wasm-pack

      - name: Build WASM
        run: wasm-pack build --target web --release

      - name: Deploy to GitHub Pages
        uses: peaceiris/actions-gh-pages@v3
        with:
          github_token: ${{ secrets.GITHUB_TOKEN }}
          publish_dir: ./
```

### Option 2: CDN Distribution

#### Cloudflare

```javascript
// wrangler.toml for Cloudflare Workers
name = "vecdb-wasm-app"
type = "webpack"
workers_dev = true

[site]
bucket = "./dist"
```

#### AWS S3 + CloudFront

```bash
# Build
wasm-pack build --target web --release

# Upload to S3
aws s3 sync ./pkg s3://your-bucket/vecdb/ --cache-control "max-age=31536000"

# Configure CloudFront distribution
# - Origin: S3 bucket
# - Compress objects: Yes
# - Cached HTTP Methods: GET, HEAD, OPTIONS
```

### Option 3: Self-Hosted

#### Nginx Configuration

```nginx
# /etc/nginx/sites-available/vecdb-app

server {
    listen 80;
    server_name your-domain.com;

    root /var/www/vecdb-app;
    index index.html;

    # Gzip compression
    gzip on;
    gzip_types text/plain text/css application/json application/javascript application/wasm;
    gzip_min_length 1000;

    # WASM files
    location ~* \.wasm$ {
        types {
            application/wasm wasm;
        }
        add_header Cache-Control "public, max-age=31536000, immutable";
        add_header Cross-Origin-Embedder-Policy "require-corp";
        add_header Cross-Origin-Opener-Policy "same-origin";
    }

    # JavaScript files
    location ~* \.js$ {
        add_header Cache-Control "public, max-age=31536000, immutable";
    }

    # HTML files (no cache)
    location / {
        try_files $uri $uri/ /index.html;
        add_header Cache-Control "no-cache, no-store, must-revalidate";
    }
}
```

#### Apache Configuration

```apache
# .htaccess

# WASM MIME type
AddType application/wasm .wasm

# Compression
<IfModule mod_deflate.c>
    AddOutputFilterByType DEFLATE text/html text/plain text/css application/javascript application/wasm
</IfModule>

# Caching
<IfModule mod_expires.c>
    ExpiresActive On
    ExpiresByType application/wasm "access plus 1 year"
    ExpiresByType application/javascript "access plus 1 year"
    ExpiresByType text/html "access plus 0 seconds"
</IfModule>

# Security headers
<IfModule mod_headers.c>
    Header set Cross-Origin-Embedder-Policy "require-corp"
    Header set Cross-Origin-Opener-Policy "same-origin"
</IfModule>
```

---

## CDN Integration

### Load from CDN

```html
<!DOCTYPE html>
<html>
<head>
    <title>VecDB-WASM App</title>
</head>
<body>
    <script type="module">
        // Load from CDN
        import init, { VectorDB, Metric, IndexType }
            from 'https://cdn.example.com/vecdb-wasm@0.2.0/vecdb_wasm.js';

        async function main() {
            await init();
            const db = new VectorDB(128, Metric.Cosine, IndexType.HNSW);
            // ...
        }

        main();
    </script>
</body>
</html>
```

### Fallback Strategy

```javascript
async function loadVecDB() {
    const sources = [
        'https://cdn1.example.com/vecdb-wasm/vecdb_wasm.js',
        'https://cdn2.example.com/vecdb-wasm/vecdb_wasm.js',
        '/local/vecdb_wasm.js'  // Fallback to local
    ];

    for (const src of sources) {
        try {
            const module = await import(src);
            await module.default();  // init()
            return module;
        } catch (err) {
            console.warn(`Failed to load from ${src}:`, err);
        }
    }

    throw new Error('Failed to load VecDB-WASM from all sources');
}

// Usage
const { VectorDB, Metric, IndexType } = await loadVecDB();
```

---

## Performance Optimization

### 1. Enable WASM SIMD

**Chrome/Edge:**
```
chrome://flags/#enable-webassembly-simd
```

**Firefox:**
```
about:config
javascript.options.wasm_simd = true
```

**Verify SIMD Support:**
```javascript
import { BrowserCompat } from './pkg/vecdb_wasm.js';

const compat = new BrowserCompat();
if (compat.has_simd) {
    console.log('✅ SIMD enabled - 2-4x performance boost!');
} else {
    console.log('⚠️ SIMD not available - using scalar fallback');
}
```

### 2. Lazy Loading

```javascript
// Load WASM only when needed
let vecdbModule = null;

async function getVecDB() {
    if (!vecdbModule) {
        vecdbModule = await import('./pkg/vecdb_wasm.js');
        await vecdbModule.default();
    }
    return vecdbModule;
}

// Usage
document.getElementById('search-btn').addEventListener('click', async () => {
    const { VectorDB } = await getVecDB();
    // Use VectorDB...
});
```

### 3. Preload WASM

```html
<head>
    <!-- Preload WASM file -->
    <link rel="preload" href="/pkg/vecdb_wasm_bg.wasm" as="fetch" crossorigin>
    <link rel="modulepreload" href="/pkg/vecdb_wasm.js">
</head>
```

### 4. Use Service Worker for Caching

```javascript
// service-worker.js
const CACHE_NAME = 'vecdb-wasm-v1';
const WASM_FILES = [
    '/pkg/vecdb_wasm.js',
    '/pkg/vecdb_wasm_bg.wasm'
];

self.addEventListener('install', (event) => {
    event.waitUntil(
        caches.open(CACHE_NAME).then((cache) => {
            return cache.addAll(WASM_FILES);
        })
    );
});

self.addEventListener('fetch', (event) => {
    event.respondWith(
        caches.match(event.request).then((response) => {
            return response || fetch(event.request);
        })
    );
});
```

```javascript
// Register service worker in main app
if ('serviceWorker' in navigator) {
    navigator.serviceWorker.register('/service-worker.js');
}
```

### 5. Database Initialization

```javascript
// Initialize database once and reuse
let globalDB = null;

async function initDatabase() {
    if (!globalDB) {
        // Try to load from IndexedDB
        try {
            const data = await VectorDB.load_from_indexeddb('main_db');
            globalDB = VectorDB.import_snapshot(data);
            console.log('✅ Loaded database from IndexedDB');
        } catch (err) {
            // Create new database
            globalDB = new VectorDB(128, Metric.Cosine, IndexType.HNSW);
            console.log('✅ Created new database');
        }
    }
    return globalDB;
}

// Usage
const db = await initDatabase();
```

---

## Security Considerations

### 1. Content Security Policy (CSP)

```html
<meta http-equiv="Content-Security-Policy"
      content="default-src 'self';
               script-src 'self' 'wasm-unsafe-eval';
               worker-src 'self' blob:;">
```

### 2. Subresource Integrity (SRI)

```bash
# Generate SRI hash
openssl dgst -sha384 -binary pkg/vecdb_wasm.js | openssl base64 -A

# Use in HTML
<script type="module"
        src="/pkg/vecdb_wasm.js"
        integrity="sha384-HASH_HERE"
        crossorigin="anonymous"></script>
```

### 3. CORS Headers

```javascript
// Server configuration
res.setHeader('Cross-Origin-Embedder-Policy', 'require-corp');
res.setHeader('Cross-Origin-Opener-Policy', 'same-origin');
```

### 4. Input Validation

```javascript
function validateVector(vector, expectedDim) {
    if (!Array.isArray(vector) && !(vector instanceof Float32Array)) {
        throw new Error('Vector must be an array or Float32Array');
    }

    if (vector.length !== expectedDim) {
        throw new Error(`Expected ${expectedDim}D vector, got ${vector.length}D`);
    }

    if (vector.some(v => !isFinite(v))) {
        throw new Error('Vector contains invalid values (NaN or Infinity)');
    }

    return true;
}

// Usage
try {
    validateVector(userVector, 128);
    db.insert(id, userVector);
} catch (err) {
    console.error('Invalid vector:', err);
}
```

---

## Monitoring

### 1. Performance Monitoring

```javascript
class VecDBMonitor {
    constructor(db) {
        this.db = db;
        this.metrics = {
            searches: 0,
            inserts: 0,
            totalSearchTime: 0,
            totalInsertTime: 0
        };
    }

    async search(query, k) {
        const start = performance.now();
        const results = this.db.search(query, k);
        const elapsed = performance.now() - start;

        this.metrics.searches++;
        this.metrics.totalSearchTime += elapsed;

        // Log slow searches
        if (elapsed > 100) {
            console.warn(`Slow search: ${elapsed.toFixed(2)}ms`);
        }

        return results;
    }

    getMetrics() {
        return {
            ...this.metrics,
            avgSearchTime: this.metrics.totalSearchTime / this.metrics.searches,
            qps: 1000 / (this.metrics.totalSearchTime / this.metrics.searches)
        };
    }
}

// Usage
const monitor = new VecDBMonitor(db);
const results = await monitor.search(query, 10);
console.log('Metrics:', monitor.getMetrics());
```

### 2. Error Tracking

```javascript
// Integrate with Sentry
import * as Sentry from '@sentry/browser';

Sentry.init({
    dsn: 'YOUR_SENTRY_DSN',
    integrations: [new Sentry.BrowserTracing()],
});

try {
    db.insert(id, vector);
} catch (error) {
    Sentry.captureException(error, {
        tags: {
            component: 'vecdb',
            operation: 'insert'
        },
        extra: {
            vectorDim: vector.length,
            dbSize: db.len()
        }
    });
}
```

### 3. Analytics

```javascript
// Track usage with Google Analytics
gtag('event', 'vecdb_search', {
    'event_category': 'database',
    'event_label': 'vector_search',
    'value': k,
    'dimension': db.dimension()
});
```

---

## Troubleshooting

### Common Issues

#### Issue 1: WASM Loading Fails

**Error:** `Failed to fetch WASM module`

**Solutions:**
```javascript
// Check MIME type
fetch('/pkg/vecdb_wasm_bg.wasm')
    .then(r => console.log('Content-Type:', r.headers.get('content-type')));
// Should be: application/wasm

// Verify CORS headers
// Add to server configuration:
// Access-Control-Allow-Origin: *
```

#### Issue 2: IndexedDB Not Available

**Error:** `IndexedDB not available in this browser`

**Solutions:**
```javascript
// Check if in private/incognito mode
import { BrowserCompat } from './pkg/vecdb_wasm.js';

const compat = new BrowserCompat();
if (!compat.has_indexeddb) {
    console.warn('IndexedDB not available - using in-memory storage');
    // Fallback to localStorage or memory
}
```

#### Issue 3: Out of Memory

**Error:** `Out of memory`

**Solutions:**
```javascript
// Monitor memory usage
const stats = db.get_stats();
console.log('Memory usage:', stats.memory_bytes / 1024 / 1024, 'MB');

// Use smaller batches
const BATCH_SIZE = 1000;
for (let i = 0; i < vectors.length; i += BATCH_SIZE) {
    const batch = vectors.slice(i, i + BATCH_SIZE);
    db.batch_insert(batch);
}
```

#### Issue 4: Slow Performance

**Solutions:**
```javascript
// 1. Check SIMD support
const compat = new BrowserCompat();
console.log('SIMD:', compat.has_simd);

// 2. Use HNSW for large datasets
if (db.len() > 10000) {
    // Rebuild with HNSW
    const newDb = VectorDB.new_with_hnsw_params(dim, metric, 16, 200);
    // Migrate data...
}

// 3. Optimize HNSW parameters
// Lower M and ef_construction for faster build
const fastDb = VectorDB.new_with_hnsw_params(dim, metric, 8, 100);
```

---

## Production Checklist

- [ ] Build with `--release` flag
- [ ] Enable gzip/brotli compression
- [ ] Set proper cache headers
- [ ] Configure CORS if needed
- [ ] Add WASM MIME type
- [ ] Implement error handling
- [ ] Add performance monitoring
- [ ] Test on target browsers
- [ ] Verify SIMD support
- [ ] Setup fallback for unsupported browsers
- [ ] Implement data backup strategy
- [ ] Add loading states
- [ ] Test with production data volume
- [ ] Configure CDN if using
- [ ] Setup analytics
- [ ] Document deployment process

---

## Example Production Setup

Complete example with all best practices:

```javascript
import * as Sentry from '@sentry/browser';

// Initialize Sentry
Sentry.init({ dsn: 'YOUR_DSN' });

// Compatibility check
import { BrowserCompat } from './pkg/vecdb_wasm.js';
const compat = new BrowserCompat();

if (!compat.is_fully_compatible()) {
    const warnings = compat.get_warnings();
    console.warn('Compatibility issues:', warnings);
    // Show user warning
}

// Initialize with fallback
let db = null;

async function initDB() {
    try {
        // Try to load from IndexedDB
        const data = await VectorDB.load_from_indexeddb('production_db');
        db = VectorDB.import_snapshot(data);
        console.log('✅ Loaded from IndexedDB');
    } catch (err) {
        // Create new database
        db = new VectorDB(128, Metric.Cosine, IndexType.HNSW);
        console.log('✅ Created new database');
    }

    // Auto-save every 5 minutes
    setInterval(async () => {
        try {
            await db.save_to_indexeddb('production_db');
            console.log('✅ Auto-saved');
        } catch (err) {
            Sentry.captureException(err);
        }
    }, 5 * 60 * 1000);
}

// Search with monitoring
async function search(query, k = 10) {
    const start = performance.now();

    try {
        const results = db.search(query, k, true);
        const elapsed = performance.now() - start;

        // Analytics
        gtag('event', 'search', {
            event_category: 'vecdb',
            event_label: 'success',
            value: elapsed
        });

        return results;
    } catch (err) {
        Sentry.captureException(err);
        throw err;
    }
}

// Initialize on load
initDB().catch(err => {
    Sentry.captureException(err);
    console.error('Failed to initialize database:', err);
});
```

---

For more information, see:
- [API Documentation](API.md)
- [Quick Start Guide](QUICKSTART.md)
- [Browser Compatibility Guide](../BROWSER_COMPATIBILITY.md)
