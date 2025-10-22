# Performance Monitoring Script
# This script monitors bot performance and logs key metrics

Write-Host "=== Performance Monitoring ===" -ForegroundColor Green
Write-Host ""

# Check if services are running
$services = @("http://localhost:3003/", "http://localhost:3004/", "http://localhost:3005/")
$serviceNames = @("Analytics", "Backtest", "Dashboard")

for ($i = 0; $i -lt $services.Length; $i++) {
    try {
        $response = Invoke-WebRequest -Uri $services[$i] -Method GET -TimeoutSec 5
        if ($response.StatusCode -eq 200) {
            Write-Host "OK $($serviceNames[$i]) service is running" -ForegroundColor Green
        }
    } catch {
        Write-Host "Error $($serviceNames[$i]) service is not accessible" -ForegroundColor Red
    }
}

Write-Host ""
Write-Host "Fetching performance metrics..." -ForegroundColor Cyan

# Get performance data
try {
    # Get analytics data
    $analytics = Invoke-WebRequest -Uri "http://localhost:3003/analytics/performance" -Method GET
    $analyticsJson = $analytics.Content | ConvertFrom-Json
    
    Write-Host "=== Current Performance Metrics ===" -ForegroundColor Yellow
    if ($analyticsJson.PSObject.Properties.Count -gt 0) {
        foreach ($metric in $analyticsJson.PSObject.Properties) {
            Write-Host "$($metric.Name): $($metric.Value)" -ForegroundColor White
        }
    } else {
        Write-Host "No performance data available yet" -ForegroundColor Gray
    }
    
    # Get recent trades
    $trades = Invoke-WebRequest -Uri "http://localhost:3003/analytics/trades" -Method GET
    $tradesJson = $trades.Content | ConvertFrom-Json
    
    Write-Host ""
    Write-Host "Recent Trades: $(if ($tradesJson.Count -gt 0) { $tradesJson.Count } else { 'None' })" -ForegroundColor White
    
    # Log to file
    $timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
    $logEntry = "$timestamp - Performance Check - Trades: $(if ($tradesJson.Count -gt 0) { $tradesJson.Count } else { 'None' })"
    Add-Content -Path "performance_log.txt" -Value $logEntry
    
    Write-Host ""
    Write-Host "Performance data logged to performance_log.txt" -ForegroundColor Cyan
    
} catch {
    Write-Host "Error fetching performance data: $($_.Exception.Message)" -ForegroundColor Red
}

Write-Host ""
Write-Host "=== Monitoring Complete ===" -ForegroundColor Green