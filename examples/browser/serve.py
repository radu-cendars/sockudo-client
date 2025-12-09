#!/usr/bin/env python3

"""
Simple HTTP server for testing the Sockudo WebSocket client
Run with: python serve.py
"""

import http.server
import socketserver
import os
import sys
from pathlib import Path

PORT = 8080
HOST = "localhost"

class CORSRequestHandler(http.server.SimpleHTTPRequestHandler):
    """HTTP request handler with CORS support for WASM"""

    def __init__(self, *args, **kwargs):
        # Set the directory to serve files from
        super().__init__(*args, directory=str(Path(__file__).parent), **kwargs)

    def end_headers(self):
        # Add CORS headers
        self.send_header('Access-Control-Allow-Origin', '*')
        self.send_header('Access-Control-Allow-Methods', 'GET, POST, OPTIONS')
        self.send_header('Access-Control-Allow-Headers', '*')
        self.send_header('Cross-Origin-Opener-Policy', 'same-origin')
        self.send_header('Cross-Origin-Embedder-Policy', 'require-corp')
        super().end_headers()

    def guess_type(self, path):
        """Add WASM MIME type support"""
        if path.endswith('.wasm'):
            return 'application/wasm'
        elif path.endswith('.js'):
            return 'text/javascript'
        return super().guess_type(path)

    def translate_path(self, path):
        """Translate URL path to filesystem path, handling /pkg/ specially"""
        # Remove query string
        path = path.split('?', 1)[0]
        path = path.split('#', 1)[0]

        # Default to test.html for root
        if path == '/':
            path = '/test.html'

        # Handle /pkg/ URLs - serve from ../../pkg/
        if path.startswith('/pkg/'):
            pkg_path = Path(__file__).parent.parent.parent / 'pkg' / path[5:]
            return str(pkg_path)

        # Serve from current directory
        return super().translate_path(path)

    def log_message(self, format, *args):
        """Custom log format"""
        sys.stdout.write(f"[{self.log_date_time_string()}] {format % args}\n")


def run_server():
    """Start the HTTP server"""
    handler = CORSRequestHandler

    with socketserver.TCPServer((HOST, PORT), handler) as httpd:
        print("=" * 60)
        print("🚀 Sockudo Test Server Started!")
        print("=" * 60)
        print(f"📍 Server running at: http://{HOST}:{PORT}/")
        print(f"📄 Test page: http://{HOST}:{PORT}/test.html")
        print()
        print("Press Ctrl+C to stop the server")
        print("=" * 60)

        try:
            httpd.serve_forever()
        except KeyboardInterrupt:
            print("\n\n👋 Shutting down server...")
            httpd.shutdown()
            print("✅ Server closed")


if __name__ == "__main__":
    # Check if pkg directory exists
    pkg_dir = Path(__file__).parent.parent.parent / 'pkg'
    if not pkg_dir.exists():
        print("⚠️  WARNING: pkg directory not found!")
        print("Please run the WASM build first:")
        print("  ./build-wasm.sh")
        print()
        response = input("Continue anyway? (y/N): ")
        if response.lower() != 'y':
            sys.exit(1)

    run_server()
