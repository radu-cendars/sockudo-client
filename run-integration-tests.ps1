# Integration Test Runner for Delta Compression
# This script helps run integration tests against a sockudo server

param(
    [string]$ServerHost = "localhost",
    [int]$ServerPort = 6001,
    [string]$AppKey = "app-key",
    [switch]$UseTLS = $false,
    [string]$TestFilter = "",
    [switch]$Verbose = $false
)

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Delta Compression Integration Tests" -ForegroundColor Cyan
Write-Host "========================================`n" -ForegroundColor Cyan

# Check if sockudo server is running
Write-Host "Checking server connectivity..." -ForegroundColor Yellow
try {
    $protocol = if ($UseTLS) { "wss" } else { "ws" }
    Write-Host "  Server: $protocol://${ServerHost}:${ServerPort}" -ForegroundColor Gray
    Write-Host "  App Key: $AppKey" -ForegroundColor Gray
} catch {
    Write-Host "❌ Failed to check server" -ForegroundColor Red
}

Write-Host "`nPreparing test environment..." -ForegroundColor Yellow

# Set environment variables for tests
$env:TEST_SERVER_HOST = $ServerHost
$env:TEST_SERVER_PORT = $ServerPort
$env:TEST_APP_KEY = $AppKey
$env:TEST_USE_TLS = if ($UseTLS) { "true" } else { "false" }

# Build the project first
Write-Host "Building project..." -ForegroundColor Yellow
cargo build --tests 2>&1 | Out-Null

if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Build failed!" -ForegroundColor Red
    exit 1
}

Write-Host "✅ Build successful`n" -ForegroundColor Green

# Run tests
Write-Host "Running integration tests..." -ForegroundColor Yellow
Write-Host "========================================`n" -ForegroundColor Cyan

$testCommand = "cargo test --test integration_delta_compression -- --ignored"

if ($TestFilter) {
    $testCommand += " $TestFilter"
}

if ($Verbose) {
    $testCommand += " --nocapture"
}

Write-Host "Command: $testCommand`n" -ForegroundColor Gray

Invoke-Expression $testCommand

$exitCode = $LASTEXITCODE

Write-Host "`n========================================" -ForegroundColor Cyan

if ($exitCode -eq 0) {
    Write-Host "✅ All tests passed!" -ForegroundColor Green
} else {
    Write-Host "❌ Some tests failed" -ForegroundColor Red
}

Write-Host "========================================" -ForegroundColor Cyan

exit $exitCode
