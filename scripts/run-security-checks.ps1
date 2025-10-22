# PowerShell script to run security checks locally
# This script performs the same security checks that run in the CI pipeline

param(
    [Parameter(Mandatory=$false)]
    [ValidateSet("quick", "full")]
    [string]$Level = "quick"
)

Write-Host "Running Security Checks for Snipping Bot" -ForegroundColor Green
Write-Host "=====================================" -ForegroundColor Green
Write-Host "Security Level: $Level" -ForegroundColor Yellow
Write-Host ""

# Function to check if a command exists
function Test-Command {
    param([string]$Command)
    return [bool](Get-Command -Name $Command -ErrorAction SilentlyContinue)
}

# Function to install a Rust tool if not present
function Install-RustTool {
    param([string]$Tool, [string]$Crate)
    
    if (-not (Test-Command $Tool)) {
        Write-Host "Installing $Tool..." -ForegroundColor Yellow
        cargo install $Crate --locked
        if ($LASTEXITCODE -ne 0) {
            Write-Host "❌ Failed to install $Tool" -ForegroundColor Red
            exit 1
        }
    }
}

# Check 1: Code formatting and linting
Write-Host "1. Checking code formatting and linting..." -ForegroundColor Yellow
if (Test-Command "rustup") {
    rustup component add rustfmt clippy 2>$null
    cargo fmt --all -- --check
    if ($LASTEXITCODE -ne 0) {
        Write-Host "❌ Code formatting check failed" -ForegroundColor Red
        exit 1
    }
    
    cargo clippy --all-targets --all-features -- -D warnings
    if ($LASTEXITCODE -ne 0) {
        Write-Host "❌ Clippy linting check failed" -ForegroundColor Red
        exit 1
    }
    Write-Host "✅ Code formatting and linting checks passed" -ForegroundColor Green
} else {
    Write-Host "⚠️  Rust not found, skipping formatting and linting checks" -ForegroundColor Yellow
}

Write-Host ""

# Check 2: Security audit
Write-Host "2. Running security audit..." -ForegroundColor Yellow
Install-RustTool "cargo-audit" "cargo-audit"
cargo audit --deny warnings
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Security audit failed" -ForegroundColor Red
    exit 1
} else {
    Write-Host "✅ Security audit passed" -ForegroundColor Green
}

Write-Host ""

# Check 3: License and dependency checks
Write-Host "3. Running license and dependency checks..." -ForegroundColor Yellow
Install-RustTool "cargo-deny" "cargo-deny"
cargo deny check bans sources licenses
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ License and dependency checks failed" -ForegroundColor Red
    exit 1
} else {
    Write-Host "✅ License and dependency checks passed" -ForegroundColor Green
}

Write-Host ""

# Full security checks (only run if level is "full")
if ($Level -eq "full") {
    # Check 4: Unused dependencies
    Write-Host "4. Checking for unused dependencies..." -ForegroundColor Yellow
    Install-RustTool "cargo-udeps" "cargo-udeps"
    cargo udeps --all-targets
    if ($LASTEXITCODE -ne 0) {
        Write-Host "❌ Unused dependencies check failed" -ForegroundColor Red
        exit 1
    } else {
        Write-Host "✅ No unused dependencies found" -ForegroundColor Green
    }
    
    Write-Host ""
    
    # Check 5: Unused Cargo.toml entries
    Write-Host "5. Checking for unused Cargo.toml entries..." -ForegroundColor Yellow
    Install-RustTool "cargo-machete" "cargo-machete"
    cargo machete
    if ($LASTEXITCODE -ne 0) {
        Write-Host "❌ Unused Cargo.toml entries check failed" -ForegroundColor Red
        exit 1
    } else {
        Write-Host "✅ No unused Cargo.toml entries found" -ForegroundColor Green
    }
    
    Write-Host ""
    
    # Check 6: Secrets scanning (if gitleaks is available)
    Write-Host "6. Scanning for secrets..." -ForegroundColor Yellow
    if (Test-Command "gitleaks") {
        gitleaks detect --source=. --verbose
        if ($LASTEXITCODE -eq 0) {
            Write-Host "✅ No secrets found" -ForegroundColor Green
        } else {
            Write-Host "❌ Secrets detected!" -ForegroundColor Red
            exit 1
        }
    } else {
        Write-Host "⚠️  gitleaks not found, skipping secrets scan" -ForegroundColor Yellow
        Write-Host "Install gitleaks with: brew install gitleaks (macOS) or download from https://github.com/gitleaks/gitleaks" -ForegroundColor Cyan
    }
    
    Write-Host ""
}

# Summary
Write-Host "🎉 All security checks passed!" -ForegroundColor Green
if ($Level -eq "quick") {
    Write-Host "Note: Run with '-Level full' for more comprehensive security checks" -ForegroundColor Cyan
}
Write-Host ""
Write-Host "For continuous security monitoring, consider setting up:" -ForegroundColor Cyan
Write-Host "- Automated dependency updates (Dependabot)" -ForegroundColor Cyan
Write-Host "- Container image scanning" -ForegroundColor Cyan
Write-Host "- Infrastructure as Code (IaC) scanning" -ForegroundColor Cyan
Write-Host "- Dynamic Application Security Testing (DAST)" -ForegroundColor Cyan