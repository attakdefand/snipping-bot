# PowerShell script to set up the 66+ testing framework
# This script generates test files and runs initial setup

Write-Host "Setting up 66+ Testing Framework..." -ForegroundColor Green
Write-Host "===================================" -ForegroundColor Green

# Check if Python is available
try {
    $pythonVersion = & python --version 2>&1
    Write-Host "Found Python: $pythonVersion" -ForegroundColor Green
} catch {
    Write-Host "Python not found. Please install Python to generate test files." -ForegroundColor Red
    exit 1
}

# Run the Python script to generate test files
Write-Host "Generating test files for all 66+ testing types..." -ForegroundColor Yellow
try {
    & python scripts/generate-test-files.py
    Write-Host "Test files generated successfully!" -ForegroundColor Green
} catch {
    Write-Host "Failed to generate test files: $_" -ForegroundColor Red
    exit 1
}

# Create a basic Cargo.toml for integration tests if it doesn't exist
$cargoTomlPath = "tests/Cargo.toml"
if (-not (Test-Path $cargoTomlPath)) {
    Write-Host "Creating Cargo.toml for integration tests..." -ForegroundColor Yellow
    $cargoTomlContent = @"
[package]
name = "snipping-bot-tests"
version = "0.1.0"
edition = "2021"
publish = false

[dependencies]
snipping-bot = { path = ".." }
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
anyhow = "1"
"@
    
    Set-Content -Path $cargoTomlPath -Value $cargoTomlContent
    Write-Host "Created Cargo.toml for integration tests" -ForegroundColor Green
}

# Create a basic test configuration
$configPath = "configs/test-config.toml"
if (-not (Test-Path $configPath)) {
    Write-Host "Creating test configuration..." -ForegroundColor Yellow
    $configDir = Split-Path $configPath -Parent
    if (-not (Test-Path $configDir)) {
        New-Item -ItemType Directory -Path $configDir | Out-Null
    }
    
    $configContent = @"
[testing]
# Test configuration for 66+ testing types

[testing.database]
url = "postgresql://localhost:5432/test_db"

[testing.redis]
url = "redis://localhost:6379"

[testing.api]
base_url = "http://localhost:8080"

[testing.security]
scan_level = "full"
vulnerability_threshold = "medium"
"@
    
    Set-Content -Path $configPath -Value $configContent
    Write-Host "Created test configuration" -ForegroundColor Green
}

Write-Host "66+ Testing Framework setup completed!" -ForegroundColor Green
Write-Host ""
Write-Host "Next steps:" -ForegroundColor Yellow
Write-Host "1. Review the generated test files in the tests/ directory" -ForegroundColor Yellow
Write-Host "2. Implement actual test logic in the placeholder files" -ForegroundColor Yellow
Write-Host "3. Run tests with: cargo test" -ForegroundColor Yellow
Write-Host "4. Run specific category tests with: cargo test --test <category>" -ForegroundColor Yellow
Write-Host "5. Check CI/CD pipeline in .github/workflows/testing-66plus.yml" -ForegroundColor Yellow