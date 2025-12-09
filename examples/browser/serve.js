#!/usr/bin/env node

/**
 * Simple HTTP server for testing the Sockudo WebSocket client
 * Run with: node serve.js
 */

const http = require('http');
const fs = require('fs');
const path = require('path');

const PORT = 8080;
const HOST = 'localhost';

// MIME types
const mimeTypes = {
    '.html': 'text/html',
    '.js': 'text/javascript',
    '.wasm': 'application/wasm',
    '.css': 'text/css',
    '.json': 'application/json',
    '.png': 'image/png',
    '.jpg': 'image/jpeg',
    '.gif': 'image/gif',
    '.svg': 'image/svg+xml',
    '.ico': 'image/x-icon'
};

const server = http.createServer((req, res) => {
    console.log(`${new Date().toISOString()} - ${req.method} ${req.url}`);

    // Parse URL
    let filePath = req.url === '/' ? '/test.html' : req.url;

    // Remove query string
    filePath = filePath.split('?')[0];

    // Security: prevent directory traversal
    if (filePath.includes('..')) {
        res.writeHead(403, { 'Content-Type': 'text/plain' });
        res.end('403 Forbidden');
        return;
    }

    // Map URLs to file paths
    let fullPath;
    if (filePath.startsWith('/pkg/')) {
        // Serve WASM package files from ../../pkg/
        fullPath = path.join(__dirname, '../../pkg', filePath.substring(5));
    } else {
        // Serve HTML/CSS/JS from current directory
        fullPath = path.join(__dirname, filePath);
    }

    // Get file extension
    const ext = path.extname(fullPath).toLowerCase();
    const contentType = mimeTypes[ext] || 'application/octet-stream';

    // Read and serve file
    fs.readFile(fullPath, (err, content) => {
        if (err) {
            if (err.code === 'ENOENT') {
                res.writeHead(404, { 'Content-Type': 'text/html' });
                res.end(`
                    <!DOCTYPE html>
                    <html>
                    <head><title>404 Not Found</title></head>
                    <body>
                        <h1>404 - File Not Found</h1>
                        <p>The requested file <code>${filePath}</code> was not found.</p>
                        <p>Looking for: <code>${fullPath}</code></p>
                        <a href="/">Go back to home</a>
                    </body>
                    </html>
                `);
            } else {
                res.writeHead(500, { 'Content-Type': 'text/plain' });
                res.end(`500 Internal Server Error: ${err.code}`);
            }
        } else {
            // Add CORS headers for WASM
            res.writeHead(200, {
                'Content-Type': contentType,
                'Cross-Origin-Opener-Policy': 'same-origin',
                'Cross-Origin-Embedder-Policy': 'require-corp',
                'Access-Control-Allow-Origin': '*'
            });
            res.end(content);
        }
    });
});

server.listen(PORT, HOST, () => {
    console.log('='.repeat(60));
    console.log('🚀 Sockudo Test Server Started!');
    console.log('='.repeat(60));
    console.log(`📍 Server running at: http://${HOST}:${PORT}/`);
    console.log(`📄 Test page: http://${HOST}:${PORT}/test.html`);
    console.log('');
    console.log('Press Ctrl+C to stop the server');
    console.log('='.repeat(60));
});

// Graceful shutdown
process.on('SIGINT', () => {
    console.log('\n\n👋 Shutting down server...');
    server.close(() => {
        console.log('✅ Server closed');
        process.exit(0);
    });
});

process.on('SIGTERM', () => {
    console.log('\n\n👋 Shutting down server...');
    server.close(() => {
        console.log('✅ Server closed');
        process.exit(0);
    });
});
