# Build script for the WASM dashboard on Windows
# This script compiles the Rust code to WebAssembly and prepares it for web deployment

Write-Host "Building WASM dashboard..."

# Check if wasm-pack is installed
if (!(Get-Command wasm-pack -ErrorAction SilentlyContinue)) {
    Write-Host "Installing wasm-pack..."
    curl https://rustwasm.github.io/wasm-pack/installer/init.ps1 -UseBasicParsing -o init.ps1
    .\init.ps1
    Remove-Item init.ps1
}

# Build the WASM package
wasm-pack build --target web --out-dir pkg

Write-Host "WASM dashboard built successfully!"
Write-Host "Output files are in the pkg/ directory"