#!/bin/bash

# Build script for nanosecond-scheduler crate
set -e

echo "🚀 Building nanosecond-scheduler..."

# Build native library
echo "📦 Building native library..."
cargo build --release

# Build with all features
echo "🔧 Building with all features..."
cargo build --release --all-features

# Run tests
echo "🧪 Running tests..."
cargo test --release

# Build WASM target
if command -v wasm-pack &> /dev/null; then
    echo "🌐 Building WASM target..."
    wasm-pack build --target web --out-dir pkg -- --features wasm
else
    echo "⚠️  wasm-pack not installed. Install with: cargo install wasm-pack"
fi

# Run benchmarks
echo "📊 Running benchmarks..."
cargo bench

echo "✅ Build complete!"