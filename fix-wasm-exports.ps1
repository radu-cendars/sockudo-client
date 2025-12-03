# Post-build script to fix WASM exports
# This copies methods from WasmSockudo to Sockudo since wasm-bindgen's js_name doesn't copy methods

Write-Host "Fixing WASM exports..." -ForegroundColor Cyan

$jsFile = "pkg/sockudo_client.js"

if (-not (Test-Path $jsFile)) {
    Write-Host "Error: $jsFile not found" -ForegroundColor Red
    exit 1
}

# Read the file
$content = Get-Content $jsFile -Raw

# Add a fix at the end of the file to copy WasmSockudo methods to Sockudo
$fix = @"

// Fix: Copy WasmSockudo prototype methods to Sockudo prototype
// This is needed because wasm-bindgen's js_name doesn't copy methods
Object.getOwnPropertyNames(WasmSockudo.prototype).forEach(name => {
    if (name !== 'constructor' && name !== '__destroy_into_raw' && name !== 'free') {
        const descriptor = Object.getOwnPropertyDescriptor(WasmSockudo.prototype, name);
        if (descriptor) {
            Object.defineProperty(Sockudo.prototype, name, descriptor);
        }
    }
});

// Fix: Copy WasmChannel prototype methods to Channel prototype
// WasmChannel methods return Channel objects, so Channel needs the methods
Object.getOwnPropertyNames(WasmChannel.prototype).forEach(name => {
    if (name !== 'constructor' && name !== '__destroy_into_raw' && name !== 'free') {
        const descriptor = Object.getOwnPropertyDescriptor(WasmChannel.prototype, name);
        if (descriptor) {
            Object.defineProperty(Channel.prototype, name, descriptor);
        }
    }
});

// Make Sockudo constructor work by delegating to WasmSockudo
const OriginalSockudo = Sockudo;
Sockudo = function(app_key, options) {
    return new WasmSockudo(app_key, options);
};
Sockudo.prototype = OriginalSockudo.prototype;
Sockudo.__wrap = OriginalSockudo.__wrap;

// Export the fixed Sockudo as SockudoClient
export { Sockudo as SockudoClient };

// Export WasmOptions as SockudoOptions for better readability
export { WasmOptions as SockudoOptions };
"@

# Check if fix is already applied
if ($content -match "Copy WasmSockudo prototype methods") {
    Write-Host "Fix already applied, skipping..." -ForegroundColor Yellow
    exit 0
}

# Find the position to insert (before the last export default line)
$insertPosition = $content.LastIndexOf("export default __wbg_init;")

if ($insertPosition -gt 0) {
    # Insert the fix before the last export
    $newContent = $content.Substring(0, $insertPosition) + $fix + "`n" + $content.Substring($insertPosition)

    # Write back to file
    Set-Content -Path $jsFile -Value $newContent -NoNewline

    Write-Host "✅ Successfully fixed WASM exports" -ForegroundColor Green
    Write-Host "   - Copied methods from WasmSockudo to Sockudo" -ForegroundColor Gray
    Write-Host "   - Fixed SockudoOptions to properly expose setters" -ForegroundColor Gray
    Write-Host "   - Made Sockudo constructor delegate to WasmSockudo" -ForegroundColor Gray
    Write-Host "   - Exported Sockudo as SockudoClient" -ForegroundColor Gray
} else {
    Write-Host "Error: Could not find insertion point in $jsFile" -ForegroundColor Red
    exit 1
}
