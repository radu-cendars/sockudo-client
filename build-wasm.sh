#!/bin/bash
# Build script for WebAssembly with Emscripten support

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
GRAY='\033[0;90m'
NC='\033[0m' # No Color

# Default values
BUILD_PROFILE="release"
TARGET="web"

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --dev)
            BUILD_PROFILE="dev"
            shift
            ;;
        --debug)
            BUILD_PROFILE="debug"
            shift
            ;;
        --target)
            TARGET="$2"
            shift 2
            ;;
        -h|--help)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --dev              Build in development mode"
            echo "  --debug            Build in debug mode"
            echo "  --target TARGET    Target platform (default: web)"
            echo "  -h, --help         Show this help message"
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            exit 1
            ;;
    esac
done

echo -e "${GREEN}Building Sockudo for WebAssembly...${NC}"
echo ""

echo -e "${CYAN}Build Profile: $BUILD_PROFILE${NC}"
echo -e "${CYAN}Target: $TARGET${NC}"
echo ""

# Check if wasm-pack is installed
if ! command -v wasm-pack &> /dev/null; then
    echo -e "${RED}Error: wasm-pack is not installed${NC}"
    echo -e "${YELLOW}Install it with: cargo install wasm-pack${NC}"
    exit 1
fi

# Detect Emscripten installation location
EMSDK_PATH=""
if [ -d "$HOME/emsdk" ]; then
    EMSDK_PATH="$HOME/emsdk"
elif [ -d "/usr/lib/emsdk" ]; then
    EMSDK_PATH="/usr/lib/emsdk"
elif [ -d "/opt/emsdk" ]; then
    EMSDK_PATH="/opt/emsdk"
fi

# Load Emscripten environment
if [ -n "$EMSDK_PATH" ] && [ -f "$EMSDK_PATH/emsdk_env.sh" ]; then
    echo -e "${CYAN}Loading Emscripten environment...${NC}"
    source "$EMSDK_PATH/emsdk_env.sh" > /dev/null 2>&1
    echo -e "${GREEN}Emscripten environment loaded${NC}"
else
    echo -e "${RED}Error: Emscripten not found${NC}"
    echo -e "${YELLOW}Install Emscripten with:${NC}"
    echo -e "${YELLOW}  git clone https://github.com/emscripten-core/emsdk.git ~/emsdk${NC}"
    echo -e "${YELLOW}  cd ~/emsdk${NC}"
    echo -e "${YELLOW}  ./emsdk install latest${NC}"
    echo -e "${YELLOW}  ./emsdk activate latest${NC}"
    exit 1
fi

echo ""
echo -e "${CYAN}Verifying Emscripten tools...${NC}"

# Test emcc
if command -v emcc &> /dev/null; then
    EMCC_VERSION=$(emcc --version 2>&1 | head -n 1)
    echo -e "  ${GREEN}emcc is working correctly${NC}"
    echo -e "  ${GRAY}$EMCC_VERSION${NC}"
else
    echo -e "  ${RED}Error: emcc not found${NC}"
    exit 1
fi

echo ""
echo -e "${CYAN}Setting up build environment...${NC}"

# Get the directory of this script
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Compile wrapper if it doesn't exist or source is newer
WRAPPER_SOURCE="$SCRIPT_DIR/emcc-wrapper.rs"
WRAPPER_PATH="$SCRIPT_DIR/emcc-wrapper"

if [ ! -f "$WRAPPER_PATH" ] || [ "$WRAPPER_SOURCE" -nt "$WRAPPER_PATH" ]; then
    echo -e "  ${CYAN}Compiling emcc wrapper...${NC}"
    rustc "$WRAPPER_SOURCE" -o "$WRAPPER_PATH"
    if [ $? -ne 0 ]; then
        echo -e "  ${RED}Error: Failed to compile emcc wrapper${NC}"
        exit 1
    fi
    chmod +x "$WRAPPER_PATH"
fi

# Set compiler environment variables for wasm32-unknown-unknown target
export CC_wasm32_unknown_unknown="$WRAPPER_PATH"
export AR_wasm32_unknown_unknown="$(which emar)"
export CFLAGS_wasm32_unknown_unknown="-s WASM=1"

