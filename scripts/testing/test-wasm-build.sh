#!/bin/bash
echo "=== Testing WASM Build Process ==="

# Check if wasm-pack is available
if ! command -v wasm-pack &> /dev/null; then
    echo "Installing wasm-pack..."
    curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
fi

# Check for Rust
if ! command -v rustc &> /dev/null; then
    echo "Rust not found. Installing..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source $HOME/.cargo/env
fi

# Add wasm target
rustup target add wasm32-unknown-unknown 2>/dev/null || true

echo "Build tools ready."
