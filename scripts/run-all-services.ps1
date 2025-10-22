# Run all snipping bot services simultaneously
# This script starts all services in the background

Write-Host "Starting all snipping bot services..." -ForegroundColor Green

# Start services in background jobs
Start-Job -ScriptBlock { Set-Location "D:\auto-sync-daily-from-winC\snipping-bot"; cargo run --bin svc-analytics } -Name "svc-analytics"
Start-Job -ScriptBlock { Set-Location "D:\auto-sync-daily-from-winC\snipping-bot"; cargo run --bin svc-backtest } -Name "svc-backtest"
Start-Job -ScriptBlock { Set-Location "D:\auto-sync-daily-from-winC\snipping-bot"; cargo run --bin svc-dashboard } -Name "svc-dashboard"
Start-Job -ScriptBlock { Set-Location "D:\auto-sync-daily-from-winC\snipping-bot"; cargo run --bin svc-executor } -Name "svc-executor"
Start-Job -ScriptBlock { Set-Location "D:\auto-sync-daily-from-winC\snipping-bot"; cargo run --bin svc-gateway } -Name "svc-gateway"
Start-Job -ScriptBlock { Set-Location "D:\auto-sync-daily-from-winC\snipping-bot"; cargo run --bin svc-nft } -Name "svc-nft"
Start-Job -ScriptBlock { Set-Location "D:\auto-sync-daily-from-winC\snipping-bot"; cargo run --bin svc-policy } -Name "svc-policy"
Start-Job -ScriptBlock { Set-Location "D:\auto-sync-daily-from-winC\snipping-bot"; cargo run --bin svc-risk } -Name "svc-risk"
Start-Job -ScriptBlock { Set-Location "D:\auto-sync-daily-from-winC\snipping-bot"; cargo run --bin svc-signals } -Name "svc-signals"
Start-Job -ScriptBlock { Set-Location "D:\auto-sync-daily-from-winC\snipping-bot"; cargo run --bin svc-storage } -Name "svc-storage"
Start-Job -ScriptBlock { Set-Location "D:\auto-sync-daily-from-winC\snipping-bot"; cargo run --bin svc-strategy } -Name "svc-strategy"
Start-Job -ScriptBlock { Set-Location "D:\auto-sync-daily-from-winC\snipping-bot"; cargo run --bin sniper-bench } -Name "sniper-bench"

Write-Host "All services started in background jobs." -ForegroundColor Green
Write-Host "Use 'Get-Job' to check status and 'Receive-Job -Name <job-name>' to see output." -ForegroundColor Yellow

# Show running jobs
Get-Job