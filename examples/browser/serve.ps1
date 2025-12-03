# Simple HTTP server for browser example
Write-Host "Starting HTTP server for Sockudo browser example..." -ForegroundColor Green

# Check if Python is available
$pythonCmd = Get-Command python -ErrorAction SilentlyContinue

if (-not $pythonCmd) {
    Write-Host "Python not found. Trying python3..." -ForegroundColor Yellow
    $pythonCmd = Get-Command python3 -ErrorAction SilentlyContinue
}

if (-not $pythonCmd) {
    Write-Host "Error: Python is not installed or not in PATH" -ForegroundColor Red
    Write-Host "Please install Python from https://www.python.org/" -ForegroundColor Yellow
    exit 1
}

# Get the project root (2 levels up from this script)
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$projectRoot = Split-Path -Parent (Split-Path -Parent $scriptDir)

Write-Host ""
Write-Host "Project root: $projectRoot" -ForegroundColor Cyan
Write-Host "Serving from: $scriptDir" -ForegroundColor Cyan
Write-Host ""
Write-Host "Open your browser to:" -ForegroundColor Green
Write-Host "  http://localhost:8000" -ForegroundColor White
Write-Host ""
Write-Host "Press Ctrl+C to stop the server" -ForegroundColor Yellow
Write-Host ""

# Start Python HTTP server from project root so pkg/ is accessible
Push-Location $projectRoot
try {
    & $pythonCmd.Source -m http.server 8000 --directory .
} finally {
    Pop-Location
}
