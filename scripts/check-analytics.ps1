# Check Analytics Script
# This script checks the analytics service and displays key metrics

Write-Host "=== Checking Analytics Service ===" -ForegroundColor Green
Write-Host ""

# Check if analytics service is running
try {
    $response = Invoke-WebRequest -Uri http://localhost:3003/ -Method GET -TimeoutSec 5
    if ($response.StatusCode -eq 200) {
        Write-Host "OK Analytics service is running" -ForegroundColor Green
    }
} catch {
    Write-Host "Error Analytics service is not running" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "Fetching analytics data..." -ForegroundColor Cyan

# Get trade data
try {
    $trades = Invoke-WebRequest -Uri http://localhost:3003/analytics/trades -Method GET
    $tradesJson = $trades.Content | ConvertFrom-Json
    
    Write-Host "Recent Trades: $(if ($tradesJson.Count -gt 0) { $tradesJson.Count } else { 'None' })" -ForegroundColor White
    
    # Get performance metrics
    $performance = Invoke-WebRequest -Uri http://localhost:3003/analytics/performance -Method GET
    $perfJson = $performance.Content | ConvertFrom-Json
    
    Write-Host ""
    Write-Host "=== Performance Metrics ===" -ForegroundColor Yellow
    foreach ($metric in $perfJson.PSObject.Properties) {
        Write-Host "$($metric.Name): $($metric.Value)" -ForegroundColor White
    }
    
    Write-Host ""
    Write-Host "Analytics data fetched successfully" -ForegroundColor Green
    
} catch {
    Write-Host "Error fetching analytics data: $($_.Exception.Message)" -ForegroundColor Red
}