# Simple test service to verify file serving

# Create a test directory
New-Item -ItemType Directory -Path "test-assets" -Force

# Create a simple test file
Set-Content -Path "test-assets/test.html" -Value "<html><body><h1>Test File</h1></body></html>"

Write-Host "Test assets created in test-assets directory"
Write-Host "You can now test file serving with a simple Python server:"
Write-Host "python -m http.server 8000 --directory test-assets"