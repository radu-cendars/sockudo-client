# 🔌 Integration Tests for Delta Compression

Comprehensive integration tests that connect to a real sockudo server instance and validate delta compression functionality end-to-end.

## 📋 Test Coverage

### Test Categories (25+ tests)

#### 1. **Basic Connection Tests** (4 tests)
- ✅ Connect with delta compression enabled
- ✅ Connect with delta compression disabled
- ✅ Connect with Fossil algorithm only
- ✅ Connect with Xdelta3 algorithm only

#### 2. **Subscription and Message Tests** (2 tests)
- ✅ Subscribe and receive messages with delta compression
- ✅ Multiple simultaneous subscriptions with delta compression

#### 3. **Callback Tests** (2 tests)
- ✅ Stats callback invocation and data capture
- ✅ Error callback invocation on failures

#### 4. **Delta Message Processing** (2 tests)
- ✅ Delta message decoding verification
- ✅ Bandwidth savings calculation accuracy

#### 5. **Conflation Key Tests** (1 test)
- ✅ Proper handling of conflation keys and grouping

#### 6. **Cache Size Tests** (2 tests)
- ✅ Small cache size (2 messages)
- ✅ Large cache size (100 messages)

#### 7. **Stats Management** (1 test)
- ✅ Stats reset functionality

#### 8. **Reconnection Tests** (1 test)
- ✅ Delta compression works after reconnection

#### 9. **Performance Tests** (1 test)
- ✅ High message volume handling (30+ seconds)

**Total: 16 comprehensive integration tests**

---

## 🚀 Prerequisites

### 1. Running Sockudo Server

You need a sockudo server instance running. You can either:

#### Option A: Use Local Sockudo Server
```bash
# Start your sockudo server
cd /path/to/sockudo-server
npm start
# or
./start-server.sh
```

#### Option B: Use Docker
```bash
docker run -p 6001:6001 sockudo/server
```

#### Option C: Use Remote Server
Configure the test script to point to your remote server

### 2. Server Configuration

Ensure your server has:
- ✅ Delta compression enabled
- ✅ WebSocket endpoint accessible
- ✅ Test app key configured (`app-key` by default)

---

## 🎯 Running the Tests

### Method 1: PowerShell Script (Recommended)

```powershell
# Run all tests with default settings (localhost:6001)
.\run-integration-tests.ps1

# Specify custom server
.\run-integration-tests.ps1 -ServerHost "your-server.com" -ServerPort 443 -UseTLS

# Run specific test
.\run-integration-tests.ps1 -TestFilter "test_connect_with_delta_compression_enabled"

# Verbose output (see println! messages)
.\run-integration-tests.ps1 -Verbose

# All options
.\run-integration-tests.ps1 `
    -ServerHost "localhost" `
    -ServerPort 6001 `
    -AppKey "your-app-key" `
    -UseTLS `
    -TestFilter "test_bandwidth" `
    -Verbose
```

### Method 2: Direct Cargo Command

```bash
# Run all integration tests
cargo test --test integration_delta_compression -- --ignored

# Run specific test
cargo test --test integration_delta_compression test_connect_with_delta_compression_enabled -- --ignored

# With output
cargo test --test integration_delta_compression -- --ignored --nocapture

# Run only tests matching pattern
cargo test --test integration_delta_compression bandwidth -- --ignored --nocapture
```

### Method 3: VS Code / IDE

Add to your `.vscode/settings.json`:
```json
{
    "rust-analyzer.cargo.allFeatures": true,
    "rust-analyzer.runnables.extraArgs": [
        "--test",
        "integration_delta_compression",
        "--",
        "--ignored"
    ]
}
```

Then click "Run Test" in your IDE.

---

## 📊 Expected Test Output

### Successful Run
```
Running integration tests...
========================================

test test_connect_with_delta_compression_enabled ... ok
test test_subscribe_and_receive_with_delta_compression ... ok
test test_stats_callback_invocation ... ok
test test_delta_message_decoding ... ok
  Total messages: 50
  Delta messages: 45
  Full messages: 5
  Bandwidth saved: 67.32%
  Errors: 0
✅ Successfully decoded delta messages!
test test_bandwidth_savings_calculation ... ok
=== Bandwidth Savings Analysis ===
Total messages: 120
Delta messages: 110
Full messages: 10
Bandwidth saved: 15234 bytes
Bandwidth saved: 71.25%
✅ Delta compression is working and saving bandwidth!

test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured
```

