#!/bin/bash

# Build script for the WASM dashboard
# This script compiles the Rust code to WebAssembly and prepares it for web deployment

set -e

echo "Building WASM dashboard..."

# Install wasm-pack if not already installed
if ! command -v wasm-pack &> /dev/null
then
    echo "Installing wasm-pack..."
    curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
fi

# Build the WASM package
wasm-pack build --target web --out-dir pkg

echo "WASM dashboard built successfully!"
echo "Output files are in the pkg/ directory"