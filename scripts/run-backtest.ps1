# Run Backtest Script
# This script runs a backtest and displays the results

Write-Host "=== Running Backtest ===" -ForegroundColor Green
Write-Host ""

# Check if backtest service is running
try {
    $response = Invoke-WebRequest -Uri http://localhost:3004/ -Method GET -TimeoutSec 5
    if ($response.StatusCode -eq 200) {
        Write-Host "OK Backtest service is running" -ForegroundColor Green
    }
} catch {
    Write-Host "Error Backtest service is not running. Starting it now..." -ForegroundColor Red
    Start-Process -FilePath "cargo" -ArgumentList "run", "--bin", "svc-backtest" -WindowStyle Hidden
    Start-Sleep -Seconds 10
    Write-Host "OK Backtest service started" -ForegroundColor Green
}

Write-Host ""
Write-Host "Running backtest with test signals..." -ForegroundColor Cyan

# Run the backtest
try {
    $result = Invoke-WebRequest -Uri http://localhost:3004/backtest/run -Method POST -ContentType "application/json" -InFile "test_signals.json"
    
    # Save results to file
    $result.Content | Out-File -FilePath "backtest_results.json" -Encoding UTF8
    
    Write-Host "OK Backtest completed successfully" -ForegroundColor Green
    Write-Host ""
    
    # Parse and display key results
    $json = $result.Content | ConvertFrom-Json
    
    Write-Host "=== Backtest Results ===" -ForegroundColor Yellow
    Write-Host "Total Trades: $($json.total_trades)" -ForegroundColor White
    Write-Host "Winning Trades: $($json.winning_trades)" -ForegroundColor White
    Write-Host "Losing Trades: $($json.losing_trades)" -ForegroundColor White
    Write-Host "Win Rate: $(($json.win_rate * 100))%" -ForegroundColor White
    Write-Host "Total Profit/Loss: $"$($json.total_profit_loss) -ForegroundColor White
    Write-Host "Average Profit Per Trade: $"$($json.avg_profit_per_trade) -ForegroundColor White
    
    if ($json.total_profit_loss -gt 0) {
        Write-Host "Result: PROFITABLE" -ForegroundColor Green
    } else {
        Write-Host "Result: NOT PROFITABLE" -ForegroundColor Red
    }
    
    Write-Host ""
    Write-Host "Full results saved to backtest_results.json" -ForegroundColor Cyan
    Write-Host "Summary saved to BACKTEST_RESULTS_SUMMARY.MD" -ForegroundColor Cyan
    
} catch {
    Write-Host "Error running backtest: $($_.Exception.Message)" -ForegroundColor Red
}