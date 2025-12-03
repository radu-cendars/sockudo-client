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
npm install sockudo-client
# or
yarn add sockudo-client
# or
pnpm add sockudo-client
```

### Flutter/Dart

```yaml
# pubspec.yaml
dependencies:
  sockudo_client: ^0.1.0
```

---

## 🚀 Quick Start

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

### JavaScript/TypeScript (Browser)

```javascript
import init, { Pusher, SockudoOptions } from 'sockudo-client';

// Initialize WASM module
await init();

// Configure client
const options = new SockudoOptions('your-app-key');
options.cluster = 'mt1';
options.ws_host = 'localhost';
options.ws_port = 6001;
options.use_tls = false;
options.debug = true;

const pusher = new Pusher('your-app-key', options);

// Connect
await pusher.connect();
console.log('Connected! Socket ID:', pusher.socket_id());

// Subscribe to channel
const channel = pusher.subscribe('my-channel');

// Bind to events
channel.bind('my-event', (event) => {
    console.log('Received:', event.data);
});
```

### JavaScript/TypeScript (Node.js)

```javascript
const { Pusher, SockudoOptions } = require('sockudo-client');

async function main() {
    // Configure client
    const options = new SockudoOptions('your-app-key');
    options.cluster = 'mt1';
    options.ws_host = 'localhost';
    options.ws_port = 6001;
    options.use_tls = false;
    
    const pusher = new Pusher('your-app-key', options);
    
    // Connect
    await pusher.connect();
    console.log('Connected! Socket ID:', pusher.socket_id());
    
    // Subscribe and bind
    const channel = pusher.subscribe('my-channel');
    channel.bind('my-event', (event) => {
        console.log('Received:', event.data);
    });
    
    // Keep alive
    await new Promise(resolve => setTimeout(resolve, 3600000));
}

main().catch(console.error);
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

### Channel Types

#### 1. Public Channels

Public channels don't require authentication.

```rust
// Rust
let channel = client.subscribe("my-channel")?;

// JavaScript
const channel = pusher.subscribe('my-channel');

// Kotlin
val channel = client.subscribe("my-channel")

// Swift
let channel = try client.subscribe(channelName: "my-channel")

// Flutter
final channel = client.subscribe('my-channel');
```

#### 2. Private Channels

Private channels require server-side authentication.

```rust
// Rust - Configure auth endpoint
let options = PusherOptions::new("key")
    .auth_endpoint("https://your-server.com/pusher/auth");

let client = SockudoClient::new(options.into())?;
let channel = client.subscribe("private-my-channel")?;

// Trigger client events (private channels only)
channel.trigger("client-message", serde_json::json!({
    "text": "Hello!"
}).to_string())?;
```

```javascript
// JavaScript
const options = new SockudoOptions('key');
options.auth_endpoint = 'https://your-server.com/pusher/auth';

const pusher = new Pusher('key', options);
const channel = pusher.subscribe('private-my-channel');

// Trigger client events
channel.trigger('client-message', { text: 'Hello!' });
```

#### 3. Presence Channels

Track who's online in real-time.

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

```javascript
// JavaScript
const channel = pusher.subscribe('presence-chat-room');

channel.bind('pusher:subscription_succeeded', (event) => {
    console.log('Members:', event.data.members);
    console.log('Count:', event.data.count);
});

channel.bind('pusher:member_added', (event) => {
    console.log('Member joined:', event.data.user_id);
});

channel.bind('pusher:member_removed', (event) => {
    console.log('Member left:', event.data.user_id);
});
```

#### 4. Private-Encrypted Channels

End-to-end encryption for sensitive data.

```rust
// Rust
let channel = client.subscribe("private-encrypted-secrets")?;

// Messages are automatically decrypted
channel.bind("secure-message", |event| {
    println!("Decrypted: {:?}", event.data);
});
```

### Delta Compression

Reduce bandwidth usage by sending only differences between messages.

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

```javascript
// JavaScript
const options = new SockudoOptions('key');
options.enable_delta_compression = true;

const pusher = new Pusher('key', options);

// Get stats
const stats = pusher.get_delta_stats();
console.log(`Bandwidth saved: ${stats.bandwidth_saved_percent}%`);
```

### Tag Filtering

Filter events server-side to reduce client processing.

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

```javascript
// JavaScript
pusher.connection.bind('state_change', (states) => {
    console.log(`State changed: ${states.previous} -> ${states.current}`);
});

// Check state
if (pusher.connection.state === 'connected') {
    console.log("We're connected!");
}

// Disconnect
await pusher.disconnect();
```

### Auto-Reconnection

Configure reconnection behavior:

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

Gracefully shutdown on Ctrl+C or termination signals.

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

### JavaScript Examples

See `nodejs/examples/` directory for Node.js examples and `pkg/examples/` for browser examples.

### Mobile Examples

- **Android**: See `kotlin/example/` directory
- **iOS**: See `swift/example/` directory  
- **Flutter**: See `flutter/example/` directory

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
└── README.md                   # This file
```

---

## 🧪 Testing

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
