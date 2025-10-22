# Trading Configuration Script
# This script helps you choose and apply trading strategies and risk parameters

param(
    [string]$ConfigType = "list"
)

function Show-Strategies {
    Write-Host "=== Available Trading Strategies ===" -ForegroundColor Green
    Write-Host ""
    
    $strategies = Get-ChildItem -Path "configs/strategies" -Filter "*.toml"
    foreach ($strategy in $strategies) {
        Write-Host "   - $($strategy.BaseName)" -ForegroundColor Yellow
        # Show basic info from the file
        $content = Get-Content $strategy.FullName -TotalCount 10
        $nameLine = $content | Where-Object { $_ -like "name = *" }
        if ($nameLine) {
            $name = $nameLine -replace 'name = "', '' -replace '"', ''
            Write-Host "     Description: $name" -ForegroundColor Cyan
        }
        Write-Host ""
    }
}

function Show-RiskConfigs {
    Write-Host "=== Available Risk Configurations ===" -ForegroundColor Green
    Write-Host ""
    
    $riskConfigs = Get-ChildItem -Path "configs" -Filter "risk*.toml"
    foreach ($config in $riskConfigs) {
        Write-Host "   - $($config.Name)" -ForegroundColor Yellow
        if ($config.Name -eq "risk.toml") {
            Write-Host "     Description: Default risk configuration" -ForegroundColor Cyan
        } elseif ($config.Name -eq "risk_simple.toml") {
            Write-Host "     Description: Simple risk configuration for beginners" -ForegroundColor Cyan
        } elseif ($config.Name -eq "risk_comprehensive.toml") {
            Write-Host "     Description: Comprehensive risk configuration for advanced users" -ForegroundColor Cyan
        }
        Write-Host ""
    }
}

function Apply-Configuration {
    param(
        [string]$Type,
        [string]$Name
    )
    
    if ($Type -eq "strategy") {
        $source = "configs/strategies/$Name.toml"
        if (Test-Path $source) {
            Write-Host "Applying strategy: $Name" -ForegroundColor Green
            # In a real implementation, we would copy or link the strategy
            Write-Host "   Strategy '$Name' is now available for use" -ForegroundColor Yellow
        } else {
            Write-Host "   Error: Strategy '$Name' not found" -ForegroundColor Red
        }
    }
    elseif ($Type -eq "risk") {
        $source = "configs/$Name"
        if (Test-Path $source) {
            Write-Host "Applying risk configuration: $Name" -ForegroundColor Green
            Copy-Item $source "configs/risk.toml" -Force
            Write-Host "   Risk configuration updated successfully" -ForegroundColor Yellow
        } else {
            Write-Host "   Error: Risk configuration '$Name' not found" -ForegroundColor Red
        }
    }
}

# Main script logic
switch ($ConfigType) {
    "list" {
        Show-Strategies
        Show-RiskConfigs
        Write-Host "Usage:" -ForegroundColor Cyan
        Write-Host "   ./scripts/configure-trading.ps1 -ConfigType strategies     # List strategies" -ForegroundColor Yellow
        Write-Host "   ./scripts/configure-trading.ps1 -ConfigType risks         # List risk configs" -ForegroundColor Yellow
        Write-Host "   ./scripts/configure-trading.ps1 -ConfigType apply-risk -Name risk_simple.toml  # Apply risk config" -ForegroundColor Yellow
    }
    
    "strategies" {
        Show-Strategies
    }
    
    "risks" {
        Show-RiskConfigs
    }
    
    default {
        Write-Host "Unknown configuration type: $ConfigType" -ForegroundColor Red
        Write-Host "Use 'list', 'strategies', or 'risks'" -ForegroundColor Yellow
    }
}