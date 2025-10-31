# Build script for the WASM dashboard on Windows
# This script compiles the Rust code to WebAssembly and prepares it for web deployment

Write-Host "Building WASM dashboard..."

# Check if wasm-pack is installed
try {
    $wasmPackVersion = wasm-pack --version
    Write-Host "Found wasm-pack: $wasmPackVersion"
} catch {
    Write-Host "Installing wasm-pack..."
    # Download and install wasm-pack
    curl -L https://github.com/rustwasm/wasm-pack/releases/download/v0.12.1/wasm-pack-init.exe -o wasm-pack-init.exe
    .\wasm-pack-init.exe -y
    Remove-Item .\wasm-pack-init.exe
}

# Build the WASM package
Write-Host "Compiling Rust to WebAssembly..."
wasm-pack build --target web --out-dir pkg

Write-Host "WASM dashboard built successfully!"
Write-Host "Output files are in the pkg/ directory"