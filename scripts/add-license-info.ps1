# PowerShell script to add license information to all crates

# Get all directories in the crates folder
$cratesDir = "d:\auto-sync-daily-from-winC\snipping-bot\crates"
$crateDirs = Get-ChildItem -Path $cratesDir -Directory

foreach ($crateDir in $crateDirs) {
    $crateName = $crateDir.Name
    $cargoTomlPath = Join-Path $crateDir.FullName "Cargo.toml"
    
    # Check if Cargo.toml exists
    if (Test-Path $cargoTomlPath) {
        # Read the content of Cargo.toml
        $content = Get-Content $cargoTomlPath -Raw
        
        # Check if license is already specified
        if ($content -notmatch 'license\s*=') {
            # Find the [package] section
            if ($content -match '\[package\]') {
                # Add license information after the edition line
                $updatedContent = $content -replace 'edition = "2021"', 'edition = "2021"' + "`nlicense = `"Apache-2.0`""
                
                # Write the updated content back to the file
                Set-Content -Path $cargoTomlPath -Value $updatedContent
                Write-Host "Added license to $crateName" -ForegroundColor Green
            } else {
                Write-Host "No [package] section found in $crateName" -ForegroundColor Yellow
            }
        } else {
            Write-Host "License already specified for $crateName" -ForegroundColor Cyan
        }
    } else {
        Write-Host "Cargo.toml not found for $crateName" -ForegroundColor Red
    }
}

Write-Host "License information added to all crates!" -ForegroundColor Green