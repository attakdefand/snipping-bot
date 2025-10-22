# Trading Checklist Validation Script
# This script helps validate that all required steps in the trading checklist have been completed

param(
    [string]$ProgressFile = "configs/checklist-progress.toml",
    [switch]$ShowProgress,
    [switch]$ResetProgress
)

Write-Host "=== Snipping Bot Trading Checklist Validator ===" -ForegroundColor Green
Write-Host ""

# Function to check if a service is running
function Test-ServiceRunning {
    param([string]$ServiceName)
    
    try {
        $result = docker-compose -f infra/compose.yml ps --services --filter "status=running" | Select-String -Pattern $ServiceName
        return $result -ne $null
    } catch {
        return $false
    }
}

# Function to check if a file exists
function Test-ConfigFile {
    param([string]$FilePath)
    
    return Test-Path $FilePath
}

# Function to check if environment variables are set
function Test-EnvironmentVariables {
    $requiredVars = @("WALLET_PRIVATE_KEY")
    $missingVars = @()
    
    foreach ($var in $requiredVars) {
        if ([string]::IsNullOrEmpty([Environment]::GetEnvironmentVariable($var))) {
            $missingVars += $var
        }
    }
    
    return $missingVars
}

# Function to parse TOML file
function Get-TomlContent {
    param([string]$Path)
    
    if (-not (Test-Path $Path)) {
        return @{}
    }
    
    $content = Get-Content $Path
    $result = @{}
    $currentSection = ""
    
    foreach ($line in $content) {
        # Skip empty lines and comments
        if ([string]::IsNullOrWhiteSpace($line) -or $line.Trim().StartsWith("#")) {
            continue
        }
        
        # Check for section headers
        if ($line.Trim().StartsWith("[") -and $line.Trim().EndsWith("]")) {
            $currentSection = $line.Trim().Trim('[', ']')
            $result[$currentSection] = @{}
            continue
        }
        
        # Check for key-value pairs
        if ($line.Contains("=") -and $currentSection -ne "") {
            $parts = $line.Split("=", 2)
            $key = $parts[0].Trim()
            $value = $parts[1].Trim()
            
            # Convert string values to boolean
            if ($value -eq "true") {
                $result[$currentSection][$key] = $true
            } elseif ($value -eq "false") {
                $result[$currentSection][$key] = $false
            } else {
                $result[$currentSection][$key] = $value
            }
        }
    }
    
    return $result
}

# Function to count completed items
function Get-CompletionStats {
    param([hashtable]$ProgressData)
    
    $total = 0
    $completed = 0
    
    foreach ($section in $ProgressData.Keys) {
        if ($ProgressData[$section] -is [hashtable]) {
            foreach ($item in $ProgressData[$section].Keys) {
                $total++
                if ($ProgressData[$section][$item] -eq $true) {
                    $completed++
                }
            }
        }
    }
    
    return @{
        Total = $total
        Completed = $completed
        Percentage = if ($total -gt 0) { [Math]::Round(($completed / $total) * 100, 2) } else { 0 }
    }
}

# Initialize progress tracking
if ($ResetProgress) {
    if (Test-Path "configs/checklist-progress.template.toml") {
        Copy-Item "configs/checklist-progress.template.toml" $ProgressFile -Force
        Write-Host "Progress tracking reset to template." -ForegroundColor Yellow
    }
} elseif (-not (Test-Path $ProgressFile)) {
    if (Test-Path "configs/checklist-progress.template.toml") {
        Copy-Item "configs/checklist-progress.template.toml" $ProgressFile
        Write-Host "Created new progress tracking file from template." -ForegroundColor Yellow
    }
}

# Load progress data
$progressData = Get-TomlContent -Path $ProgressFile

Write-Host "1. Checking Docker services..." -ForegroundColor Cyan
$services = @("svc-gateway", "svc-strategy", "svc-executor", "svc-risk", "svc-storage")
$runningServices = 0

foreach ($service in $services) {
    if (Test-ServiceRunning -ServiceName $service) {
        Write-Host "   [OK] $service is running" -ForegroundColor Green
        $runningServices++
    } else {
        Write-Host "   [!!] $service is not running" -ForegroundColor Red
    }
}

Write-Host ""
Write-Host "2. Checking configuration files..." -ForegroundColor Cyan
$configFiles = @(
    "configs/risk.toml",
    "configs/routes.toml",
    "configs/app.toml"
)

$existingConfigs = 0
foreach ($file in $configFiles) {
    if (Test-ConfigFile -FilePath $file) {
        Write-Host "   [OK] $file exists" -ForegroundColor Green
        $existingConfigs++
    } else {
        Write-Host "   [!!] $file missing" -ForegroundColor Red
    }
}

