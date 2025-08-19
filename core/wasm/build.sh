#!/bin/bash

# Build script for WriteMagic WASM module
set -e

echo "Building WriteMagic WASM module..."

# Check if wasm-pack is installed
if ! command -v wasm-pack &> /dev/null; then
    echo "wasm-pack is not installed. Installing..."
    curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
fi

# Build for different targets
echo "Building for web..."
wasm-pack build --target web --out-dir pkg-web --out-name writemagic_wasm

echo "Building for Node.js..."
wasm-pack build --target nodejs --out-dir pkg-node --out-name writemagic_wasm

echo "Building for bundlers (webpack, etc.)..."
wasm-pack build --target bundler --out-dir pkg --out-name writemagic_wasm

# Optimize for production
echo "Building optimized version..."
wasm-pack build --target web --out-dir pkg-optimized --out-name writemagic_wasm --release -- --features "wee_alloc"

echo "Build completed successfully!"
echo ""
echo "Generated packages:"
echo "  - pkg/         (for bundlers like webpack, rollup)"
echo "  - pkg-web/     (for direct web usage)"
echo "  - pkg-node/    (for Node.js)"
echo "  - pkg-optimized/ (optimized for production)"
echo ""
echo "Example usage:"
echo "  Web: import init, { WriteMagicEngine } from './pkg-web/writemagic_wasm.js';"
echo "  Node: const wasm = require('./pkg-node/writemagic_wasm.js');"
echo "  Bundler: import { WriteMagicEngine } from './pkg/writemagic_wasm.js';"