# Sockudo WASM Quick Start

## 🚀 Get Started in 3 Steps

### 1. Build the WASM package

```bash
./build-wasm.sh
```

### 2. Use in your app

```javascript
import init, { SockudoClient, SockudoOptions } from './pkg/sockudo_client.js';

await init();

const sockudo = new SockudoClient('your-app-key', {
    ws_host: 'ws-mt1.pusher.com',
    use_tls: true
});

await sockudo.connect();

const channel = sockudo.subscribe('my-channel');
channel.bind('my-event', (data) => {
    console.log('Received:', data);
});
```

### 3. Test everything

```bash
cd examples/browser
./serve.sh
```

Open: `http://localhost:8000/tests/interactive/public/comprehensive-test.html`

## 📚 Quick API Reference

```javascript
// Create client
const client = new SockudoClient(appKey, options);
await client.connect();

// Subscribe
const channel = client.subscribe('channel-name');

// Bind events
client.bind('event', callback);              // Global event
client.bind_global(callback);                // All global events
channel.bind('event', callback);             // Channel event
channel.bind_global(callback);               // All channel events

// Unbind
client.unbind('event');                      // Unbind specific
client.unbind_all();                         // Unbind everything

// Filters
const filter = FilterOp.eq('type', 'important');
client.subscribe('filtered', filter);

// Disconnect
client.disconnect();
```

## 🧪 Test Suites

| Test Suite | URL | What it tests |
|------------|-----|---------------|
| Comprehensive | `/comprehensive-test.html` | Everything |
| Event Binding | `/event-binding-test.html` | All bind methods |
| Rust Tests | `wasm-pack test --headless --firefox` | Automated tests |

## 🎯 Key Features

✅ Clean API: `SockudoClient`, not `WasmSockudo`  
✅ Delta compression with Fossil & Xdelta3  
✅ Publish filtering (eq, gt, lt, in, exists, and, or)  
✅ Conflation keys  
✅ Event binding (global, channel, multiple callbacks)  
✅ Private & presence channels  
✅ Full TypeScript support  

## 📖 Documentation

- `WASM_API_SUMMARY.md` - Complete API reference
- `tests/WASM_TEST_SUITE.md` - Testing guide
- TypeScript definitions in `pkg/sockudo_client.d.ts`

## 🐛 Issues Fixed

✅ Line endings (Windows CRLF → Unix LF)  
✅ Emscripten wrapper for Linux  
✅ Clean JavaScript exports  
✅ Comprehensive test coverage  

## 💡 Examples

See the test suites for working examples of:
- Connection management
- Channel subscriptions
- Event binding patterns
- Filter usage
- Delta compression
- Private channels

---

**Need help?** Check the test suites - they're fully working examples!