$strategyFiles = Get-ChildItem -Path "configs/strategies" -Filter "*.toml" | Measure-Object
Write-Host "   Found $($strategyFiles.Count) strategy configurations" -ForegroundColor Yellow

Write-Host ""
Write-Host "3. Checking environment variables..." -ForegroundColor Cyan
$missingVars = Test-EnvironmentVariables
if ($missingVars.Count -eq 0) {
    Write-Host "   [OK] All required environment variables are set" -ForegroundColor Green
} else {
    foreach ($var in $missingVars) {
        Write-Host "   [!!] Missing environment variable: $var" -ForegroundColor Red
    }
}

Write-Host ""
Write-Host "4. Checking key directories..." -ForegroundColor Cyan
$directories = @("configs", "configs/strategies", "configs/chains")
$existingDirs = 0

foreach ($dir in $directories) {
    if (Test-Path $dir) {
        Write-Host "   [OK] $dir exists" -ForegroundColor Green
        $existingDirs++
    } else {
        Write-Host "   [!!] $dir missing" -ForegroundColor Red
    }
}

Write-Host ""
Write-Host "5. Checking dashboard accessibility..." -ForegroundColor Cyan
try {
    $dashboardResponse = Invoke-WebRequest -Uri "http://localhost:3005/health" -Method GET -TimeoutSec 5 -ErrorAction SilentlyContinue
    if ($dashboardResponse.StatusCode -eq 200) {
        Write-Host "   [OK] Dashboard is accessible" -ForegroundColor Green
    } else {
        Write-Host "   [!!] Dashboard returned status code $($dashboardResponse.StatusCode)" -ForegroundColor Red
    }
} catch {
    Write-Host "   [!!] Dashboard is not accessible" -ForegroundColor Red
}

Write-Host ""
Write-Host "=== Validation Summary ===" -ForegroundColor Cyan
Write-Host "Services running: $runningServices/$($services.Length)" -ForegroundColor $(if($runningServices -eq $services.Length) { "Green" } else { "Yellow" })
Write-Host "Configuration files: $existingConfigs/$($configFiles.Length)" -ForegroundColor $(if($existingConfigs -eq $configFiles.Length) { "Green" } else { "Yellow" })
Write-Host "Strategy configurations: $($strategyFiles.Count)" -ForegroundColor $(if($strategyFiles.Count -gt 0) { "Green" } else { "Red" })
Write-Host "Environment variables: $(if($missingVars.Count -eq 0) { "All set" } else { "$($missingVars.Count) missing" })" -ForegroundColor $(if($missingVars.Count -eq 0) { "Green" } else { "Red" })
Write-Host "Key directories: $existingDirs/$($directories.Length)" -ForegroundColor $(if($existingDirs -eq $directories.Length) { "Green" } else { "Yellow" })

# Show progress if requested
if ($ShowProgress) {
    Write-Host ""
    Write-Host "=== Checklist Progress ===" -ForegroundColor Cyan
    
    $stats = Get-CompletionStats -ProgressData $progressData
    Write-Host "Overall Progress: $($stats.Completed)/$($stats.Total) ($($stats.Percentage)%)" -ForegroundColor Yellow
    
    if ($stats.Percentage -lt 100) {
        Write-Host ""
        Write-Host "Incomplete Sections:" -ForegroundColor Yellow
        foreach ($section in $progressData.Keys | Sort-Object) {
            if ($progressData[$section] -is [hashtable]) {
                $sectionCompleted = ($progressData[$section].Values | Where-Object { $_ -eq $true } | Measure-Object).Count
                $sectionTotal = $progressData[$section].Count
                
                if ($sectionCompleted -lt $sectionTotal) {
                    Write-Host "  $($section): $sectionCompleted/$sectionTotal" -ForegroundColor Yellow
                }
            }
        }
    }
}

Write-Host ""
Write-Host "Next steps:" -ForegroundColor Cyan
Write-Host "   - Review the ENHANCED_TRADING_CHECKLIST.MD for additional validation steps" -ForegroundColor Yellow
Write-Host "   - Run compliance tests: ./scripts/run_compliance_tests.ps1" -ForegroundColor Yellow
Write-Host "   - Run backtests: ./scripts/run-backtest.ps1" -ForegroundColor Yellow
Write-Host "   - Track your progress: ./scripts/validate-trading-checklist.ps1 -ShowProgress" -ForegroundColor Yellow
Write-Host "   - Reset progress tracking: ./scripts/validate-trading-checklist.ps1 -ResetProgress" -ForegroundColor Yellow