# Sockudo Client

A high-performance, cross-platform Pusher-compatible WebSocket client library written in Rust with bindings for multiple languages and platforms.

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org)

## 🌍 Supported Platforms

| Platform | Technology | Status |
|----------|-----------|--------|
| 🦀 **Rust** | Native | ✅ Fully Supported |
| 📱 **Android/Kotlin** | UniFFI | ✅ Fully Supported |
| 🍎 **iOS/Swift** | UniFFI | ✅ Fully Supported |
| 🌐 **Browser (Web)** | WebAssembly | ✅ Fully Supported |
| 🖥️ **Node.js** | WebAssembly | ✅ Fully Supported |
| 🐦 **Flutter/Dart** | flutter_rust_bridge | ✅ Fully Supported |

## ✨ Features

- ✅ **Full Pusher Protocol v7 Compatibility**
- ✅ **Channel Types**: Public, Private, Presence, and Private-Encrypted
- ✅ **Delta Compression**: Reduce bandwidth by up to 70% (Fossil & Xdelta3 algorithms)
- ✅ **Tag Filtering**: Server-side event filtering
- ✅ **Auto-Reconnection**: Exponential backoff with configurable limits
- ✅ **Activity Monitoring**: Built-in ping/pong keep-alive
- ✅ **End-to-End Encryption**: For private-encrypted channels
- ✅ **Cross-Platform Signal Handling**: Graceful shutdown on all platforms
- ✅ **Type-Safe APIs**: Strong typing across all language bindings

---

## 📦 Installation

### Rust

```toml
[dependencies]
sockudo-client = "0.1"
tokio = { version = "1", features = ["full"] }
```

### Kotlin/Android

```kotlin
// build.gradle.kts
dependencies {
    implementation("io.sockudo:sockudo-client:0.1.0")
}
```

### Swift/iOS

```swift
// Package.swift
dependencies: [
    .package(url: "https://github.com/sockudo/sockudo-client-swift", from: "0.1.0")
]
```

### JavaScript/TypeScript (Browser & Node.js)

```bash
npm install sockudo
# or
yarn add sockudo
# or
pnpm add sockudo
```

### Flutter/Dart

```yaml
# pubspec.yaml
dependencies:
  sockudo_client: ^0.1.0
```

---

## 🚀 Quick Start

### JavaScript/TypeScript (Browser)

The WASM client provides a full-featured WebSocket implementation for browsers.

```javascript
import init, { WasmSockudo, WasmOptions } from 'sockudo';

// Initialize WASM module (required before using any WASM functions)
await init();

// Create options
const options = new WasmOptions('your-app-key');
options.cluster = 'mt1';
options.ws_host = 'localhost';
options.ws_port = 6001;
options.use_tls = false;
options.auth_endpoint = 'https://your-server.com/pusher/auth';

// Enable delta compression (optional)
options.enableDeltaCompression();

// Create client
const client = new WasmSockudo('your-app-key', options);

// Set up connection event handlers
client.bind('pusher:connection_established', (data) => {
    console.log('Connected! Socket ID:', data.socket_id);
});

client.bind('state_change', (data) => {
    console.log('State changed:', data.previous, '->', data.current);
});

// Connect to server
await client.connect();

// Subscribe to a channel
const channel = client.subscribe('my-channel');

// Bind to events on the channel
channel.bind('my-event', (event) => {
    console.log('Received event:', event);
});

// Send client events (requires private/presence channel)
channel.trigger('client-message', { text: 'Hello!' });

// Disconnect when done
client.disconnect();
```

### JavaScript/TypeScript (Node.js)

```javascript
const { WasmSockudo, WasmOptions } = require('sockudo');

async function main() {
    // Create options
    const options = new WasmOptions('your-app-key');
    options.cluster = 'mt1';
    options.ws_host = 'localhost';
    options.ws_port = 6001;
    options.use_tls = false;
    
    // Create and connect client
    const client = new WasmSockudo('your-app-key', options);
    
    client.bind('pusher:connection_established', (data) => {
        console.log('Connected!');
    });
    
    await client.connect();
    
    // Subscribe to channel
    const channel = client.subscribe('my-channel');
    channel.bind('my-event', (event) => {
        console.log('Event received:', event);
    });
    
    // Keep running...
    await new Promise(resolve => setTimeout(resolve, 60000));
    
    client.disconnect();
}

main().catch(console.error);
```

