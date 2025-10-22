# Check status of all snipping bot services

Write-Host "Checking status of all snipping bot services..." -ForegroundColor Green

# Get all jobs related to our services
$jobs = Get-Job | Where-Object { $_.Name -like "svc-*" -or $_.Name -eq "sniper-bench" }

if ($jobs.Count -eq 0) {
    Write-Host "No services are currently running." -ForegroundColor Yellow
} else {
    Write-Host "Running services:" -ForegroundColor Cyan
    $jobs | Format-Table -Property Name, State, HasMoreData
}

# Show output from jobs if available
foreach ($job in $jobs) {
    if ($job.HasMoreData) {
        Write-Host "`nOutput from $($job.Name):" -ForegroundColor Cyan
        Receive-Job -Id $job.Id
    }
}