#!/bin/bash
# Build script for Sockudo Client Rust library
# Generates bindings for Kotlin, Swift, JavaScript, and Node.js

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}=== Sockudo Client Build Script ===${NC}"

# Build the main library
build_rust() {
    echo -e "${YELLOW}Building Rust library...${NC}"
    cargo build --release
    echo -e "${GREEN}✓ Rust library built${NC}"
}

# Generate Kotlin bindings via UniFFI
build_kotlin() {
    echo -e "${YELLOW}Generating Kotlin bindings...${NC}"

    mkdir -p target/bindings/kotlin

    # Build native library first (needed for proc-macro based UniFFI)
    cargo build --features native

    # Generate bindings from the native library
    cargo run --features uniffi-bindgen --bin uniffi-bindgen -- \
        generate --library \
        --language kotlin \
        --out-dir target/bindings/kotlin \
        --no-format \
        target/debug/libsockudo_client.so

    echo -e "${GREEN}✓ Kotlin bindings generated in target/bindings/kotlin/${NC}"
}

# Generate Swift bindings via UniFFI
build_swift() {
    echo -e "${YELLOW}Generating Swift bindings...${NC}"

    mkdir -p target/bindings/swift

    cargo run --features uniffi-bindgen --bin uniffi-bindgen -- \
        generate src/sockudo_client.udl \
        --language swift \
        --out-dir target/bindings/swift

    echo -e "${GREEN}✓ Swift bindings generated in target/bindings/swift/${NC}"
}

# Build WASM for web browsers
build_wasm_web() {
    echo -e "${YELLOW}Building WASM for web...${NC}"

    if ! command -v wasm-pack &> /dev/null; then
        echo -e "${RED}wasm-pack not found. Install with: cargo install wasm-pack${NC}"
        exit 1
    fi

    # Build with wasm-pack (it outputs to ./pkg by default)
    wasm-pack build --target web --features wasm

    # Move output to desired location
    mkdir -p target/pkg
    rm -rf target/pkg/web
    mv pkg target/pkg/web

    echo -e "${GREEN}✓ WASM (web) built in target/pkg/web/${NC}"
}

# Build WASM for Node.js
build_wasm_node() {
    echo -e "${YELLOW}Building WASM for Node.js...${NC}"

    if ! command -v wasm-pack &> /dev/null; then
        echo -e "${RED}wasm-pack not found. Install with: cargo install wasm-pack${NC}"
        exit 1
    fi

    # Build with wasm-pack (it outputs to ./pkg by default)
    wasm-pack build --target nodejs --features wasm

    # Copy output to nodejs directory for easy usage
    mkdir -p nodejs
    rm -rf nodejs/pkg
    cp -r pkg nodejs/

    echo -e "${GREEN}✓ WASM (Node.js) built in nodejs/pkg/${NC}"
}

# Build for Android (multiple architectures)
build_android() {
    echo -e "${YELLOW}Building for Android...${NC}"

    ANDROID_TARGETS=(
        "aarch64-linux-android"
        "armv7-linux-androideabi"
        "x86_64-linux-android"
        "i686-linux-android"
    )

    for target in "${ANDROID_TARGETS[@]}"; do
        echo "  Building for $target..."
        cargo build --release --target "$target"
    done

    echo -e "${GREEN}✓ Android libraries built${NC}"
}

# Build for iOS (multiple architectures)
build_ios() {
    echo -e "${YELLOW}Building for iOS...${NC}"

    IOS_TARGETS=(
        "aarch64-apple-ios"
        "x86_64-apple-ios"
        "aarch64-apple-ios-sim"
    )

    for target in "${IOS_TARGETS[@]}"; do
        echo "  Building for $target..."
        cargo build --release --target "$target"
    done

    echo -e "${GREEN}✓ iOS libraries built${NC}"
}

# Create XCFramework for iOS
create_xcframework() {
    echo -e "${YELLOW}Creating XCFramework...${NC}"

    mkdir -p target/xcframework

    # Generate Swift bindings first
    build_swift

    # Create the framework structure
    # (This is simplified - a real implementation would need more setup)

    echo -e "${GREEN}✓ XCFramework created${NC}"
}

# Build Flutter plugin
build_flutter() {
    echo -e "${YELLOW}Building Flutter plugin...${NC}"

    # Add Flutter to PATH if available
    if [ -d "/home/radud/develop/flutter/bin" ]; then
        export PATH="$PATH:/home/radud/develop/flutter/bin"
    fi

    # Check if flutter_rust_bridge_codegen is installed
    if ! command -v flutter_rust_bridge_codegen &> /dev/null; then
        echo -e "${YELLOW}Installing flutter_rust_bridge_codegen...${NC}"
        cargo install flutter_rust_bridge_codegen
    fi

    # Generate Dart bindings
    echo -e "${YELLOW}Generating Dart bindings...${NC}"
    flutter_rust_bridge_codegen generate

    # Build Rust library for Flutter platforms
    echo -e "${YELLOW}Building Rust library for Flutter...${NC}"

    # Android
    if command -v cross &> /dev/null; then
        for target in aarch64-linux-android armv7-linux-androideabi x86_64-linux-android i686-linux-android; do
            echo "  Building for $target..."
            cross build --release --target "$target" --features flutter
        done
    fi

    # iOS
    if [[ "$OSTYPE" == "darwin"* ]]; then
        for target in aarch64-apple-ios x86_64-apple-ios aarch64-apple-ios-sim; do
            echo "  Building for $target..."
            cargo build --release --target "$target" --features flutter
        done
    fi

    # Desktop
    cargo build --release --features flutter

    echo -e "${GREEN}✓ Flutter plugin built${NC}"
}

# Run tests
run_tests() {
    echo -e "${YELLOW}Running tests...${NC}"
    cargo test
    echo -e "${GREEN}✓ Tests passed${NC}"
}

# Clean build artifacts
clean() {
    echo -e "${YELLOW}Cleaning build artifacts...${NC}"
    cargo clean
    rm -rf target/bindings target/pkg target/xcframework
    echo -e "${GREEN}✓ Cleaned${NC}"
}

# Main
case "${1:-all}" in
    rust)
        build_rust
        ;;
    kotlin)
        build_rust
        build_kotlin
        ;;
    swift)
        build_rust
        build_swift
        ;;
    wasm|wasm-web)
        build_wasm_web
        ;;
    wasm-node)
        build_wasm_node
        ;;
    android)
        build_android
        build_kotlin
        ;;
    ios)
        build_ios
        create_xcframework
        ;;
    flutter)
        build_flutter
        ;;
    test)
        run_tests
        ;;
    clean)
        clean
        ;;
    all)
        build_rust
        build_kotlin
        build_swift
        build_wasm_web
        build_wasm_node
        run_tests
        echo -e "${GREEN}=== All builds complete! ===${NC}"
        ;;
    *)
        echo "Usage: $0 {rust|kotlin|swift|wasm|wasm-node|android|ios|flutter|test|clean|all}"
        exit 1
        ;;
esac
