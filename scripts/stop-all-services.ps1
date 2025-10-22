# Stop all snipping bot services

Write-Host "Stopping all snipping bot services..." -ForegroundColor Green

# Get all jobs and stop them
Get-Job | Where-Object { $_.Name -like "svc-*" -or $_.Name -eq "sniper-bench" } | Stop-Job

Write-Host "All services stopped." -ForegroundColor Green