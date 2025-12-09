# Running WASM Tests

## Prerequisites

1. Install Emscripten (required for xdelta3 C code compilation):
   ```bash
   git clone https://github.com/emscripten-core/emsdk.git ~/emsdk
   cd ~/emsdk
   ./emsdk install latest
   ./emsdk activate latest
   ```

2. Install wasm-pack:
   ```bash
   cargo install wasm-pack
   ```

## Running Tests

Use the provided test script which sets up the Emscripten environment automatically:

```bash
# Run tests in headless Chrome (default)
./test-wasm.sh --headless

# Run tests in headed mode (shows browser window)
./test-wasm.sh --headed

# Run tests in Firefox
./test-wasm.sh --firefox --headless

# Run tests in Safari
./test-wasm.sh --safari --headless
```

## Manual Testing

If you need to run tests manually, make sure to set up the Emscripten environment first:

```bash
# Source Emscripten environment
source ~/emsdk/emsdk_env.sh

# Set compiler environment variables
export CC_wasm32_unknown_unknown="./emcc-wrapper"
export AR_wasm32_unknown_unknown="emar"
export CFLAGS_wasm32_unknown_unknown="-s WASM=1"

# Set bindgen flags
export BINDGEN_EXTRA_CLANG_ARGS_wasm32_unknown_unknown="--sysroot=$EMSDK/upstream/emscripten/cache/sysroot -I$EMSDK/upstream/emscripten/cache/sysroot/include"

# Run wasm-pack test
wasm-pack test --chrome --headless . --test wasm_comprehensive_tests --no-default-features --features wasm
```

## Troubleshooting

### xdelta3 compilation fails

Make sure Emscripten is properly installed and activated:
```bash
which emcc  # Should output the path to emcc
emcc --version  # Should show Emscripten version
```

### Tests don't run

The WASM tests are in `tests/wasm_comprehensive_tests.rs` and are only compiled when the target is `wasm32-unknown-unknown`.
