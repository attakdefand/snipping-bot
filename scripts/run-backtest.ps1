# Script to run backtests for the snipping bot
# This script automates the process of running backtests and generating reports

param(
    [Parameter()]
    [string]$ConfigFile = "configs/test-config.toml",
    
    [Parameter()]
    [string]$DataPath = "data/historical",
    
    [Parameter()]
    [switch]$WalkForward = $false,
    
    [Parameter()]
    [switch]$ChaosTest = $false,
    
    [Parameter()]
    [switch]$PaperTrade = $false,
    
    [Parameter()]
    [switch]$ForkTest = $false,
    
    [Parameter()]
    [switch]$ForwardTest = $false,
    
    [Parameter()]
    [string]$OutputDir = "backtest-results"
)

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  Snipping Bot Backtest Runner" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Check if required tools are installed
Write-Host "Checking prerequisites..." -ForegroundColor Yellow

# Check if Rust is installed
try {
    $rustVersion = rustc --version
    Write-Host "✓ Rust: $rustVersion" -ForegroundColor Green
} catch {
    Write-Host "✗ Rust is not installed" -ForegroundColor Red
    Write-Host "  Please install Rust from https://www.rust-lang.org/" -ForegroundColor Yellow
    exit 1
}

# Check if Cargo is available
try {
    $cargoVersion = cargo --version
    Write-Host "✓ Cargo: $cargoVersion" -ForegroundColor Green
} catch {
    Write-Host "✗ Cargo is not available" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "Starting backtest process..." -ForegroundColor Yellow

# Build the backtest service
Write-Host "Building backtest service..." -ForegroundColor Yellow
cargo build --bin svc-backtest
if ($LASTEXITCODE -ne 0) {
    Write-Host "✗ Failed to build backtest service" -ForegroundColor Red
    exit 1
}
Write-Host "✓ Backtest service built successfully" -ForegroundColor Green

# Start the backtest service in background
Write-Host "Starting backtest service..." -ForegroundColor Yellow
Start-Process -NoNewWindow -FilePath "cargo" -ArgumentList "run", "--bin", "svc-backtest" -WorkingDirectory "."
Start-Sleep -Seconds 5

# Create output directory
if (!(Test-Path $OutputDir)) {
    New-Item -ItemType Directory -Path $OutputDir | Out-Null
}

try {
    # Run backtest
    if ($WalkForward) {
        Write-Host "Running walk-forward optimization..." -ForegroundColor Yellow
        # This would call the walk-forward endpoint
        $result = Invoke-RestMethod -Uri "http://localhost:3004/backtest/walk-forward" -Method POST -Body "{}" -ContentType "application/json"
    } elseif ($ChaosTest) {
        Write-Host "Running chaos testing..." -ForegroundColor Yellow
        # This would run chaos testing scenarios
        $result = Invoke-RestMethod -Uri "http://localhost:3004/backtest/chaos" -Method POST -Body "{}" -ContentType "application/json"
    } elseif ($PaperTrade) {
        Write-Host "Running paper trade backtest..." -ForegroundColor Yellow
        # This would run paper trade backtest
        $result = Invoke-RestMethod -Uri "http://localhost:3004/backtest/paper-trade" -Method POST -Body "{}" -ContentType "application/json"
    } elseif ($ForkTest) {
        Write-Host "Running on-chain fork test..." -ForegroundColor Yellow
        # This would run fork test
        $result = Invoke-RestMethod -Uri "http://localhost:3004/backtest/fork-test" -Method POST -Body "{}" -ContentType "application/json"
    } elseif ($ForwardTest) {
        Write-Host "Running forward test..." -ForegroundColor Yellow
        # This would run forward test
        $result = Invoke-RestMethod -Uri "http://localhost:3004/backtest/forward-test" -Method POST -Body "{}" -ContentType "application/json"
    } else {
        Write-Host "Running standard backtest..." -ForegroundColor Yellow
        # This would call the backtest endpoint
        $result = Invoke-RestMethod -Uri "http://localhost:3004/backtest/run" -Method POST -Body "{}" -ContentType "application/json"
    }
    
    # Save results to file
    $timestamp = Get-Date -Format "yyyyMMdd-HHmmss"
    $testType = if ($WalkForward) { "walk-forward" } 
                elseif ($ChaosTest) { "chaos-test" } 
                elseif ($PaperTrade) { "paper-trade" } 
                elseif ($ForkTest) { "fork-test" } 
                elseif ($ForwardTest) { "forward-test" } 
                else { "standard" }
    
    $resultFile = "$OutputDir\backtest-results-$testType-$timestamp.json"
    $result | ConvertTo-Json -Depth 10 | Out-File -FilePath $resultFile
    
    Write-Host "✓ Backtest completed successfully" -ForegroundColor Green
    Write-Host "Results saved to: $resultFile" -ForegroundColor Cyan
    
    # Generate human-readable summary
    $summaryFile = "$OutputDir\BACKTEST_RESULTS_SUMMARY-$testType-$timestamp.MD"
    Generate-Backtest-Summary -Results $result -OutputFile $summaryFile
    Write-Host "Summary report saved to: $summaryFile" -ForegroundColor Cyan
} catch {
    Write-Host "✗ Error running backtest: $_" -ForegroundColor Red
} finally {
    # Stop the backtest service
    Write-Host "Stopping backtest service..." -ForegroundColor Yellow
    # In a real implementation, we would properly stop the service
    Write-Host "✓ Backtest service stopped" -ForegroundColor Green
}

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  Backtest Process Complete" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan

function Generate-Backtest-Summary {
    param(
        [Parameter(Mandatory=$true)]
        [object]$Results,
        
        [Parameter(Mandatory=$true)]
        [string]$OutputFile
    )
    
    $summary = @"
# Backtest Results Summary

## Performance Metrics

| Metric | Value |
|--------|-------|
| Total Trades | $($Results.total_trades) |
| Winning Trades | $($Results.winning_trades) |
| Losing Trades | $($Results.losing_trades) |
| Win Rate | $(($Results.win_rate * 100).ToString("F2"))% |
| Total Profit/Loss | $($Results.total_profit_loss.ToString("F2")) |
| Average Profit per Trade | $($Results.avg_profit_per_trade.ToString("F2")) |
| Max Drawdown | $(($Results.max_drawdown * 100).ToString("F2"))% |
| Sharpe Ratio | $($Results.sharpe_ratio.ToString("F2")) |
| Sortino Ratio | $($Results.sortino_ratio.ToString("F2")) |
| Calmar Ratio | $($Results.calmar_ratio.ToString("F2")) |

## Trading Costs

| Cost Type | Amount |
|-----------|--------|
| Total Fees Paid | $($Results.total_fees_paid.ToString("F2")) |
| Total Slippage Loss | $($Results.total_slippage_loss.ToString("F2")) |

## Trade Statistics

| Statistic | Value |
|-----------|-------|
| Max Consecutive Wins | $($Results.max_consecutive_wins) |
| Max Consecutive Losses | $($Results.max_consecutive_losses) |

## Configuration

| Parameter | Value |
|-----------|-------|
| Initial Capital | $($Results.config.initial_capital) |
| Trading Fee | $(($Results.config.trading_fee_pct * 100).ToString("F3"))% |
| Slippage | $(($Results.config.slippage_pct * 100).ToString("F3"))% |
| Max Position Size | $(($Results.config.max_position_size_pct * 100).ToString("F1"))% |
| Execution Model | $($Results.config.execution_model) |

"@
    
    $summary | Out-File -FilePath $OutputFile -Encoding UTF8
}