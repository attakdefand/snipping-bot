# Quick Start Trading Script
# This script helps you set up and start trading with the snipping bot

Write-Host "=== Snipping Bot Quick Start Trading Setup ===" -ForegroundColor Green
Write-Host ""

Write-Host "1. Checking Docker services..." -ForegroundColor Cyan
docker-compose -f infra/compose.yml ps

Write-Host ""
Write-Host "2. Checking configuration files..." -ForegroundColor Cyan
Write-Host "   Config directories:" -ForegroundColor Yellow
Get-ChildItem -Path "configs" -Directory | ForEach-Object {
    Write-Host "     - $($_.Name)" -ForegroundColor Green
}

Write-Host ""
Write-Host "3. Key configuration files:" -ForegroundColor Cyan
$configFiles = @("configs/risk.toml", "configs/routes.toml")
foreach ($file in $configFiles) {
    if (Test-Path $file) {
        Write-Host "   OK $file exists" -ForegroundColor Green
    } else {
        Write-Host "   !! $file missing (will use defaults)" -ForegroundColor Yellow
    }
}

Write-Host ""
Write-Host "4. Available trading strategies:" -ForegroundColor Cyan
$strategies = Get-ChildItem -Path "configs/strategies" -Filter "*.toml" | Measure-Object
Write-Host "   Found $($strategies.Count) strategies" -ForegroundColor Yellow

Write-Host ""
Write-Host "5. Suggested next steps:" -ForegroundColor Cyan
Write-Host ""
Write-Host "   a. Configure your wallet keys:" -ForegroundColor Yellow
Write-Host "      - Create a .env file with your WALLET_PRIVATE_KEY" -ForegroundColor Yellow
Write-Host ""
Write-Host "   b. Choose trading strategies:" -ForegroundColor Yellow
Write-Host "      - View strategies: ./scripts/configure-trading.ps1 -ConfigType strategies" -ForegroundColor Yellow
Write-Host "      - Use launch_snipe.toml for basic sniping" -ForegroundColor Yellow
Write-Host "      - Use comprehensive_launch_snipe.toml for advanced sniping" -ForegroundColor Yellow
Write-Host ""
Write-Host "   c. Adjust risk parameters:" -ForegroundColor Yellow
Write-Host "      - View options: ./scripts/configure-trading.ps1 -ConfigType risks" -ForegroundColor Yellow
Write-Host "      - Use risk_simple.toml for beginners" -ForegroundColor Yellow
Write-Host "      - Use risk_comprehensive.toml for advanced users" -ForegroundColor Yellow
Write-Host ""
Write-Host "   d. Monitor the dashboard:" -ForegroundColor Yellow
Write-Host "      - Open http://localhost:3005 in your browser" -ForegroundColor Yellow
Write-Host ""
Write-Host "For detailed instructions, see:" -ForegroundColor Cyan
Write-Host "   - TRADING_STARTUP_GUIDE.MD" -ForegroundColor Yellow
Write-Host "   - TRADING_STRATEGIES_AND_RISK_PARAMETERS.MD" -ForegroundColor Yellow