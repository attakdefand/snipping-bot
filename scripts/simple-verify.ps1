# Simple verification script for 66+ testing implementation

Write-Host "Verifying 66+ Testing Implementation..." -ForegroundColor Green
Write-Host "======================================" -ForegroundColor Green

# Check for required files
Write-Host "Checking required components..." -ForegroundColor Yellow

$RequiredFiles = @(
    ".github/workflows/testing-66plus.yml",
    "docs/testing/66plus-testing-implementation.md",
    "scripts/generate-test-files.py",
    "scripts/setup-testing-framework.ps1"
)

$AllPresent = $true
foreach ($file in $RequiredFiles) {
    if (Test-Path $file) {
        Write-Host "  [✓] $file" -ForegroundColor Green
    } else {
        Write-Host "  [ ] $file" -ForegroundColor Red
        $AllPresent = $false
    }
}

# Check for test directories
Write-Host ""
Write-Host "Checking test directories..." -ForegroundColor Yellow

$TestDirs = @(
    "tests/happy_path",
    "tests/boundary",
    "tests/equivalence",
    "tests/state",
    "tests/api_contract",
    "tests/auth",
    "tests/secrets",
    "tests/smoke",
    "tests/regression"
)

foreach ($dir in $TestDirs) {
    if (Test-Path $dir) {
        Write-Host "  [✓] $dir" -ForegroundColor Green
    } else {
        Write-Host "  [ ] $dir" -ForegroundColor Red
    }
}

Write-Host ""
if ($AllPresent) {
    Write-Host "✅ Core components are in place!" -ForegroundColor Green
    Write-Host "The 66+ testing framework has been successfully set up." -ForegroundColor Green
} else {
    Write-Host "❌ Some components are missing." -ForegroundColor Red
}

Write-Host ""
Write-Host "Next steps:" -ForegroundColor Yellow
Write-Host "1. Run tests with: cargo test" -ForegroundColor Yellow
Write-Host "2. Check CI/CD execution: .github/workflows/testing-66plus.yml" -ForegroundColor Yellow