#!/bin/bash

# Simple HTTP server for browser example
echo "Starting HTTP server for Sockudo browser example..."

# Check if Python is available
if command -v python3 &> /dev/null; then
    PYTHON_CMD="python3"
elif command -v python &> /dev/null; then
    PYTHON_CMD="python"
else
    echo "Error: Python is not installed or not in PATH"
    echo "Please install Python from https://www.python.org/"
    exit 1
fi

# Get the project root (2 levels up from this script)
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$( cd "$SCRIPT_DIR/../.." && pwd )"

echo ""
echo "Project root: $PROJECT_ROOT"
echo "Serving from: $SCRIPT_DIR"
echo ""
echo "Open your browser to:"
echo "  http://localhost:8000"
echo ""
echo "Then navigate to: examples/browser/"
echo ""
echo "Press Ctrl+C to stop the server"
echo ""

# Start Python HTTP server from project root so pkg/ is accessible
cd "$PROJECT_ROOT"
$PYTHON_CMD -m http.server 8000
