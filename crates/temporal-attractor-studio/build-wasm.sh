#!/bin/bash

# Build script for WASM package

echo "🚀 Building Temporal Attractor Studio WASM package..."

# Install wasm-pack if not present
if ! command -v wasm-pack &> /dev/null; then
    echo "📦 Installing wasm-pack..."
    curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
fi

# Clean previous builds
echo "🧹 Cleaning previous builds..."
rm -rf pkg pkg-node pkg-bundler

# Build for web (browser)
echo "🌐 Building for web..."
wasm-pack build --target web --out-dir pkg --features wasm

# Build for Node.js
echo "📦 Building for Node.js..."
wasm-pack build --target nodejs --out-dir pkg-node --features wasm

# Build for bundlers (webpack, rollup, etc.)
echo "📦 Building for bundlers..."
wasm-pack build --target bundler --out-dir pkg-bundler --features wasm

echo "✅ Build complete!"
echo ""
echo "📂 Output directories:"
echo "  - pkg/        : Web browser package"
echo "  - pkg-node/   : Node.js package"
echo "  - pkg-bundler/: Bundler package (webpack, rollup)"
echo ""
echo "🎯 Usage:"
echo "  - Browser: import from './pkg/temporal_attractor_studio.js'"
echo "  - Node.js: const tas = require('./pkg-node/temporal_attractor_studio.js')"
echo "  - Bundler: import * as tas from './pkg-bundler'"