### Pusher-Compatible Wrapper

For easier migration from Pusher JS, you can create a compatibility wrapper:

```javascript
class PusherCompat {
    constructor(appKey, config) {
        const options = new WasmOptions(appKey);
        
        if (config.cluster) options.cluster = config.cluster;
        if (config.wsHost) options.ws_host = config.wsHost;
        if (config.wsPort) options.ws_port = parseInt(config.wsPort);
        if (config.forceTLS !== undefined) options.use_tls = config.forceTLS;
        if (config.authEndpoint) options.auth_endpoint = config.authEndpoint;
        
        this._client = new WasmSockudo(appKey, options);
        
        // Create connection object
        this.connection = {
            state: 'initialized',
            socket_id: null,
            
            bind: (event, callback) => {
                this._client.bind(event, callback);
            }
        };
        
        // Monitor connection events
        this._client.bind('pusher:connection_established', (data) => {
            this.connection.state = 'connected';
            this.connection.socket_id = data.socket_id || this._client.socket_id;
        });
    }
    
    async connect() {
        await this._client.connect();
    }
    
    disconnect() {
        this._client.disconnect();
    }
    
    subscribe(channelName) {
        return this._client.subscribe(channelName);
    }
    
    bind(event, callback) {
        this._client.bind(event, callback);
    }
}

// Usage
const pusher = new PusherCompat('app-key', {
    cluster: 'mt1',
    wsHost: 'localhost',
    wsPort: 6001,
    forceTLS: false,
    authEndpoint: 'http://localhost:3000/pusher/auth'
});

await pusher.connect();
const channel = pusher.subscribe('my-channel');
channel.bind('my-event', (data) => console.log(data));
```

### Rust

```rust
use sockudo_client::{SockudoClient, PusherOptions};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure client
    let options = PusherOptions::new("your-app-key")
        .cluster("mt1")
        .ws_host("localhost")
        .ws_port(6001)
        .use_tls(false)
        .debug(true);

    let client = SockudoClient::new(options.into())?;
    
    // Connect to server
    client.connect().await?;
    println!("Connected! Socket ID: {:?}", client.socket_id());
    
    // Subscribe to a channel
    let channel = client.subscribe("my-channel")?;
    
    // Bind to events
    channel.bind("my-event", |event| {
        println!("Received: {:?}", event.data);
    });
    
    // Keep alive until signal
    sockudo_client::utils::wait_for_signal().await;
    client.disconnect().await;
    
    Ok(())
}
```

### Kotlin/Android

```kotlin
import io.sockudo.client.*
import kotlinx.coroutines.*

fun main() = runBlocking {
    // Configure client
    val options = SockudoOptions(
        appKey = "your-app-key",
        cluster = "mt1",
        wsHost = "localhost",
        wsPort = 6001u,
        useTls = false,
        debug = true
    )

    val client = SockudoClient(options)
    
    // Connect
    client.connect()
    println("Connected! Socket ID: ${client.socketId()}")
    
    // Subscribe to channel
    val channel = client.subscribe("my-channel")
    
    // Bind to events
    channel.bind("my-event", object : EventCallback {
        override fun onEvent(event: PusherEvent) {
            println("Received: ${event.data}")
        }
    })
    
    // Keep running
    delay(Long.MAX_VALUE)
}
```

### Swift/iOS

```swift
import SockudoClient

Task {
    // Configure client
    let options = SockudoOptions(
        appKey: "your-app-key",
        cluster: "mt1",
        wsHost: "localhost",
        wsPort: 6001,
        useTls: false,
        debug: true
    )
    
    let client = try SockudoClient(options: options)
    
    // Connect
    try await client.connect()
    print("Connected! Socket ID: \(client.socketId() ?? "nil")")
    
    // Subscribe to channel
    let channel = try client.subscribe(channelName: "my-channel")
    
    // Bind to events
    channel.bind(eventName: "my-event", callback: MyEventCallback())
    
    // Keep running
    try await Task.sleep(for: .seconds(3600))
}

class MyEventCallback: EventCallback {
    func onEvent(event: PusherEvent) {
        print("Received: \(event.data ?? "nil")")
    }
}
```

