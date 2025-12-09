# Simple HTTP server for testing the Sockudo WebSocket client
# Run with: .\serve.ps1

$Port = 8080
$Host = "localhost"

Write-Host "=" -NoNewline -ForegroundColor Cyan
Write-Host ("=" * 59) -ForegroundColor Cyan
Write-Host "🚀 Sockudo Test Server Starting..." -ForegroundColor Green
Write-Host "=" -NoNewline -ForegroundColor Cyan
Write-Host ("=" * 59) -ForegroundColor Cyan

# Check if pkg directory exists
$pkgDir = Join-Path $PSScriptRoot "..\..\pkg"
if (-not (Test-Path $pkgDir)) {
    Write-Host "⚠️  WARNING: pkg directory not found!" -ForegroundColor Yellow
    Write-Host "Please run the WASM build first:"
    Write-Host "  .\build-wasm.ps1" -ForegroundColor Cyan
    Write-Host ""
    $response = Read-Host "Continue anyway? (y/N)"
    if ($response -ne "y") {
        exit 1
    }
}

# Define MIME types
$mimeTypes = @{
    '.html' = 'text/html'
    '.js'   = 'text/javascript'
    '.wasm' = 'application/wasm'
    '.css'  = 'text/css'
    '.json' = 'application/json'
    '.png'  = 'image/png'
    '.jpg'  = 'image/jpeg'
    '.gif'  = 'image/gif'
    '.svg'  = 'image/svg+xml'
    '.ico'  = 'image/x-icon'
}

# Create HTTP listener
$listener = New-Object System.Net.HttpListener
$listener.Prefixes.Add("http://${Host}:${Port}/")

try {
    $listener.Start()

    Write-Host "📍 Server running at: http://${Host}:${Port}/" -ForegroundColor Green
    Write-Host "📄 Test page: http://${Host}:${Port}/test.html" -ForegroundColor Green
    Write-Host ""
    Write-Host "Press Ctrl+C to stop the server" -ForegroundColor Yellow
    Write-Host "=" -NoNewline -ForegroundColor Cyan
    Write-Host ("=" * 59) -ForegroundColor Cyan
    Write-Host ""

    while ($listener.IsListening) {
        # Get context
        $context = $listener.GetContext()
        $request = $context.Request
        $response = $context.Response

        # Get URL path
        $urlPath = $request.Url.LocalPath

        # Log request
        $timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
        Write-Host "[$timestamp] $($request.HttpMethod) $urlPath"

        # Default to test.html for root
        if ($urlPath -eq '/') {
            $urlPath = '/test.html'
        }

        # Remove query string
        $urlPath = $urlPath -replace '\?.*$', ''

        # Security: prevent directory traversal
        if ($urlPath -match '\.\.') {
            $response.StatusCode = 403
            $buffer = [System.Text.Encoding]::UTF8.GetBytes("403 Forbidden")
            $response.ContentLength64 = $buffer.Length
            $response.OutputStream.Write($buffer, 0, $buffer.Length)
            $response.Close()
            continue
        }

        # Determine file path
        if ($urlPath -match '^/pkg/(.+)$') {
            # Serve from ../../pkg/
            $filePath = Join-Path $PSScriptRoot "..\..\pkg\$($matches[1])"
        } else {
            # Serve from current directory
            $filePath = Join-Path $PSScriptRoot $urlPath.TrimStart('/')
        }

        # Check if file exists
        if (Test-Path $filePath -PathType Leaf) {
            # Get MIME type
            $extension = [System.IO.Path]::GetExtension($filePath).ToLower()
            $contentType = $mimeTypes[$extension]
            if (-not $contentType) {
                $contentType = 'application/octet-stream'
            }

            # Read file
            $content = [System.IO.File]::ReadAllBytes($filePath)

            # Set headers
            $response.ContentType = $contentType
            $response.ContentLength64 = $content.Length
            $response.AddHeader("Access-Control-Allow-Origin", "*")
            $response.AddHeader("Cross-Origin-Opener-Policy", "same-origin")
            $response.AddHeader("Cross-Origin-Embedder-Policy", "require-corp")

            # Send response
            $response.StatusCode = 200
            $response.OutputStream.Write($content, 0, $content.Length)
        } else {
            # File not found
            $response.StatusCode = 404
            $html = @"
<!DOCTYPE html>
<html>
<head><title>404 Not Found</title></head>
<body>
    <h1>404 - File Not Found</h1>
    <p>The requested file <code>$urlPath</code> was not found.</p>
    <p>Looking for: <code>$filePath</code></p>
    <a href="/">Go back to home</a>
</body>
</html>
"@
            $buffer = [System.Text.Encoding]::UTF8.GetBytes($html)
            $response.ContentType = "text/html"
            $response.ContentLength64 = $buffer.Length
            $response.OutputStream.Write($buffer, 0, $buffer.Length)
        }

        $response.Close()
    }
} catch {
    if ($_.Exception.InnerException -is [System.Net.HttpListenerException]) {
        $errorCode = $_.Exception.InnerException.ErrorCode
        if ($errorCode -eq 5) {
            Write-Host ""
            Write-Host "❌ Access Denied!" -ForegroundColor Red
            Write-Host "Run PowerShell as Administrator, or use a different port." -ForegroundColor Yellow
            Write-Host "Alternatively, run this command as Administrator once:" -ForegroundColor Yellow
            Write-Host "  netsh http add urlacl url=http://+:${Port}/ user=Everyone" -ForegroundColor Cyan
        } elseif ($errorCode -eq 183) {
            Write-Host ""
            Write-Host "❌ Port $Port is already in use!" -ForegroundColor Red
            Write-Host "Please close the other application or use a different port." -ForegroundColor Yellow
        } else {
            Write-Host ""
            Write-Host "❌ Error: $($_.Exception.Message)" -ForegroundColor Red
        }
        exit 1
    }

    # Handle Ctrl+C gracefully
    if ($_.Exception -is [System.Management.Automation.PipelineStoppedException]) {
        Write-Host ""
    } else {
        Write-Host ""
        Write-Host "❌ Error: $($_.Exception.Message)" -ForegroundColor Red
    }
} finally {
    if ($listener.IsListening) {
        $listener.Stop()
    }
    $listener.Close()
    Write-Host ""
    Write-Host "👋 Shutting down server..." -ForegroundColor Yellow
    Write-Host "✅ Server closed" -ForegroundColor Green
}
