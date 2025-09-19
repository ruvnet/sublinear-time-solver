#!/bin/bash

# Build WASM module with optimization flags

echo "🚀 Building optimized WASM module..."

# Install wasm-pack if not already installed
if ! command -v wasm-pack &> /dev/null; then
    echo "Installing wasm-pack..."
    curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
fi

# Clean previous builds
rm -rf pkg/

# Create WASM-specific Cargo.toml
cat > Cargo-wasm.toml << 'EOF'
[package]
name = "sublinear-wasm"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]
path = "src/lib_simple.rs"

[dependencies]
wasm-bindgen = "0.2"

[profile.release]
opt-level = 3
lto = true
codegen-units = 1
panic = "abort"

[target.'cfg(target_arch = "wasm32")'.dependencies]
console_error_panic_hook = { version = "0.1", optional = true }

[features]
default = ["console_error_panic_hook"]
EOF

# Build with wasm-pack
wasm-pack build --target nodejs --out-dir pkg --no-typescript -- --manifest-path Cargo-wasm.toml

# Optimize WASM with wasm-opt if available
if command -v wasm-opt &> /dev/null; then
    echo "Optimizing WASM with wasm-opt..."
    wasm-opt -O3 --enable-simd pkg/sublinear_wasm_bg.wasm -o pkg/sublinear_wasm_bg_optimized.wasm
    mv pkg/sublinear_wasm_bg_optimized.wasm pkg/sublinear_wasm_bg.wasm
fi

echo "✅ WASM module built successfully!"
echo "📦 Output in pkg/ directory"