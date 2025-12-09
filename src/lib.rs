//! # Sockudo Client
//!
//! A high-performance Pusher-compatible WebSocket client library for Rust with
//! bindings for Kotlin, Swift, JavaScript/Node.js, and Flutter.
//!
//! ## Features
//!
//! - Full Pusher protocol compatibility
//! - Public, Private, Presence, and Encrypted channels
//! - Delta compression with Fossil and Xdelta3 algorithms
//! - Tag filtering for server-side event filtering
//! - Automatic reconnection with exponential backoff
//! - Activity/ping monitoring
//! - Cross-platform support via UniFFI (Kotlin/Swift) and wasm-bindgen (JS)
//!
//! ## Example
//!
//! ```ignore
//! use sockudo_client::{SockudoClient, PusherOptions};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let options = PusherOptions {
//!         app_key: "your-app-key".to_string(),
//!         cluster: Some("mt1".to_string()),
//!         ws_host: Some("your-server.com".to_string()),
//!         ws_port: Some(6001),
//!         use_tls: Some(false),
//!         ..Default::default()
//!     };
//!
//!     let client = SockudoClient::new(options.into())?;
//!     client.connect().await?;
//!
//!     let channel = client.subscribe("my-channel")?;
//!     channel.bind("my-event", |data| {
//!         println!("Received: {:?}", data);
//!     });
//!
//!     Ok(())
//! }
//! ```

#![allow(dead_code)]
#![allow(unused_variables)]

// Module declarations
pub mod auth;
pub mod channels;
pub mod connection;
pub mod delta;
pub mod events;
pub mod protocol;
#[cfg(not(target_arch = "wasm32"))]
pub mod transports;
pub mod utils;

mod error;
#[cfg(not(target_arch = "wasm32"))]
mod ffi_callbacks;
#[cfg(not(target_arch = "wasm32"))]
mod ffi_types;
mod options;
mod pusher;

// Re-exports
pub use channels::{Channel, ChannelType, MemberInfo, Members, PresenceChannel};
pub use connection::{ConnectionManager, ConnectionState};
pub use delta::{DeltaAlgorithm, DeltaManager, DeltaOptions, DeltaStats};
pub use error::{Result, SockudoError};
pub use events::{EventDispatcher, PusherEvent};
#[cfg(feature = "uniffi")]
pub use ffi_callbacks::{ChannelCallback, ConnectionCallback, EventCallback, PresenceCallback};
#[cfg(feature = "uniffi")]
pub use ffi_types::SockudoOptions as UniffiSockudoOptions;
#[cfg(feature = "uniffi")]
pub use ffi_types::{UniffiDeltaStats, UniffiMemberInfo, UniffiPusherEvent};
pub use options::{PusherOptions, SockudoOptions};
pub use protocol::{FilterOp, Protocol};
#[cfg(not(target_arch = "wasm32"))]
pub use pusher::{Pusher, SockudoClient};

// UniFFI setup for Kotlin/Swift bindings
#[cfg(feature = "uniffi")]
uniffi::setup_scaffolding!();

// WASM bindings
#[cfg(feature = "wasm")]
pub mod wasm;

// Flutter Rust Bridge bindings
#[cfg(feature = "flutter")]
pub mod flutter_api;
