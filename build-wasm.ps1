# Build script for all WASM components in the project
# This script builds the WASM dashboard and prepares it for deployment

Write-Host "Building all WASM components..."

# Navigate to the WASM dashboard directory
Set-Location -Path "crates\sniper-wasm-dashboard"

# Check if wasm-pack is installed
if (!(Get-Command wasm-pack -ErrorAction SilentlyContinue)) {
    Write-Host "Installing wasm-pack..."
    curl https://rustwasm.github.io/wasm-pack/installer/init.ps1 -UseBasicParsing -o init.ps1
    .\init.ps1
    Remove-Item init.ps1
}

# Build the WASM package
Write-Host "Building WASM dashboard..."
wasm-pack build --target web --out-dir pkg

Write-Host "WASM components built successfully!"

# Return to project root
Set-Location -Path "..\.."

Write-Host "To run the WASM dashboard service:"
Write-Host "  cargo run --bin svc-wasm-dashboard"
Write-Host ""
Write-Host "Or to run all services:"
Write-Host "  .\scripts\run-all-services.ps1"