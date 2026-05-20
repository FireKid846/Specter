#!/bin/bash
# Build Specter Engine to WebAssembly
# Requires: wasm-pack (curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh)

set -e

echo "Building Specter Engine → WASM..."

# Build with wasm feature enabled
wasm-pack build packages/engine \
  --target web \
  --out-dir ../../packages/npm/wasm \
  --release \
  -- --features wasm

echo "✓ WASM build complete → packages/npm/wasm/"
echo ""
echo "Files generated:"
ls packages/npm/wasm/