### Flutter/Dart

```dart
import 'package:sockudo_client/sockudo_client.dart';

Future<void> main() async {
  // Configure client
  final options = PusherOptions(
    appKey: 'your-app-key',
    cluster: 'mt1',
    wsHost: 'localhost',
    wsPort: 6001,
    useTls: false,
    debug: true,
  );

  final client = SockudoClient(options);
  
  // Connect
  await client.connect();
  print('Connected! Socket ID: ${client.socketId()}');
  
  // Subscribe to channel
  final channel = client.subscribe('my-channel');
  
  // Bind to events
  channel.bind('my-event', (event) {
    print('Received: ${event.data}');
  });
  
  // Keep running
  await Future.delayed(Duration(hours: 1));
}
```

---

## 📚 Complete Documentation

### WASM Client API

#### Initialization

```javascript
import init from 'sockudo';

// Must be called before using any WASM functions
await init();
```

#### WasmOptions Configuration

```javascript
const options = new WasmOptions('app-key');

// Connection settings
options.cluster = 'mt1';              // Cluster name
options.ws_host = 'localhost';        // WebSocket host
options.ws_port = 6001;               // WebSocket port
options.use_tls = false;              // Use TLS/SSL

// Authentication
options.auth_endpoint = 'https://your-server.com/pusher/auth';

// Delta compression
options.enableDeltaCompression();     // Enable bandwidth-saving compression

// Reconnection settings
options.max_reconnection_attempts = 10;    // 0 = unlimited
options.reconnection_delay_ms = 1000;      // Initial delay
options.max_reconnection_delay_ms = 30000; // Max delay
```

#### WasmSockudo Client Methods

```javascript
const client = new WasmSockudo('app-key', options);

// Connection
await client.connect();               // Connect to server
client.disconnect();                  // Disconnect from server
client.socket_id;                     // Get current socket ID

// Channels
const channel = client.subscribe('channel-name');
client.unsubscribe('channel-name');
const ch = client.channel('channel-name');  // Get existing channel

// Event binding
client.bind('event-name', (data) => {
    console.log('Event received:', data);
});

client.bind_global((eventName, data) => {
    console.log('Any event:', eventName, data);
});

client.unbind('event-name');
client.unbind_all();
client.unbind_global();

// Send custom events
client.send_event('pusher:enable_delta_compression', {});

// Delta compression stats
const stats = client.get_delta_stats();
client.reset_delta_stats();
```

#### WasmChannel Methods

```javascript
const channel = client.subscribe('my-channel');

// Event binding
channel.bind('event-name', (data) => {
    console.log('Channel event:', data);
});

channel.unbind('event-name');
channel.unbind_all();

// Client events (requires private/presence channel)
channel.trigger('client-event', { message: 'Hello!' });
```

### Channel Types

#### 1. Public Channels

Public channels don't require authentication.

```javascript
// JavaScript
const channel = client.subscribe('my-channel');
channel.bind('my-event', (data) => {
    console.log('Event data:', data);
});

// Rust
let channel = client.subscribe("my-channel")?;
channel.bind("my-event", |event| {
    println!("Received: {:?}", event.data);
});
```

#### 2. Private Channels

Private channels require server-side authentication.

```javascript
// JavaScript - Configure auth endpoint
const options = new WasmOptions('key');
options.auth_endpoint = 'https://your-server.com/pusher/auth';

const client = new WasmSockudo('key', options);
const channel = client.subscribe('private-my-channel');

// Trigger client events (private channels only)
channel.trigger('client-message', { text: 'Hello!' });
```

```rust
// Rust - Configure auth endpoint
let options = PusherOptions::new("key")
    .auth_endpoint("https://your-server.com/pusher/auth");

let client = SockudoClient::new(options.into())?;
let channel = client.subscribe("private-my-channel")?;

// Trigger client events
channel.trigger("client-message", serde_json::json!({
    "text": "Hello!"
}).to_string())?;
```

#### 3. Presence Channels

Track who's online in real-time.

