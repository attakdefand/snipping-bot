# PowerShell script to verify security compliance based on the security layers checklist

param(
    [Parameter(Mandatory=$false)]
    [string]$OutputFormat = "table",  # table, csv, json
    [Parameter(Mandatory=$false)]
    [string]$LayerFilter = "",        # Filter by layer number
    [Parameter(Mandatory=$false)]
    [switch]$ShowOnlyMissing        # Show only missing controls
)

Write-Host "Security Compliance Verification" -ForegroundColor Green
Write-Host "===============================" -ForegroundColor Green
Write-Host ""

# Function to check if a file exists
function Test-FileExists {
    param([string]$Path)
    return Test-Path $Path -PathType Leaf
}

# Function to check if a directory exists
function Test-DirectoryExists {
    param([string]$Path)
    return Test-Path $Path -PathType Container
}

# Function to check if a command exists
function Test-CommandExists {
    param([string]$Command)
    return [bool](Get-Command -Name $Command -ErrorAction SilentlyContinue)
}

# Load the security layers checklist
$checklistPath = "security_layers_checklist.csv"
if (-not (Test-FileExists $checklistPath)) {
    Write-Host "❌ Security layers checklist not found at $checklistPath" -ForegroundColor Red
    exit 1
}

# Read the CSV file
$checklist = Import-Csv $checklistPath

# Filter by layer if specified
if ($LayerFilter -ne "") {
    $checklist = $checklist | Where-Object { $_."Layer #" -eq $LayerFilter }
}

# Initialize results array
$results = @()

# Process each control in the checklist
foreach ($control in $checklist) {
    $layerNum = $control."Layer #"
    $layerName = $control."Layer Name"
    $controlGroup = $control."Control Group"
    $controlName = $control."Control"
    $artifact = $control."Policy/Config Artifact"
    $component = $control."Component (Rust/K8s/Web3)"
    $testCategory = $control."Test Category"
    $metric = $control."Metric/KPI"
    
    # Initialize status
    $status = "Unknown"
    $details = ""
    
    # Check if artifact exists
    if ($artifact -ne "") {
        # Handle wildcard patterns
        if ($artifact -like "*.*") {
            # It's a file pattern
            if (Test-FileExists $artifact) {
                $status = "Implemented"
                $details = "Artifact found"
            } else {
                # Try to find files matching the pattern
                $pattern = $artifact -replace '\*', '.*' -replace '\?', '.'
                $foundFiles = Get-ChildItem -Path . -Recurse -Name | Where-Object { $_ -match $pattern }
                if ($foundFiles.Count -gt 0) {
                    $status = "Implemented"
                    $details = "Pattern matched $($foundFiles.Count) files"
                } else {
                    $status = "Missing"
                    $details = "Artifact not found"
                }
            }
        } else {
            # It's a directory or path
            if (Test-DirectoryExists $artifact) {
                $status = "Implemented"
                $details = "Directory exists"
            } elseif (Test-FileExists $artifact) {
                $status = "Implemented"
                $details = "File exists"
            } else {
                # Check if it's a relative path that might exist in subdirectories
                $fullPath = Join-Path "." $artifact
                if (Test-Path $fullPath) {
                    $status = "Implemented"
                    $details = "Path exists"
                } else {
                    $status = "Missing"
                    $details = "Path not found"
                }
            }
        }
    } else {
        $status = "Not Applicable"
        $details = "No artifact specified"
    }
    
    # Create result object
    $result = [PSCustomObject]@{
        "Layer #" = $layerNum
        "Layer Name" = $layerName
        "Control Group" = $controlGroup
        "Control" = $controlName
        "Status" = $status
        "Details" = $details
        "Artifact" = $artifact
        "Component" = $component
        "Test Category" = $testCategory
        "Metric/KPI" = $metric
    }
    
    # Add to results unless we're only showing missing and this isn't missing
    if (-not $ShowOnlyMissing -or $status -eq "Missing") {
        $results += $result
    }
}

# Output results based on format
switch ($OutputFormat) {
    "csv" {
        $results | Export-Csv -Path "security-compliance-report.csv" -NoTypeInformation
        Write-Host "✅ Compliance report saved to security-compliance-report.csv" -ForegroundColor Green
    }
    "json" {
        $results | ConvertTo-Json | Out-File -FilePath "security-compliance-report.json"
        Write-Host "✅ Compliance report saved to security-compliance-report.json" -ForegroundColor Green
    }
    default {
        # Display summary
        $totalControls = $results.Count
        $implementedControls = ($results | Where-Object { $_.Status -eq "Implemented" }).Count
        $missingControls = ($results | Where-Object { $_.Status -eq "Missing" }).Count
        $notApplicableControls = ($results | Where-Object { $_.Status -eq "Not Applicable" }).Count
        
        Write-Host "Compliance Summary:" -ForegroundColor Cyan
        Write-Host "===================" -ForegroundColor Cyan
        Write-Host "Total Controls: $totalControls" -ForegroundColor White
        Write-Host "Implemented: $implementedControls" -ForegroundColor Green
        Write-Host "Missing: $missingControls" -ForegroundColor Red
        Write-Host "Not Applicable: $notApplicableControls" -ForegroundColor Yellow
        Write-Host "Compliance Rate: $([math]::Round(($implementedControls / $totalControls * 100), 2))%" -ForegroundColor Cyan
        Write-Host ""
        
        # Show detailed results in a table
        if ($results.Count -gt 0) {
            $results | Format-Table -Property "Layer #", "Layer Name", "Control Group", "Control", "Status", "Details" -AutoSize
        }
        
        # Show only missing controls if requested
        if ($missingControls -gt 0) {
            Write-Host "Missing Controls:" -ForegroundColor Red
            Write-Host "=================" -ForegroundColor Red
            $results | Where-Object { $_.Status -eq "Missing" } | Format-Table -Property "Layer #", "Layer Name", "Control Group", "Control", "Artifact" -AutoSize
        }
    }
}

Write-Host ""
Write-Host "Security compliance verification completed!" -ForegroundColor Green