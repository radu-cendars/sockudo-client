#!/bin/bash
# Test script for WebAssembly with Emscripten support

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
GRAY='\033[0;90m'
NC='\033[0m' # No Color

# Default values
HEADLESS=true
BROWSER="chrome"

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --firefox)
            BROWSER="firefox"
            shift
            ;;
        --chrome)
            BROWSER="chrome"
            shift
            ;;
        --safari)
            BROWSER="safari"
            shift
            ;;
        --headed)
            HEADLESS=false
            shift
            ;;
        --headless)
            HEADLESS=true
            shift
            ;;
        -h|--help)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --firefox          Run tests in Firefox"
            echo "  --chrome           Run tests in Chrome (default)"
            echo "  --safari           Run tests in Safari"
            echo "  --headed           Run tests in headed mode (show browser)"
            echo "  --headless         Run tests in headless mode (default)"
            echo "  -h, --help         Show this help message"
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            exit 1
            ;;
    esac
done

echo -e "${GREEN}Running Sockudo WebAssembly Tests...${NC}"
echo ""

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
    echo -e "${GREEN}✓ Emscripten environment loaded${NC}"
else
    echo -e "${YELLOW}Warning: Emscripten not found${NC}"
    echo -e "${YELLOW}Tests will run but C code compilation (xdelta3) may fail${NC}"
    echo ""
    echo -e "${CYAN}To install Emscripten:${NC}"
    echo -e "${GRAY}  git clone https://github.com/emscripten-core/emsdk.git ~/emsdk${NC}"
    echo -e "${GRAY}  cd ~/emsdk${NC}"
    echo -e "${GRAY}  ./emsdk install latest${NC}"
    echo -e "${GRAY}  ./emsdk activate latest${NC}"
    echo ""
fi

# Verify Emscripten tools
if command -v emcc &> /dev/null; then
    EMCC_VERSION=$(emcc --version 2>&1 | head -n 1 | cut -d' ' -f1-2)
    echo -e "${GREEN}✓ emcc available:${NC} ${GRAY}$EMCC_VERSION${NC}"
else
    echo -e "${YELLOW}⚠ emcc not found - xdelta3 compilation may fail${NC}"
fi

echo ""
echo -e "${CYAN}Setting up build environment...${NC}"

# Get the directory of this script
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Compile wrapper if needed
WRAPPER_SOURCE="$SCRIPT_DIR/emcc-wrapper.rs"
WRAPPER_PATH="$SCRIPT_DIR/emcc-wrapper"

if [ -f "$WRAPPER_SOURCE" ]; then
    if [ ! -f "$WRAPPER_PATH" ] || [ "$WRAPPER_SOURCE" -nt "$WRAPPER_PATH" ]; then
        echo -e "${CYAN}Compiling emcc wrapper...${NC}"
        rustc "$WRAPPER_SOURCE" -o "$WRAPPER_PATH"
        if [ $? -ne 0 ]; then
            echo -e "${RED}Error: Failed to compile emcc wrapper${NC}"
            exit 1
        fi
        chmod +x "$WRAPPER_PATH"
        echo -e "${GREEN}✓ emcc wrapper compiled${NC}"
    fi
fi

# Set compiler environment variables for wasm32-unknown-unknown target
# Use Emscripten's clang which produces object files compatible with rust-lld
if [ -n "$EMSDK_PATH" ] && [ -d "$EMSDK_PATH/upstream/emscripten" ]; then
    EMSCRIPTEN_CLANG="$EMSDK_PATH/upstream/bin/clang"
    EMSCRIPTEN_AR="$EMSDK_PATH/upstream/bin/llvm-ar"
    EMSDK_SYSROOT="$EMSDK_PATH/upstream/emscripten/cache/sysroot"

    if [ -f "$EMSCRIPTEN_CLANG" ]; then
        export CC_wasm32_unknown_unknown="$EMSCRIPTEN_CLANG"
        echo -e "${GREEN}✓ CC_wasm32_unknown_unknown = Emscripten clang${NC}"

        # Set CFLAGS for wasm32 target with Emscripten includes
        CFLAGS="--target=wasm32-unknown-unknown"
        CFLAGS="$CFLAGS -nostdinc"
        CFLAGS="$CFLAGS -isystem $EMSDK_SYSROOT/include"
        CFLAGS="$CFLAGS -isystem $EMSDK_SYSROOT/include/compat"
        CFLAGS="$CFLAGS -D__EMSCRIPTEN__"
        CFLAGS="$CFLAGS -Wno-unused-parameter"
        export CFLAGS_wasm32_unknown_unknown="$CFLAGS"
        echo -e "${GREEN}✓ CFLAGS configured for wasm32${NC}"
    fi

    if [ -f "$EMSCRIPTEN_AR" ]; then
        export AR_wasm32_unknown_unknown="$EMSCRIPTEN_AR"
        echo -e "${GREEN}✓ AR_wasm32_unknown_unknown = llvm-ar${NC}"
    fi

    # Configure bindgen to use Emscripten sysroot
    BINDGEN_ARGS="--sysroot=$EMSDK_SYSROOT"
    BINDGEN_ARGS="$BINDGEN_ARGS -I$EMSDK_SYSROOT/include"
    BINDGEN_ARGS="$BINDGEN_ARGS -I$EMSDK_SYSROOT/include/compat"
    export BINDGEN_EXTRA_CLANG_ARGS_wasm32_unknown_unknown="$BINDGEN_ARGS"
    echo -e "${GREEN}✓ BINDGEN configured for Emscripten sysroot${NC}"
else
    echo -e "${RED}Error: Emscripten upstream directory not found${NC}"
    exit 1
fi

echo ""
echo -e "${CYAN}Test Configuration:${NC}"
echo -e "  Browser: ${GRAY}$BROWSER${NC}"
echo -e "  Mode: ${GRAY}$([ "$HEADLESS" = true ] && echo "headless" || echo "headed")${NC}"
echo ""

# Build test arguments
TEST_ARGS=(
    "test"
    "--$BROWSER"
)

if [ "$HEADLESS" = true ]; then
    TEST_ARGS+=("--headless")
fi

TEST_ARGS+=(
    "."
    "--test" "wasm_comprehensive_tests"
    "--no-default-features"
    "--features" "wasm"
)

echo -e "${CYAN}Running wasm-pack test...${NC}"
echo -e "${GRAY}Command: wasm-pack ${TEST_ARGS[*]}${NC}"
echo ""

# Run wasm-pack test
wasm-pack "${TEST_ARGS[@]}"

TEST_RESULT=$?

echo ""
if [ $TEST_RESULT -eq 0 ]; then
    echo -e "${GREEN}════════════════════════════════════════${NC}"
    echo -e "${GREEN}  ✓ All tests passed!${NC}"
    echo -e "${GREEN}════════════════════════════════════════${NC}"
else
    echo -e "${RED}════════════════════════════════════════${NC}"
    echo -e "${RED}  ✗ Tests failed!${NC}"
    echo -e "${RED}════════════════════════════════════════${NC}"
fi

exit $TEST_RESULT