```javascript
// JavaScript
const channel = client.subscribe('presence-chat-room');

// Subscription succeeded - get initial members
channel.bind('pusher:subscription_succeeded', (data) => {
    console.log('Members:', data.members);
    console.log('Count:', data.count);
});

// Member joined
channel.bind('pusher:member_added', (data) => {
    console.log('Member joined:', data.user_id);
});

// Member left
channel.bind('pusher:member_removed', (data) => {
    console.log('Member left:', data.user_id);
});
```

```rust
// Rust
let channel = client.subscribe("presence-chat-room")?;

// Subscription succeeded - get initial members
channel.bind("pusher:subscription_succeeded", |event| {
    if let Some(data_str) = &event.data {
        if let Ok(data) = serde_json::from_str::<serde_json::Value>(data_str) {
            println!("Members: {:?}", data.get("members"));
            println!("Count: {:?}", data.get("count"));
        }
    }
});

// Member joined
channel.bind("pusher:member_added", |event| {
    println!("Member joined: {:?}", event.data);
});

// Member left
channel.bind("pusher:member_removed", |event| {
    println!("Member left: {:?}", event.data);
});
```

#### 4. Private-Encrypted Channels

End-to-end encryption for sensitive data.

```javascript
// JavaScript
const channel = client.subscribe('private-encrypted-secrets');

// Messages are automatically decrypted
channel.bind('secure-message', (data) => {
    console.log('Decrypted:', data);
});
```

```rust
// Rust
let channel = client.subscribe("private-encrypted-secrets")?;

// Messages are automatically decrypted
channel.bind("secure-message", |event| {
    println!("Decrypted: {:?}", event.data);
});
```

### Delta Compression

Reduce bandwidth usage by up to 70% by sending only differences between messages.

```javascript
// JavaScript - Enable during client creation
const options = new WasmOptions('key');
options.enableDeltaCompression();

const client = new WasmSockudo('key', options);
await client.connect();

// Or enable after connection
client.send_event('pusher:enable_delta_compression', {});

// Monitor delta compression
client.bind('pusher:delta_compression_enabled', (data) => {
    console.log('Delta compression enabled:', data);
});

// Check compression stats
const stats = client.get_delta_stats();
if (stats) {
    console.log('Total messages:', stats.total_messages);
    console.log('Delta messages:', stats.delta_messages);
    console.log('Bandwidth saved:', stats.bandwidth_saved_percent + '%');
}
```

```rust
// Rust - Enable delta compression
let options = PusherOptions::new("key")
    .delta_compression(true)
    .delta_algorithms(vec![DeltaAlgorithm::Fossil, DeltaAlgorithm::Xdelta3]);

let client = SockudoClient::new(options.into())?;

// Check compression stats
if let Some(stats) = client.get_delta_stats() {
    println!("Total messages: {}", stats.total_messages);
    println!("Delta messages: {}", stats.delta_messages);
    println!("Bandwidth saved: {:.1}%", stats.bandwidth_saved_percent);
}
```

### Tag Filtering

Filter events server-side to reduce client processing and bandwidth.