### With Verbose Output
```
Received event: PusherEvent { event: "test-event", channel: Some("test-delta-channel"), data: Some("{...}") }
Delta compression stats: DeltaStats { total_messages: 1, delta_messages: 0, full_messages: 1, ... }
Stats callback invoked: DeltaStats { total_messages: 5, delta_messages: 4, ... }
Decoded delta event: PusherEvent { ... }
```

---

## 🔧 Configuration

### Environment Variables

You can set these instead of using script parameters:

```bash
export TEST_SERVER_HOST="localhost"
export TEST_SERVER_PORT="6001"
export TEST_APP_KEY="app-key"
export TEST_USE_TLS="false"
```

### Test File Constants

Edit `tests/integration_delta_compression.rs`:

```rust
const TEST_APP_KEY: &str = "your-app-key";
const TEST_HOST: &str = "your-server.com";
const TEST_PORT: u16 = 443;
const TEST_USE_TLS: bool = true;
```

---

## 🧪 Individual Test Descriptions

### Basic Connection Tests

#### `test_connect_with_delta_compression_enabled`
- Connects with full delta compression
- Verifies connection established
- Checks delta compression is active
- Validates no errors occurred

#### `test_connect_with_delta_compression_disabled`
- Connects with delta disabled
- Verifies normal operation
- Ensures no delta messages processed

#### `test_connect_with_fossil_only`
- Tests Fossil algorithm preference
- Validates server accepts algorithm choice

#### `test_connect_with_xdelta3_only`
- Tests Xdelta3 algorithm preference
- Validates server accepts algorithm choice

### Subscription Tests

#### `test_subscribe_and_receive_with_delta_compression`
- Subscribes to a channel
- Receives messages
- Validates delta decoding
- Checks for errors

#### `test_multiple_subscriptions_with_delta_compression`
- Tests 3+ simultaneous channels
- Validates per-channel delta state
- Ensures no cross-channel interference

### Callback Tests

#### `test_stats_callback_invocation`
- Verifies callback is invoked
- Captures stats data
- Validates stats accuracy

#### `test_error_callback_invocation`
- Tests error reporting
- Captures error messages
- Validates error handling

### Processing Tests

#### `test_delta_message_decoding`
- Receives delta-compressed messages
- Decodes and validates content
- Tracks decoding success rate
- Measures bandwidth savings

#### `test_bandwidth_savings_calculation`
- Accumulates message statistics
- Calculates savings percentage
- Validates compression effectiveness
- Reports detailed metrics

### Conflation Tests

#### `test_conflation_key_handling`
- Tests channels with conflation keys
- Validates proper grouping
- Checks per-key caching
- Reports conflation statistics

### Cache Tests

#### `test_small_cache_size`
- Tests with 2-message cache
- Validates FIFO eviction
- Ensures no errors with small cache

#### `test_large_cache_size`
- Tests with 100-message cache
- Validates memory handling
- Ensures performance is acceptable

### Management Tests

#### `test_stats_reset`
- Gets initial statistics
- Resets counters
- Validates all counters are zero
- Confirms clean reset

### Reconnection Tests

#### `test_delta_compression_after_reconnection`
- Connects initially
- Disconnects cleanly
- Reconnects
- Validates delta still works
- Checks state preservation

### Performance Tests

#### `test_high_message_volume`
- Receives 100+ messages
- Measures processing speed
- Validates no errors under load
- Reports performance metrics

---

## 📈 Success Criteria

### Per-Test Criteria
- ✅ Connection establishes successfully
- ✅ Messages are received
- ✅ Delta decoding succeeds
- ✅ No decoding errors
- ✅ Bandwidth savings > 0% (when delta active)
- ✅ Callbacks are invoked
- ✅ Stats are accurate

### Overall Criteria
- ✅ All 16 tests pass
- ✅ 0 errors in delta decoding
- ✅ Bandwidth savings measured
- ✅ No panics or crashes
- ✅ Clean disconnection

---

## 🐛 Troubleshooting

### Issue: "Connection refused"
```
Error: Connection refused (os error 111)
```

**Solution:**
1. Verify server is running: `netstat -an | grep 6001`
2. Check firewall rules
3. Verify server address and port
4. Ensure server is bound to correct interface

### Issue: "Tests timeout"
```
test test_subscribe_and_receive ... timeout
```

**Solution:**
1. Increase timeout in test (edit `sleep(Duration::from_secs(X))`)
2. Check server is sending messages
3. Verify channel names match server configuration
4. Check server logs for errors

