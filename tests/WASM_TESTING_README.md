# WASM Delta Compression Test Suite

Comprehensive test suite for validating the delta compression functionality in the WASM build of sockudo-client.

## 📋 Test Coverage

### Test Categories

1. **Delta Options - Basic** (4 tests)
   - Default delta options creation
   - Enable/disable delta compression
   - Toggle debug mode
   - Set max messages per key

2. **Delta Options - Algorithms** (7 tests)
   - Set single algorithm (fossil)
   - Set single algorithm (xdelta3)
   - Set multiple algorithms
   - Set algorithms with spaces
   - Handle invalid algorithms gracefully
   - Handle mixed valid/invalid algorithms
   - Case insensitive algorithm names

3. **WasmOptions Integration** (4 tests)
   - Create WasmOptions with delta compression
   - Enable delta compression convenience method
   - Set disabled delta compression
   - Full configuration with all options

4. **Configuration Variations** (6 tests)
   - Fossil only configuration
   - Xdelta3 only configuration
   - Both algorithms enabled
   - Debug mode enabled
   - Minimal cache size (1)
   - Large cache size (100)

5. **Edge Cases** (6 tests)
   - Zero max messages (edge case)
   - Very large max messages
   - Empty algorithm string
   - Only commas in algorithms
   - Duplicate algorithms
   - Replace delta configuration

6. **sockudo-js API Compatibility** (4 tests)
   - Default sockudo-js configuration
   - Custom algorithm preference
   - Debug mode like sockudo-js
   - Disabled like sockudo-js

7. **Real-world Scenarios** (4 tests)
   - Production configuration
   - Development configuration
   - Minimal configuration
   - High-throughput configuration

**Total: 35+ comprehensive tests**

## 🚀 Running the Tests

### Method 1: Browser Test Runner (Recommended)

1. Build the WASM package:
   ```powershell
   .\build-wasm.ps1
   ```

2. Serve the test runner:
   ```bash
   # Using Python
   python -m http.server 8000
   
   # Or using Node.js
   npx serve
   ```

3. Open in browser:
   ```
   http://localhost:8000/tests/wasm_test_runner.html
   ```

4. Click "▶️ Run All Tests" button

### Method 2: wasm-pack test

Run the Rust test file:

```bash
wasm-pack test --headless --chrome tests/wasm_delta_compression.rs
wasm-pack test --headless --firefox tests/wasm_delta_compression.rs
```

## 📊 Test Output

The HTML test runner provides:

### Statistics Dashboard
- **Passed Count**: Number of successful tests
- **Failed Count**: Number of failed tests
- **Pending Count**: Tests not yet run
- **Total Time**: Cumulative execution time
- **Success Rate**: Percentage of passing tests

### Visual Indicators
- ✅ **Green** - Test passed
- ❌ **Red** - Test failed
- ⏸️ **Orange** - Test pending
- Progress bar showing test execution
- Error details for failed tests

### Filters
- **All**: Show all tests
- **Passed**: Show only passed tests
- **Failed**: Show only failed tests

## 🎯 What's Being Tested

### API Correctness
- ✅ All configuration options work correctly
- ✅ Default values match sockudo-js
- ✅ Type conversions are accurate
- ✅ Getters and setters function properly

### Edge Cases
- ✅ Invalid input handling
- ✅ Boundary values (0, MAX_INT)
- ✅ Empty strings and special characters
- ✅ Configuration replacement

### Integration
- ✅ WasmOptions + WasmDeltaOptions integration
- ✅ Conversion to internal Rust types
- ✅ No panics or crashes

### Compatibility
- ✅ API matches sockudo-js exactly
- ✅ Same configuration patterns work
- ✅ Same behavior for edge cases

## 🔧 Example Test Patterns

### Basic Configuration Test
```javascript
const opts = new WasmOptions('test-app-key');
const deltaOpts = new WasmDeltaOptions();
deltaOpts.enabled = true;
deltaOpts.setAlgorithms('fossil,xdelta3');
opts.setDeltaCompression(deltaOpts);
```

### Convenience Method Test
```javascript
const opts = new WasmOptions('test-app-key');
opts.enableDeltaCompression(); // Uses defaults
```

### Full Configuration Test
```javascript
const opts = new WasmOptions('test-app-key');
opts.cluster = 'mt1';
opts.ws_host = 'localhost';
opts.ws_port = 6001;
opts.use_tls = false;

const deltaOpts = new WasmDeltaOptions();
deltaOpts.enabled = true;
deltaOpts.debug = true;
deltaOpts.max_messages_per_key = 15;
deltaOpts.setAlgorithms('fossil,xdelta3');

opts.setDeltaCompression(deltaOpts);
```

## 📝 Adding New Tests

Add tests to `wasm_delta_compression.rs`:

```rust
#[wasm_bindgen_test]
fn test_my_new_feature() {
    let mut opts = WasmDeltaOptions::new();
    // Your test code here
    assert_eq!(opts.enabled(), true);
}
```

Or add to the HTML test runner in `testCategories`:

```javascript
'My Test Category': [
    {
        name: 'My test description',
        fn: () => {
            const opts = new WasmDeltaOptions();
            assert(opts.enabled === true, 'Should be enabled');
        }
    }
]
```

## ✅ Expected Results

All tests should pass (100% success rate) with:
- No panics or crashes
- No JavaScript errors
- Proper type conversions
- Correct default values
- Graceful error handling

## 🐛 Debugging Failed Tests

1. **Check Browser Console**: Look for JavaScript errors or WASM panics
2. **Review Error Message**: The test runner shows specific assertion failures
3. **Test Duration**: Unusually long tests may indicate infinite loops
4. **Isolate Test**: Run specific test category using filters

## 📚 Related Documentation

- [Delta Compression Documentation](../sockudo-js/DELTA_COMPRESSION.md)
- [WASM Build Guide](../WASM_BUILD.md)
- [API Documentation](../README.md)

## 🎉 Success Criteria

✅ All 35+ tests pass  
✅ 100% success rate  
✅ No console errors  
✅ Total execution time < 500ms  
✅ Works in Chrome, Firefox, Safari, Edge  

## 📧 Support

If tests fail or you encounter issues:
1. Check that WASM build completed successfully
2. Verify browser supports WebAssembly
3. Review error messages in test output
4. Check browser console for additional details
