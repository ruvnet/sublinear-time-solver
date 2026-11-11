#!/usr/bin/env bash
# Build the temporal-neural-solver npm package from this crate.
# Needs: rustup target wasm32-unknown-unknown, and wasm-bindgen-cli at the
# exact version pinned in Cargo.lock (the CLI and the crate must match).
set -euo pipefail
cd "$(dirname "$0")"
want=$(awk '/^name = "wasm-bindgen"$/{getline; gsub(/[^0-9.]/,""); print}' Cargo.lock)
have=$(wasm-bindgen --version | awk '{print $2}')
if [ "$want" != "$have" ]; then
  echo "wasm-bindgen-cli $have found, Cargo.lock pins $want: cargo install wasm-bindgen-cli --version $want --locked" >&2
  exit 1
fi
cargo build --release --locked --target wasm32-unknown-unknown
target_dir=${CARGO_TARGET_DIR:-target}
wasm-bindgen --target nodejs --out-dir npm --out-name temporal-neural-solver \
  "$target_dir/wasm32-unknown-unknown/release/temporal_neural_solver_wasm.wasm"
echo "built npm/temporal-neural-solver{.js,.d.ts,_bg.wasm,_bg.wasm.d.ts}"
