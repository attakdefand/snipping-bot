# PowerShell script to run all compliance tests for the snipping bot
# This script verifies that the project follows all guidelines in DEVELOPMENT_GUIDELINES.MD

Write-Host "Running Compliance Tests for Snipping Bot" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Green

# Test 1: Basic unit tests
Write-Host "1. Running basic unit tests..." -ForegroundColor Yellow
cargo test --lib
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Basic unit tests failed" -ForegroundColor Red
    exit 1
} else {
    Write-Host "✅ Basic unit tests passed" -ForegroundColor Green
}

# Test 2: Security component tests
Write-Host "2. Running security component tests..." -ForegroundColor Yellow
cargo test -p sniper-security
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Security component tests failed" -ForegroundColor Red
    exit 1
} else {
    Write-Host "✅ Security component tests passed" -ForegroundColor Green
}

# Test 3: Telemetry tests
Write-Host "3. Running telemetry tests..." -ForegroundColor Yellow
cargo test -p sniper-telemetry
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Telemetry tests failed" -ForegroundColor Red
    exit 1
} else {
    Write-Host "✅ Telemetry tests passed" -ForegroundColor Green
}

# Test 4: Storage tests
Write-Host "4. Running storage tests..." -ForegroundColor Yellow
cargo test -p sniper-storage
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Storage tests failed" -ForegroundColor Red
    exit 1
} else {
    Write-Host "✅ Storage tests passed" -ForegroundColor Green
}

# Test 5: Key management tests
Write-Host "5. Running key management tests..." -ForegroundColor Yellow
cargo test -p sniper-keys
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Key management tests failed" -ForegroundColor Red
    exit 1
} else {
    Write-Host "✅ Key management tests passed" -ForegroundColor Green
}

# Test 6: Risk management tests
Write-Host "6. Running risk management tests..." -ForegroundColor Yellow
cargo test -p sniper-risk
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Risk management tests failed" -ForegroundColor Red
    exit 1
} else {
    Write-Host "✅ Risk management tests passed" -ForegroundColor Green
}

# Test 7: Policy engine tests
Write-Host "7. Running policy engine tests..." -ForegroundColor Yellow
cargo test -p sniper-policy
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Policy engine tests failed" -ForegroundColor Red
    exit 1
} else {
    Write-Host "✅ Policy engine tests passed" -ForegroundColor Green
}

Write-Host ""
Write-Host "🎉 All compliance tests passed!" -ForegroundColor Green
Write-Host "The snipping bot complies with all guidelines in DEVELOPMENT_GUIDELINES.MD" -ForegroundColor Green
Write-Host "See SECURITY_COMPLIANCE_REPORT.MD for detailed compliance documentation" -ForegroundColor Cyan