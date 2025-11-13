# Build script for the WASM dashboard and copy files to the service directory
# This script compiles the Rust code to WebAssembly and copies the output to the svc-wasm-dashboard assets

Write-Host "Building WASM dashboard and copying files..." -ForegroundColor Green

# Navigate to the WASM dashboard directory
Set-Location "$PSScriptRoot\crates\sniper-wasm-dashboard"

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

# Copy the built files to the svc-wasm-dashboard assets directory
Write-Host "Copying files to svc-wasm-dashboard assets directory..."
$sourceDir = "$PSScriptRoot\crates\sniper-wasm-dashboard\pkg"
$destDir = "$PSScriptRoot\crates\svc-wasm-dashboard\assets\static"

# Create destination directory if it doesn't exist
if (!(Test-Path $destDir)) {
    New-Item -ItemType Directory -Path $destDir -Force | Out-Null
}

# Copy all files from pkg to assets/static
Copy-Item -Path "$sourceDir\*" -Destination $destDir -Force

Write-Host "WASM dashboard built and files copied successfully!" -ForegroundColor Green
Write-Host "Files are now available in: $destDir"