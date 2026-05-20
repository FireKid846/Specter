#!/bin/bash
# Build WASM and publish specter-engine to npm
# Requires: wasm-pack, pnpm, npm auth token

set -e

VERSION=${1:-"0.1.0"}

echo "Publishing specter-engine v${VERSION}..."

# 1. Build WASM
bash scripts/build-wasm.sh

# 2. Update version
cd packages/npm
sed -i "s/\"version\": \".*\"/\"version\": \"${VERSION}\"/" package.json

# 3. Build TypeScript wrapper
pnpm build

# 4. Publish (dist only — source excluded via .npmignore)
npm publish --access public

echo "✓ Published specter-engine@${VERSION}"
