#!/bin/bash

set -e

echo "🦀 Building VecDB-WASM..."

# Check if wasm-pack is installed
if ! command -v wasm-pack &> /dev/null; then
    echo "❌ wasm-pack not found. Installing..."
    cargo install wasm-pack
fi

# Build release version
echo "📦 Building release version..."
wasm-pack build --target web --release

echo "✅ Build complete! Package available in pkg/"
echo ""
echo "To test:"
echo "  cd examples"
echo "  python3 -m http.server 8080"
echo "  Open http://localhost:8080"
