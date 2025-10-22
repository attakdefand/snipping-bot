# Build script for WASM Dashboard Service
# This script builds the WASM dashboard and copies assets to the service directory

Write-Host "Building WASM Dashboard Service..."

# Build the WASM dashboard
Set-Location -Path "../sniper-wasm-dashboard"
Write-Host "Building WASM dashboard..."
wasm-pack build --target web --out-dir pkg

# Copy WASM assets to service assets directory
Write-Host "Copying WASM assets to service directory..."
Copy-Item -Path "pkg/*" -Destination "../svc-wasm-dashboard/assets/static" -Recurse -Force

# Return to service directory
Set-Location -Path "../svc-wasm-dashboard"

Write-Host "WASM Dashboard Service build complete!"
Write-Host "Assets copied to assets/static directory"