**Note:** Tag filtering requires server-side support. See [Sockudo Server](https://github.com/sockudo/sockudo) documentation.

```javascript
// JavaScript - Subscribe with tag filter
// (Currently requires manual JSON construction)
const filter = {
    key: "event_type",
    cmp: "eq",
    val: "goal"
};

// Complex filters
const complexFilter = {
    op: "and",
    nodes: [
        { key: "event_type", cmp: "eq", val: "shot" },
        { key: "xG", cmp: "gte", val: "0.8" }
    ]
};

// Note: Filter parameter support in subscribe() is planned for future release
```

```rust
// Rust
use sockudo_client::FilterOp;

// Simple filter - only "goal" events
let filter = FilterOp::eq("type", "goal");
let channel = client.subscribe_with_filter("sports", Some(filter))?;

// Complex filter
let filter = FilterOp::and(vec![
    FilterOp::eq("type", "goal"),
    FilterOp::in_set("league", vec!["premier".to_string(), "champions".to_string()]),
    FilterOp::ne("team", "excluded-team"),
]);

let channel = client.subscribe_with_filter("sports-updates", Some(filter))?;
```

### Connection Management

```javascript
// JavaScript - Connection state events
client.bind('pusher:connection_established', (data) => {
    console.log('Connected! Socket ID:', data.socket_id);
});

client.bind('state_change', (data) => {
    console.log(`State changed: ${data.previous} -> ${data.current}`);
});

client.bind('connecting', () => console.log('Connecting...'));
client.bind('connected', () => console.log('Connected!'));
client.bind('disconnected', () => console.log('Disconnected'));
client.bind('unavailable', () => console.log('Connection unavailable'));
client.bind('failed', () => console.log('Connection failed'));
client.bind('error', (err) => console.error('Error:', err));

// Manual disconnect
client.disconnect();
```

```rust
// Rust - Connection state events
client.bind_global(|event| {
    match event.event.as_str() {
        "connecting" => println!("Connecting..."),
        "connected" => println!("Connected!"),
        "disconnected" => println!("Disconnected"),
        "unavailable" => println!("Connection unavailable"),
        "failed" => println!("Connection failed"),
        _ => {}
    }
});

// Check connection state
if client.is_connected() {
    println!("We're connected!");
}

// Manual disconnect
client.disconnect().await;
```

### Auto-Reconnection

Configure reconnection behavior:

```javascript
// JavaScript
const options = new WasmOptions('key');
options.max_reconnection_attempts = 10;      // 0 = unlimited
options.reconnection_delay_ms = 1000;        // Initial delay
options.max_reconnection_delay_ms = 30000;   // Max delay
```

```rust
// Rust
let options = PusherOptions::new("key")
    .max_reconnection_attempts(10)           // 0 = unlimited
    .reconnection_delay_ms(1000)             // Initial delay
    .max_reconnection_delay_ms(30000);       // Max delay

// Disable reconnection
let options = PusherOptions::new("key")
    .disable_reconnection(true);
```

### Cross-Platform Signal Handling

Gracefully shutdown on Ctrl+C or termination signals (Rust only).

```rust
// Rust - Simple approach
use sockudo_client::utils::wait_for_signal;

client.connect().await?;
println!("Press Ctrl+C to stop...");

wait_for_signal().await;  // Waits for SIGINT, SIGTERM, SIGHUP (Unix) or Ctrl+C, Ctrl+Break (Windows)

client.disconnect().await;
```

```rust
// Rust - Advanced approach
use sockudo_client::utils::SignalHandler;

let mut signal_handler = SignalHandler::new()?;

// Application code...

signal_handler.wait().await;

// Cleanup...
```

**Supported Signals:**
- **Windows**: Ctrl+C (SIGINT), Ctrl+Break
- **Unix/Linux/macOS**: SIGINT, SIGTERM, SIGHUP

---

## 🏗️ Building from Source

### Prerequisites

- Rust 1.70+
- For WASM: `wasm-pack`
- For UniFFI: `uniffi-bindgen`

### Build Commands

```bash
# Clone repository
git clone https://github.com/sockudo/sockudo-client
cd sockudo-client

# Build Rust library
cargo build --release

# Run tests
cargo test

# Build WASM for browser
wasm-pack build --target web --features wasm

# Build WASM for Node.js
wasm-pack build --target nodejs --features wasm

# Generate Kotlin bindings
cargo run --features uniffi-bindgen --bin uniffi-bindgen -- \
    generate src/sockudo_client.udl --language kotlin --out-dir kotlin/

# Generate Swift bindings
cargo run --features uniffi-bindgen --bin uniffi-bindgen -- \
    generate src/sockudo_client.udl --language swift --out-dir swift/

# Build Flutter bindings
flutter_rust_bridge_codegen generate
```

---

## 📖 Examples

The repository includes comprehensive examples for all platforms:

### Rust Examples

```bash
# Basic usage
cargo run --example basic

# Presence channels
cargo run --example presence

# Signal handling
cargo run --example signal_handling
```

### JavaScript/WASM Examples

See `tests/interactive/` directory for comprehensive browser-based examples:

```bash
cd tests/interactive

# Install dependencies
npm install  # or bun install

# Build the WASM client
npm run build

# Start the test server
npm start

# Open browser to http://localhost:3000
```

The interactive test suite includes:
- **Delta Compression Testing**: See bandwidth savings in real-time
- **Conflation Keys**: Test multiple message streams with compression
- **Tag Filtering**: Server-side event filtering demonstrations
- **Presence Channels**: Real-time user tracking
- **Event Binding**: Global and channel-specific event handling

### Mobile Examples

- **Android**: See `kotlin/example/` directory
- **iOS**: See `swift/example/` directory  
- **Flutter**: See `flutter/example/` directory

---

## 🧪 Testing

### Rust Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_xdelta3_decoder

# Run with logging
RUST_LOG=debug cargo test

# Run examples
cargo run --example basic
```

### WASM Integration Tests

```bash
cd tests/interactive

# Run automated test suite
npm test

# Or with Bun
bun test

# Run specific test
npm test -- --grep "delta compression"
```

---

## 🏛️ Architecture

```
sockudo-client/
├── src/
│   ├── lib.rs                  # Main library entry
│   ├── pusher.rs               # Client implementation
│   ├── options.rs              # Configuration
│   ├── error.rs                # Error types
│   ├── channels/               # Channel implementations
│   │   ├── channel.rs          # Base channel
│   │   ├── channels.rs         # Channel manager
│   │   ├── presence_channel.rs # Presence channels
│   │   ├── encrypted_channel.rs# Encrypted channels
│   │   ├── private_channel.rs  # Private channels
│   │   └── members.rs          # Member management
│   ├── connection/             # Connection management
│   │   ├── manager.rs          # Connection manager
│   │   └── state.rs            # Connection states
│   ├── delta/                  # Delta compression
│   │   ├── manager.rs          # Delta manager
│   │   ├── decoders.rs         # Fossil & Xdelta3
│   │   ├── channel_state.rs    # Channel state tracking
│   │   └── types.rs            # Delta types
│   ├── events/                 # Event system
│   │   ├── dispatcher.rs       # Event dispatcher
│   │   └── callback.rs         # Callback registry
│   ├── protocol/               # Pusher protocol
│   │   ├── message_types.rs    # Message formats
│   │   └── filter.rs           # Tag filtering
│   ├── transports/             # Transport layer
│   │   ├── transport.rs        # Transport trait
│   │   ├── native.rs           # Native WebSocket
│   │   └── wasm.rs             # WASM WebSocket
│   ├── utils/                  # Utilities
│   │   ├── signals.rs          # Signal handling
│   │   ├── timers.rs           # Timer utilities
│   │   └── collections.rs      # Collection helpers
│   ├── auth.rs                 # Authentication
│   ├── ffi_types.rs            # FFI type conversions
│   ├── ffi_callbacks.rs        # FFI callback traits
│   ├── wasm.rs                 # WASM bindings
│   └── flutter_api.rs          # Flutter bindings
├── Cargo.toml                  # Rust dependencies
├── src/sockudo_client.udl      # UniFFI interface
├── pkg/                        # WASM build output
│   ├── sockudo_client.js       # JS bindings
│   ├── sockudo_client.d.ts     # TypeScript definitions
│   └── sockudo_client_bg.wasm  # WASM binary
├── tests/interactive/          # Browser test suite
│   ├── test-all.test.js        # Automated tests
│   ├── server.js               # Test backend
│   └── public/                 # Browser dashboard
└── README.md                   # This file
```

---

## 🤝 Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

---

## 📝 License

MIT License - see [LICENSE](LICENSE) file for details.

---

## 🔗 Related Projects

- [Sockudo Server](https://github.com/sockudo/sockudo) - Pusher-compatible server in Rust
- [Pusher Protocol](https://pusher.com/docs/channels/library_auth_reference/pusher-websockets-protocol/) - Official Pusher protocol documentation

---

## 💬 Support

- **Issues**: [GitHub Issues](https://github.com/sockudo/sockudo-client/issues)
- **Discussions**: [GitHub Discussions](https://github.com/sockudo/sockudo-client/discussions)
- **Documentation**: [Full API Docs](https://docs.rs/sockudo-client)

---

## 🌟 Acknowledgments

Built with ❤️ using:
- [Rust](https://www.rust-lang.org/)
- [UniFFI](https://github.com/mozilla/uniffi-rs) - For Kotlin/Swift bindings
- [flutter_rust_bridge](https://github.com/fzyzcjy/flutter_rust_bridge) - For Flutter bindings
- [wasm-bindgen](https://github.com/rustwasm/wasm-bindgen) - For WebAssembly bindings
- [Tokio](https://tokio.rs/) - Async runtime
