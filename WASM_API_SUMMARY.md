# Sockudo WASM API Summary

## Clean JavaScript API

Your WASM library now exports with clean, professional names:

### Exported Classes

- ✅ `SockudoClient` (was `WasmSockudo`)
- ✅ `SockudoOptions` (was `WasmOptions`)
- ✅ `DeltaOptions` (was `WasmDeltaOptions`)
- ✅ `FilterOp` (was `WasmFilterOp`)
- ✅ `Channel`
- ✅ `PresenceChannel`
- ✅ `Member`

## Usage Example

```javascript
import init, { SockudoClient, SockudoOptions, DeltaOptions, FilterOp } from './pkg/sockudo_client.js';

// Initialize WASM
await init();

// Create client with options
const options = new SockudoOptions('your-app-key');
options.ws_host = 'ws-mt1.pusher.com';
options.use_tls = true;

// Optional: Enable delta compression
const deltaOpts = new DeltaOptions();
deltaOpts.enabled = true;
deltaOpts.setAlgorithms('fossil,xdelta3');
options.setDeltaCompression(deltaOpts);

// Create and connect
const client = new SockudoClient('your-app-key', options);
await client.connect();

// Subscribe to channels
const channel = client.subscribe('my-channel', null);

// Subscribe with filter
const filter = FilterOp.eq('type', 'important');
const filtered = client.subscribe('filtered-channel', filter);

// Bind events - multiple ways

// 1. Global event binding
client.bind('my-event', (data) => {
    console.log('Event received:', data);
});

// 2. Global all events
client.bind_global((data) => {
    console.log('Any event:', data);
});

// 3. Channel-specific events
channel.bind('channel-event', (data) => {
    console.log('Channel event:', data);
});

// 4. Channel all events
channel.bind_global((eventName, data) => {
    console.log('Channel event:', eventName, data);
});

// Unbind events
client.unbind('my-event');        // Unbind specific event
client.unbind_global();            // Unbind all global callbacks
client.unbind_all();               // Unbind everything

channel.unbind('channel-event');   // Unbind specific channel event
channel.unbind_global();           // Unbind channel global callbacks
channel.unbind_all();              // Unbind all channel callbacks

// Send events
client.send_event('custom-event', { message: 'hello' }, 'my-channel');

// Disconnect
client.disconnect();
```

## All Filter Operations

```javascript
// Equality
FilterOp.eq('status', 'active')
FilterOp.neq('status', 'inactive')

// Comparison
FilterOp.gt('age', '18')
FilterOp.gte('age', '18')
FilterOp.lt('age', '65')
FilterOp.lte('age', '65')

// Sets
FilterOp.inSet('role', ['admin', 'moderator'])
FilterOp.notIn('status', ['banned', 'suspended'])

// Existence
FilterOp.exists('premium')
FilterOp.notExists('banned')

// Logical operators
FilterOp.and([filter1, filter2, filter3])
FilterOp.or([filter1, filter2])

// Complex nested example
const complexFilter = FilterOp.and([
    FilterOp.eq('verified', 'true'),
    FilterOp.or([
        FilterOp.eq('plan', 'premium'),
        FilterOp.eq('plan', 'enterprise')
    ]),
    FilterOp.gt('score', '100')
]);
```

## API Methods

### SockudoClient

```typescript
class SockudoClient {
    constructor(app_key: string, options?: SockudoOptions)
    
    // Connection
    async connect(): Promise<void>
    disconnect(): void
    
    // State
    get state(): string
    get socket_id(): string | null
    
    // Channels
    subscribe(channel_name: string, filter?: FilterOp): Channel
    unsubscribe(channel_name: string): void
    channel(name: string): Channel | null
    
    // Event binding
    bind(event_name: string, callback: Function): void
    bind_global(callback: Function): void
    unbind(event_name?: string): void
    unbind_global(): void
    unbind_all(): void
    
    // Events
    send_event(event_name: string, data: any, channel?: string): boolean
    
    // Delta compression
    get_delta_stats(): any
    reset_delta_stats(): void
}
```

### Channel

```typescript
class Channel {
    get name(): string
    get subscribed(): boolean
    
    // Event binding (returns this for chaining)
    bind(event_name: string, callback: Function): Channel
    bind_global(callback: Function): Channel
    unbind(event_name?: string): Channel
    unbind_global(): Channel
    unbind_all(): Channel
    
    // Client events (private/presence channels only)
    trigger(event_name: string, data: any): boolean
}
```

### SockudoOptions

```typescript
class SockudoOptions {
    constructor(app_key: string)
    
    cluster?: string
    ws_host?: string
    ws_port?: number
    use_tls?: boolean
    auth_endpoint?: string
    
    setDeltaCompression(options: DeltaOptions): void
    enableDeltaCompression(): void  // Quick enable with defaults
}
```

### DeltaOptions

```typescript
class DeltaOptions {
    constructor()
    
    enabled: boolean
    max_messages_per_key: number
    debug: boolean
    
    setAlgorithms(algorithms: string): void  // e.g., "fossil,xdelta3"
}
```

## Test Suites

### 1. Automated Rust Tests
```bash
wasm-pack test --headless --firefox
```

### 2. Interactive Browser Tests

Navigate to:
- `http://localhost:8000/tests/interactive/public/comprehensive-test.html`
- `http://localhost:8000/tests/interactive/public/event-binding-test.html`

Test coverage:
- ✅ Client initialization
- ✅ Connection/disconnection
- ✅ Channel subscription (public, private, presence)
- ✅ Event binding (all methods)
- ✅ Delta compression
- ✅ Conflation keys
- ✅ Publish filtering (all filter types)
- ✅ Multiple callbacks
- ✅ Unbind operations
- ✅ Pusher protocol events

## Building

```bash
# Build WASM package
./build-wasm.sh

# Development build
./build-wasm.sh --dev

# Debug build with symbols
./build-wasm.sh --debug
```

## Files Generated

After building, the `pkg/` directory contains:

- `sockudo_client.js` - JavaScript bindings
- `sockudo_client_bg.wasm` - WebAssembly binary
- `sockudo_client.d.ts` - TypeScript definitions
- `package.json` - NPM package metadata

## TypeScript Support

Full TypeScript definitions are included. Your IDE will provide:
- Autocomplete for all methods
- Type checking
- Inline documentation
- Parameter hints

## Cross-Platform

This same WASM build works on:
- ✅ Modern browsers (Chrome, Firefox, Safari, Edge)
- ✅ Node.js (with WASM support)
- ✅ Electron apps
- ✅ Progressive Web Apps (PWAs)
- ✅ Web Workers
- ✅ Service Workers

## Next Steps

1. Test the interactive test suites
2. Integrate into your application
3. Configure delta compression for your use case
4. Set up filters for your event streams
5. Deploy to production

See `tests/WASM_TEST_SUITE.md` for detailed testing documentation.
