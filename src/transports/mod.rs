//! Transport implementations for different platforms.
//!
//! This module provides WebSocket transport implementations for:
//! - Native platforms (using fast_websocket_client)
//! - WASM/Browser (using web-sys WebSocket API)
//!
//! The transport layer abstracts the platform-specific WebSocket implementation
//! and provides a common interface via the `Transport` trait.

mod transport;

pub use transport::{MessageCallback, Transport};

/// Native WebSocket transport (Tokio + fast_websocket_client)
#[cfg(feature = "native")]
pub mod native;

#[cfg(feature = "native")]
pub use native::NativeTransport;

/// WASM WebSocket transport (web-sys)
#[cfg(all(feature = "wasm", target_arch = "wasm32"))]
pub mod wasm;

#[cfg(all(feature = "wasm", target_arch = "wasm32"))]
pub use wasm::WasmTransport;

/// Create the default transport for the current platform
#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
pub fn create_default_transport() -> Box<dyn Transport> {
    Box::new(NativeTransport::new())
}

/// Create the default transport for the current platform
#[cfg(all(feature = "wasm", target_arch = "wasm32"))]
pub fn create_default_transport() -> Box<dyn Transport> {
    Box::new(WasmTransport::new())
}