# Configure bindgen to use Emscripten sysroot
if [ -d "$EMSDK_PATH/upstream/emscripten/cache/sysroot" ]; then
    EMSDK_SYSROOT="$EMSDK_PATH/upstream/emscripten/cache/sysroot"
    BINDGEN_ARGS="--sysroot=$EMSDK_SYSROOT -I$EMSDK_SYSROOT/include -I$EMSDK_SYSROOT/include/compat -I$EMSDK_PATH/upstream/emscripten/system/include"
    export BINDGEN_EXTRA_CLANG_ARGS_wasm32_unknown_unknown="$BINDGEN_ARGS"
fi

echo -e "  ${GRAY}CC_wasm32_unknown_unknown = $WRAPPER_PATH (emcc wrapper)${NC}"
echo -e "  ${GRAY}AR_wasm32_unknown_unknown = $(which emar)${NC}"
echo -e "  ${GRAY}BINDGEN_EXTRA_CLANG_ARGS configured for Emscripten sysroot${NC}"

# Build arguments for wasm-pack
BUILD_ARGS=(
    "build"
    "--target" "$TARGET"
    "--no-default-features"
    "--features" "wasm"
)

# Add profile flag (release is default for wasm-pack, so only add if different)
if [ "$BUILD_PROFILE" = "dev" ]; then
    BUILD_ARGS+=("--dev")
elif [ "$BUILD_PROFILE" = "debug" ]; then
    BUILD_ARGS+=("--debug")
fi

echo ""
echo -e "${CYAN}Running wasm-pack...${NC}"
echo -e "${GRAY}Command: wasm-pack ${BUILD_ARGS[*]}${NC}"
echo ""

# Run wasm-pack build
wasm-pack "${BUILD_ARGS[@]}"

if [ $? -ne 0 ]; then
    echo ""
    echo -e "${RED}Build failed!${NC}"
    exit 1
fi

echo ""
echo -e "${CYAN}Running post-build fixes...${NC}"

# Check if fix script exists and run it
if [ -f "$SCRIPT_DIR/fix-wasm-exports.sh" ]; then
    bash "$SCRIPT_DIR/fix-wasm-exports.sh"
elif [ -f "$SCRIPT_DIR/fix-wasm-exports.ps1" ]; then
    # Try to run PowerShell script if no bash version exists
    if command -v pwsh &> /dev/null; then
        pwsh -File "$SCRIPT_DIR/fix-wasm-exports.ps1"
    else
        echo -e "${YELLOW}Warning: fix-wasm-exports script not found for Unix${NC}"
    fi
else
    echo -e "${YELLOW}Warning: fix-wasm-exports script not found${NC}"
fi

if [ $? -ne 0 ]; then
    echo ""
    echo -e "${RED}Post-build fix failed!${NC}"
    exit 1
fi

echo ""
echo -e "${GREEN}Build successful!${NC}"
echo ""
echo -e "${CYAN}Output directory: pkg/${NC}"

# Show generated files
if [ -d "pkg" ]; then
    echo ""
    echo -e "${CYAN}Generated files:${NC}"
    for file in pkg/*; do
        if [ -f "$file" ]; then
            filename=$(basename "$file")
            size=$(du -h "$file" | cut -f1)
            printf "  ${GRAY}%-40s %s${NC}\n" "$filename" "$size"
        fi
    done
fi

echo ""
echo -e "${GREEN}You can now use the WebAssembly package in your JavaScript project!${NC}"
echo ""
echo -e "${YELLOW}Example:${NC}"
echo -e "  ${NC}import init, { SockudoClient } from './pkg/sockudo_client.js';${NC}"
echo -e "  ${NC}await init();${NC}"
echo -e "  ${NC}const client = new SockudoClient('YOUR_APP_KEY');${NC}"
echo ""
echo -e "${CYAN}Try the browser example:${NC}"
echo -e "  ${NC}cd examples/browser${NC}"
echo -e "  ${NC}./serve.sh              # Linux/Mac${NC}"
echo -e "  ${NC}pwsh serve.ps1          # Windows${NC}"
echo -e "  ${NC}Then open: http://localhost:8000/examples/browser/${NC}"
echo ""
echo -e "${CYAN}To build again:${NC}"
echo -e "  ${NC}./build-wasm.sh         # Release build (default)${NC}"
echo -e "  ${NC}./build-wasm.sh --dev   # Development build${NC}"
echo -e "  ${NC}./build-wasm.sh --debug # Debug build${NC}"
echo ""
