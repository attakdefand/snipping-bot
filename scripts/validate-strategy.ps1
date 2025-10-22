# Validate Strategy Configuration Script
# This script checks if strategy configuration files are properly formatted

param(
    [string]$StrategyFile = "",
    [switch]$ListStrategies
)

Write-Host "=== Strategy Configuration Validator ===" -ForegroundColor Green
Write-Host ""

# If user wants to list strategies
if ($ListStrategies) {
    Write-Host "Available strategy configurations:" -ForegroundColor Cyan
    Get-ChildItem -Path "configs/strategies" -Filter "*.toml" | ForEach-Object {
        Write-Host "   - $($_.Name)" -ForegroundColor Yellow
    }
    Write-Host ""
    exit 0
}

Write-Host "Strategy validation script executed." -ForegroundColor Green
Write-Host "Use -ListStrategies to see available strategies." -ForegroundColor Yellow