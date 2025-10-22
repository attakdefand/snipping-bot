# Checklist Progress Update Script
# This script helps users update their progress on the enhanced trading checklist

param(
    [string]$ProgressFile = "configs/checklist-progress.toml",
    [string]$Section,
    [string]$Item,
    [switch]$Complete,
    [switch]$Incomplete,
    [switch]$ListSections,
    [switch]$ListItems
)

function Get-TomlContent {
    param([string]$Path)
    
    if (-not (Test-Path $Path)) {
        Write-Host "Progress file not found: $Path" -ForegroundColor Red
        return $null
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

function Set-TomlValue {
    param(
        [string]$Path,
        [string]$TargetSection,
        [string]$TargetKey,
        [bool]$Value
    )
    
    if (-not (Test-Path $Path)) {
        Write-Host "Progress file not found: $Path" -ForegroundColor Red
        return $false
    }
    
    $content = Get-Content $Path
    $newContent = @()
    $currentSection = ""
    $updated = $false
    
    foreach ($line in $content) {
        # Check for section headers
        if ($line.Trim().StartsWith("[") -and $line.Trim().EndsWith("]")) {
            $currentSection = $line.Trim().Trim('[', ']')
            $newContent += $line
            continue
        }
        
        # Check for key-value pairs in the target section
        if ($currentSection -eq $TargetSection -and $line.Contains("=")) {
            $parts = $line.Split("=", 2)
            $key = $parts[0].Trim()
            
            # Only update the specific key
            if ($key -eq $TargetKey) {
                $newContent += "$key = $($Value.ToString().ToLower())"
                $updated = $true
                continue
            }
        }
        
        $newContent += $line
    }
    
    if ($updated) {
        $newContent | Set-Content $Path
        return $true
    } else {
        Write-Host "Item not found: $TargetSection.$TargetKey" -ForegroundColor Red
        return $false
    }
}

function Show-Sections {
    param([hashtable]$Data)
    
    Write-Host "Available sections:" -ForegroundColor Cyan
    foreach ($section in $Data.Keys | Sort-Object) {
        Write-Host "  - $section" -ForegroundColor Yellow
    }
}

function Show-Items {
    param(
        [hashtable]$Data,
        [string]$SectionName
    )
    
    if ($SectionName) {
        if ($Data.ContainsKey($SectionName)) {
            Write-Host "Items in section '$SectionName':" -ForegroundColor Cyan
            foreach ($item in $Data[$SectionName].Keys | Sort-Object) {
                $status = if ($Data[$SectionName][$item]) { "Completed" } else { "Pending" }
                $color = if ($Data[$SectionName][$item]) { "Green" } else { "Yellow" }
                Write-Host "  - $item : $status" -ForegroundColor $color
            }
        } else {
            Write-Host "Section '$SectionName' not found." -ForegroundColor Red
        }
    } else {
        Write-Host "Please specify a section with -Section parameter." -ForegroundColor Yellow
        Show-Sections -Data $Data
    }
}

# Main script logic
if (-not (Test-Path $ProgressFile)) {
    Write-Host "Progress file not found. Creating from template..." -ForegroundColor Yellow
    if (Test-Path "configs/checklist-progress.template.toml") {
        Copy-Item "configs/checklist-progress.template.toml" $ProgressFile
        Write-Host "Created progress file from template." -ForegroundColor Green
    } else {
        Write-Host "Template file not found. Please run validate-trading-checklist.ps1 first." -ForegroundColor Red
        exit 1
    }
}

$progressData = Get-TomlContent -Path $ProgressFile

if ($ListSections) {
    Show-Sections -Data $progressData
    exit 0
}

if ($ListItems) {
    Show-Items -Data $progressData -SectionName $Section
    exit 0
}

if ($Section -and $Item) {
    if ($Complete -and $Incomplete) {
        Write-Host "Please specify either -Complete or -Incomplete, not both." -ForegroundColor Red
        exit 1
    }
    
    if (-not $Complete -and -not $Incomplete) {
        Write-Host "Please specify either -Complete or -Incomplete." -ForegroundColor Red
        exit 1
    }
    
    $newValue = if ($Complete) { $true } else { $false }
    $result = Set-TomlValue -Path $ProgressFile -TargetSection $Section -TargetKey $Item -Value $newValue
    
    if ($result) {
        $status = if ($Complete) { "completed" } else { "marked as incomplete" }
        Write-Host "Successfully $status item: $Section.$Item" -ForegroundColor Green
    } else {
        Write-Host "Failed to update item: $Section.$Item" -ForegroundColor Red
        exit 1
    }
} else {
    Write-Host "Usage:" -ForegroundColor Cyan
    Write-Host "  List sections: ./scripts/update-checklist-progress.ps1 -ListSections" -ForegroundColor Yellow
    Write-Host "  List items in a section: ./scripts/update-checklist-progress.ps1 -ListItems -Section 'section_name'" -ForegroundColor Yellow
    Write-Host "  Mark item as complete: ./scripts/update-checklist-progress.ps1 -Section 'section_name' -Item 'item_name' -Complete" -ForegroundColor Yellow
    Write-Host "  Mark item as incomplete: ./scripts/update-checklist-progress.ps1 -Section 'section_name' -Item 'item_name' -Incomplete" -ForegroundColor Yellow
}