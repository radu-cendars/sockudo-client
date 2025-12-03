# Build script for WebAssembly with Emscripten support
param(
    [switch]$Dev,
    [switch]$Debug,
    [string]$Target = "web"
)

Write-Host "Building Sockudo for WebAssembly..." -ForegroundColor Green

# Determine build profile (default is release)
$BuildProfile = "release"
if ($Dev) {
    $BuildProfile = "dev"
} elseif ($Debug) {
    $BuildProfile = "debug"
}

Write-Host "Build Profile: $BuildProfile" -ForegroundColor Cyan
Write-Host "Target: $Target" -ForegroundColor Cyan

# Check if wasm-pack is installed
$wasmPackInstalled = Get-Command wasm-pack -ErrorAction SilentlyContinue
if (-not $wasmPackInstalled) {
    Write-Host "Error: wasm-pack is not installed" -ForegroundColor Red
    Write-Host "Install it with: cargo install wasm-pack" -ForegroundColor Yellow
    exit 1
}

# Load Emscripten environment
$emsdkEnvPath = "C:\emsdk\emsdk_env.ps1"
if (Test-Path $emsdkEnvPath) {
    Write-Host "Loading Emscripten environment..." -ForegroundColor Cyan
    & $emsdkEnvPath

    # Refresh the PATH in current session
    $env:Path = [System.Environment]::GetEnvironmentVariable("Path","Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path","User")

    Write-Host "Emscripten environment loaded" -ForegroundColor Green
} else {
    Write-Host "Error: Emscripten not found at $emsdkEnvPath" -ForegroundColor Red
    Write-Host "Please run setup-emsdk.ps1 first" -ForegroundColor Yellow
    exit 1
}

Write-Host ""
Write-Host "Verifying Emscripten tools..." -ForegroundColor Cyan

# Test emcc
$emccTest = & "C:/emsdk/upstream/emscripten/emcc.bat" "--version" 2>&1
if ($LASTEXITCODE -eq 0) {
    Write-Host "  emcc is working correctly" -ForegroundColor Green
} else {
    Write-Host "  Error: emcc test failed" -ForegroundColor Red
    Write-Host "  Output: $emccTest" -ForegroundColor Gray
    exit 1
}

Write-Host ""
Write-Host "Setting up build environment..." -ForegroundColor Cyan

# Get the full path to the wrapper executable
$wrapperPath = Join-Path $PSScriptRoot "emcc-wrapper.exe"
$wrapperPath = $wrapperPath -replace '\\', '/'

# Compile wrapper if it doesn't exist or source is newer
$wrapperSourcePath = Join-Path $PSScriptRoot "emcc-wrapper.rs"
if (-not (Test-Path $wrapperPath) -or (Get-Item $wrapperSourcePath).LastWriteTime -gt (Get-Item $wrapperPath).LastWriteTime) {
    Write-Host "  Compiling emcc wrapper..." -ForegroundColor Cyan
    & rustc $wrapperSourcePath -o $wrapperPath
    if ($LASTEXITCODE -ne 0) {
        Write-Host "  Error: Failed to compile emcc wrapper" -ForegroundColor Red
        exit 1
    }
}

# Set compiler environment variables for wasm32-unknown-unknown target
# Use compiled wrapper that filters out --target flag
$env:CC_wasm32_unknown_unknown = $wrapperPath
$env:AR_wasm32_unknown_unknown = "C:/emsdk/upstream/emscripten/emar.bat"
$env:CFLAGS_wasm32_unknown_unknown = "-s WASM=1"

# Configure bindgen to use Emscripten sysroot
$emsdkSysroot = "C:/emsdk/upstream/emscripten/cache/sysroot"
$bindgenArgs = "--sysroot=$emsdkSysroot -I$emsdkSysroot/include -I$emsdkSysroot/include/compat -IC:/emsdk/upstream/emscripten/system/include"
$env:BINDGEN_EXTRA_CLANG_ARGS_wasm32_unknown_unknown = $bindgenArgs

Write-Host "  CC_wasm32_unknown_unknown = $wrapperPath (emcc wrapper)" -ForegroundColor Gray
Write-Host "  AR_wasm32_unknown_unknown = C:/emsdk/upstream/emscripten/emar.bat" -ForegroundColor Gray
Write-Host "  BINDGEN_EXTRA_CLANG_ARGS configured for Emscripten sysroot" -ForegroundColor Gray

# Build arguments for wasm-pack
$buildArgs = @(
    "build",
    "--target", $Target,
    "--no-default-features",
    "--features", "wasm"
)

# Add profile flag (release is default for wasm-pack, so only add if different)
if ($BuildProfile -eq "dev") {
    $buildArgs += "--dev"
} elseif ($BuildProfile -eq "debug") {
    $buildArgs += "--debug"
}

Write-Host ""
Write-Host "Running wasm-pack..." -ForegroundColor Cyan
Write-Host "Command: wasm-pack $($buildArgs -join ' ')" -ForegroundColor Gray
Write-Host ""

# Run wasm-pack build
& wasm-pack @buildArgs

if ($LASTEXITCODE -ne 0) {
    Write-Host ""
    Write-Host "Build failed!" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "Running post-build fixes..." -ForegroundColor Cyan
& pwsh -File fix-wasm-exports.ps1

if ($LASTEXITCODE -ne 0) {
    Write-Host ""
    Write-Host "Post-build fix failed!" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "Build successful!" -ForegroundColor Green
Write-Host ""
Write-Host "Output directory: pkg/" -ForegroundColor Cyan

# Show generated files
if (Test-Path "pkg") {
    Write-Host ""
    Write-Host "Generated files:" -ForegroundColor Cyan
    Get-ChildItem -Path "pkg" | ForEach-Object {
        if ($_.PSIsContainer) {
            $size = "<DIR>"
        } else {
            $sizeKB = [math]::Round($_.Length / 1KB, 2)
            $size = "$sizeKB KB"
        }
        Write-Host "  $($_.Name.PadRight(40)) $size" -ForegroundColor Gray
    }
}

Write-Host ""
Write-Host "You can now use the WebAssembly package in your JavaScript project!" -ForegroundColor Green
Write-Host ""
Write-Host "Example:" -ForegroundColor Yellow
Write-Host "  import init, { SockudoClient } from './pkg/sockudo_client.js';" -ForegroundColor White
Write-Host "  await init();" -ForegroundColor White
Write-Host "  const client = new SockudoClient('YOUR_APP_KEY');" -ForegroundColor White
Write-Host ""
Write-Host "Try the browser example:" -ForegroundColor Cyan
Write-Host "  cd examples/browser" -ForegroundColor White
Write-Host "  pwsh serve.ps1          # Windows" -ForegroundColor White
Write-Host "  ./serve.sh              # Linux/Mac" -ForegroundColor White
Write-Host "  Then open: http://localhost:8000/examples/browser/" -ForegroundColor White
Write-Host ""
Write-Host "To build again:" -ForegroundColor Cyan
Write-Host "  .\build-wasm.ps1        # Release build (default)" -ForegroundColor White
Write-Host "  .\build-wasm.ps1 -Dev   # Development build" -ForegroundColor White
Write-Host "  .\build-wasm.ps1 -Debug # Debug build" -ForegroundColor White
