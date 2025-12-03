# Setup Emscripten SDK for WASM builds

$emsdkPath = "C:\emsdk"

Write-Host "Setting up Emscripten environment..." -ForegroundColor Cyan

if (Test-Path $emsdkPath) {
    & "$emsdkPath\emsdk_env.ps1"
    Write-Host "Emscripten environment loaded successfully!" -ForegroundColor Green
} else {
    Write-Host "Error: Emscripten not found at $emsdkPath" -ForegroundColor Red
    Write-Host "Please install Emscripten first:" -ForegroundColor Yellow
    Write-Host "  git clone https://github.com/emscripten-core/emsdk.git C:\emsdk" -ForegroundColor White
    Write-Host "  cd C:\emsdk" -ForegroundColor White
    Write-Host "  .\emsdk install latest" -ForegroundColor White
    Write-Host "  .\emsdk activate latest" -ForegroundColor White
    exit 1
}