### Issue: "No delta messages received"
```
Delta messages: 0
Full messages: 50
```

**Solution:**
1. Verify server has delta compression enabled
2. Check server supports chosen algorithms
3. Ensure server sent cache sync message
4. Enable debug mode to see protocol messages

### Issue: "Delta decoding errors"
```
Errors: 5
```

**Solution:**
1. Check algorithm compatibility
2. Verify base64 encoding is correct
3. Enable error callbacks to see details
4. Check server delta implementation

### Issue: "Stats callback not called"
```
Stats callback was called: false
```

**Solution:**
1. Ensure messages are being sent
2. Check callback is properly set
3. Wait longer for messages
4. Verify server is configured correctly

---

## 📊 Performance Benchmarks

### Expected Performance

| Metric | Target | Notes |
|--------|--------|-------|
| Connection Time | <2s | Initial connection |
| Message Decode | <5ms | Per delta message |
| Bandwidth Savings | 50-80% | With similar messages |
| Error Rate | 0% | No decode failures |
| High Volume | 1000+ msg/s | Sustained throughput |

### Measuring Your Results

Run the performance test:
```powershell
.\run-integration-tests.ps1 -TestFilter "test_high_message_volume" -Verbose
```

Look for output:
```
=== High Volume Test Results ===
Total messages processed: 1250
Delta messages: 1200
Full messages: 50
Bandwidth saved: 68.45%
Errors: 0
```

---

## 🔬 Advanced Testing

### Custom Test Scenarios

Create a new test in `tests/integration_delta_compression.rs`:

```rust
#[tokio::test]
#[ignore]
async fn test_my_custom_scenario() {
    let options = create_test_options()
        .delta_compression(DeltaOptions {
            enabled: true,
            algorithms: vec![DeltaAlgorithm::Fossil],
            debug: true,
            max_messages_per_key: 5,
            on_stats: Some(Arc::new(|stats| {
                println!("Custom stats: {:?}", stats);
            })),
            on_error: None,
        });

    let client = SockudoClient::new(options.into())
        .expect("Failed to create client");

    sleep(Duration::from_secs(2)).await;

    // Your test logic here

    client.disconnect().await;
}
```

### Stress Testing

Run tests repeatedly:
```powershell
for ($i=1; $i -le 10; $i++) {
    Write-Host "Run $i"
    .\run-integration-tests.ps1
}
```

### Continuous Integration

Add to CI pipeline:
```yaml
- name: Start sockudo server
  run: docker run -d -p 6001:6001 sockudo/server

- name: Wait for server
  run: sleep 5

- name: Run integration tests
  run: cargo test --test integration_delta_compression -- --ignored

- name: Stop server
  run: docker stop $(docker ps -q)
```

---

## 📝 Test Maintenance

### Adding New Tests

1. Add test function to `tests/integration_delta_compression.rs`
2. Mark with `#[tokio::test]` and `#[ignore]`
3. Use `create_test_options()` helper
4. Add assertions and logging
5. Update this README

### Updating Server Configuration

If server config changes:
1. Update constants in test file
2. Update script parameters
3. Update documentation
4. Re-run full test suite

---

## ✅ Checklist Before Running

- [ ] Sockudo server is running
- [ ] Server has delta compression enabled
- [ ] Correct app key is configured
- [ ] Network connectivity is good
- [ ] Firewall allows WebSocket connections
- [ ] Test configuration matches server

---

## 🎉 Success Indicators

When tests are working correctly, you should see:

```
✅ All tests passed
✅ 0 delta decoding errors
✅ Bandwidth savings: 50-80%
✅ Callbacks invoked successfully
✅ Reconnection works
✅ High volume handled without errors
✅ Stats accurately tracked
```

---

## 📚 Related Documentation

- [Delta Compression Overview](../sockudo-js/DELTA_COMPRESSION.md)
- [WASM Tests](./WASM_TESTING_README.md)
- [API Documentation](../README.md)
- [Server Setup](../SERVER_SETUP.md)

---

## 📧 Support

If integration tests fail:
1. Check server logs for errors
2. Enable verbose output (`-Verbose`)
3. Run single test with `--nocapture`
4. Verify server configuration
5. Check network connectivity
6. Review error callback messages

---

**Last Updated:** 2025-01-02  
**Test Suite Version:** 1.0  
**Total Tests:** 16 integration tests
