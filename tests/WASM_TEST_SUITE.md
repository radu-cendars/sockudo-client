# Sockudo WASM Test Suite

Comprehensive test suite for the Sockudo WebAssembly client library.

## Test Files

### 1. Rust Tests (`tests/wasm_comprehensive_tests.rs`)

Automated Rust tests that run in the browser using `wasm-bindgen-test`.

**Tests included:**
- ✅ Client creation and initialization
- ✅ Client with cluster configuration
- ✅ Delta compression options (enable, disable, algorithms)
- ✅ Filter operations (eq, neq, gt, lt, gte, lte, in, exists, and, or)
- ✅ Complex nested filters
- ✅ Channel subscription (public channels)
- ✅ Channel subscription with filters
- ✅ Multiple channel subscriptions
- ✅ Channel resubscription handling
- ✅ Channel unsubscription
- ✅ Private channel subscription
- ✅ Presence channel subscription
- ✅ Encrypted channel subscription
- ✅ Event sending
- ✅ Delta compression stats
- ✅ Unbind operations

**Running the tests:**
```bash
# Install wasm-pack if you haven't
cargo install wasm-pack

# Run tests in browser
wasm-pack test --headless --firefox

# Or in Chrome
wasm-pack test --headless --chrome

# With debug output
wasm-pack test --headless --firefox -- --nocapture
```

### 2. Interactive Browser Tests

#### a. Comprehensive Test Suite (`tests/interactive/public/comprehensive-test.html`)

Full-featured interactive test interface for manual and automated testing.

**Features:**
- 🔌 Connection management (connect/disconnect)
- 📡 Channel subscription with filters
- 🗜️ Delta compression testing with live stats
- 🔀 Conflation key testing
- 🔍 Publish filtering (all filter types)
- 🔒 Private and presence channel tests
- 📊 Real-time statistics and metrics

**Test Categories:**

1. **Configuration**
   - Set app key, WebSocket host, port
   - Enable/disable TLS
   - Enable/disable delta compression
   - Configure debug logging

2. **Basic Connection Tests**
   - Client initialization
   - Connection establishment
   - Reconnection handling
   - Socket ID verification

3. **Channel Subscription Tests**
   - Public channel subscription
   - Private channel subscription
   - Presence channel subscription
   - Multiple channel subscriptions
   - Subscription with filters

4. **Delta Compression Tests**
   - Enable delta compression
   - Test different algorithms (Fossil, Xdelta3)
   - Monitor compression ratio
   - Track bytes saved
   - View compression statistics

5. **Conflation Key Tests**
   - Send messages with conflation keys
   - Verify message grouping
   - Test deduplication

6. **Publish Filtering Tests**
   - Equality filters (`eq`, `neq`)
   - Comparison filters (`gt`, `lt`, `gte`, `lte`)
   - Set filters (`in`, `notIn`)
   - Existence filters (`exists`, `notExists`)
   - Complex filters (`and`, `or`)
   - Nested filter combinations

7. **Private Channel Tests**
   - Configure auth endpoint
   - Subscribe to private channels
   - Subscribe to presence channels
   - Handle authentication

**Accessing the test:**
```bash
cd examples/browser
./serve.sh  # or serve.ps1 on Windows
```
Then navigate to: `http://localhost:8000/tests/interactive/public/comprehensive-test.html`

#### b. Event Binding Test Suite (`tests/interactive/public/event-binding-test.html`)

Dedicated test suite for all event binding functionality.

**Tests included:**

1. **Global Event Binding (`client.bind`)**
   - Bind to specific events globally
   - Trigger events and verify callbacks
   - Unbind specific events
   - Verify unbind works

2. **Global All Events (`client.bind_global`)**
   - Bind to all events globally
   - Trigger multiple events
   - Verify all events are captured
   - Unbind global callbacks

3. **Channel Event Binding (`channel.bind`)**
   - Bind to specific events on a channel
   - Trigger channel-specific events
   - Verify channel isolation
   - Unbind channel events

4. **Channel Global Binding (`channel.bind_global`)**
   - Bind to all events on a specific channel
   - Trigger multiple channel events
   - Verify all channel events captured
   - Unbind channel global callbacks

5. **Multiple Callbacks**
   - Register multiple callbacks for same event
   - Verify all callbacks are invoked
   - Test callback order

6. **Unbind All (`client.unbind_all`)**
   - Register multiple callbacks
   - Unbind all at once
   - Verify no callbacks fire after unbind

7. **Pusher Protocol Events**
   - Test `pusher:connection_established`
   - Test `pusher:ping`/`pusher:pong`
   - Test `pusher:subscribe` success/error

**Accessing the test:**
```bash
cd examples/browser
./serve.sh
```
Then navigate to: `http://localhost:8000/tests/interactive/public/event-binding-test.html`

## API Coverage

### Exported Classes

The WASM build exports the following clean JavaScript API:

```javascript
// Main client
import init, { SockudoClient, SockudoOptions, DeltaOptions, FilterOp } from './pkg/sockudo_client.js';

await init();

// Create options
const options = new SockudoOptions('your-app-key');
options.ws_host = 'ws-mt1.pusher.com';
options.use_tls = true;

// Configure delta compression
const deltaOpts = new DeltaOptions();
deltaOpts.enabled = true;
deltaOpts.setAlgorithms('fossil,xdelta3');
deltaOpts.max_messages_per_key = 10;
options.setDeltaCompression(deltaOpts);

// Create client
const client = new SockudoClient('your-app-key', options);

// Connect
await client.connect();

// Subscribe to channel
const channel = client.subscribe('my-channel', null);

// Subscribe with filter
const filter = FilterOp.eq('type', 'important');
const filteredChannel = client.subscribe('filtered-channel', filter);

// Bind events
client.bind('my-event', (data) => {
    console.log('Event received:', data);
});

// Bind to all events
client.bind_global((data) => {
    console.log('Any event received:', data);
});

// Channel-specific binding
channel.bind('channel-event', (data) => {
    console.log('Channel event:', data);
});

// Channel global binding
channel.bind_global((eventName, data) => {
    console.log('Any channel event:', eventName, data);
});

// Unbind
client.unbind('my-event');
client.unbind_global();
client.unbind_all();

channel.unbind('channel-event');
channel.unbind_global();
channel.unbind_all();

// Get stats
const stats = client.get_delta_stats();
client.reset_delta_stats();

// Disconnect
client.disconnect();
```

### Filter Operations

All filter operations are available:

```javascript
// Comparison filters
FilterOp.eq('field', 'value')
FilterOp.neq('field', 'value')
FilterOp.gt('field', 'value')
FilterOp.gte('field', 'value')
FilterOp.lt('field', 'value')
FilterOp.lte('field', 'value')

// Set filters
FilterOp.inSet('field', ['value1', 'value2'])
FilterOp.notIn('field', ['value1', 'value2'])

// Existence filters
FilterOp.exists('field')
FilterOp.notExists('field')

// Logical operators
FilterOp.and([filter1, filter2])
FilterOp.or([filter1, filter2])

// Complex nested filters
const complexFilter = FilterOp.and([
    FilterOp.eq('status', 'active'),
    FilterOp.or([
        FilterOp.eq('role', 'admin'),
        FilterOp.eq('role', 'moderator')
    ]),
    FilterOp.gt('age', '18')
]);
```

## Test Results Expected

When running the comprehensive test suite, you should see:

1. **Connection Tests:** All tests passing
2. **Channel Tests:** All subscriptions successful
3. **Delta Compression:** Compression ratio > 0% when enabled
4. **Filtering:** Only filtered events received
5. **Event Binding:** All callbacks firing correctly
6. **Unbind:** No callbacks after unbind operations

## Known Issues and Limitations

1. **Delta Compression Status:** The status display in the interactive interface may not update immediately. The compression itself works correctly, but the UI state needs manual refresh.

2. **Authentication:** Private and presence channel tests require a running auth server at the configured endpoint.

3. **Browser Compatibility:** Tests are designed for modern browsers (Chrome, Firefox, Safari, Edge). Older browsers may not support all features.

## Adding New Tests

### Adding Rust Tests

1. Open `tests/wasm_comprehensive_tests.rs`
2. Add a new test function:

```rust
#[wasm_bindgen_test]
fn test_my_new_feature() {
    console::log_1(&"Test: My new feature".into());
    
    // Test code here
    
    assert!(true, "Test should pass");
}
```

3. Run the test:
```bash
wasm-pack test --headless --firefox
```

### Adding Interactive Tests

1. Open `tests/interactive/public/comprehensive-test.html` or create a new HTML file
2. Add a new test section
3. Implement test logic in the module script
4. Add buttons and logging

## Troubleshooting

### Tests fail to connect

- Verify your app key is correct
- Check that the WebSocket host is reachable
- Ensure firewall allows WebSocket connections

### Delta compression not working

- Verify Emscripten is properly installed
- Check that xdelta3 dependency compiled correctly
- Enable debug logging to see compression events

### Events not firing

- Check that you're connected before binding
- Verify event names match exactly
- Use `bind_global` to see all events

### WASM initialization fails

- Clear browser cache
- Check browser console for errors
- Verify WASM file is being served with correct MIME type

## CI/CD Integration

To integrate these tests into your CI pipeline:

```bash
# Install dependencies
cargo install wasm-pack

# Run tests
wasm-pack test --headless --firefox
wasm-pack test --headless --chrome

# Build WASM package
./build-wasm.sh
```

## Contributing

When adding new features, please:

1. Add corresponding Rust tests in `tests/wasm_comprehensive_tests.rs`
2. Add interactive tests in the HTML test suites
3. Update this README with new test coverage
4. Ensure all tests pass before submitting PR

## License

Same as the main Sockudo project.
