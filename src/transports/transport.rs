//! Transport trait definition.

use crate::error::Result;
use async_trait::async_trait;

/// Callback for message events
#[cfg(not(target_arch = "wasm32"))]
pub type MessageCallback = Box<dyn Fn(&str) + Send + Sync>;

#[cfg(target_arch = "wasm32")]
pub type MessageCallback = Box<dyn Fn(&str)>;

/// Transport trait for WebSocket connections
#[async_trait]
#[cfg(not(target_arch = "wasm32"))]
pub trait Transport: Send + Sync {
    /// Connect to the WebSocket server
    async fn connect(&mut self, url: &str) -> Result<()>;

    /// Disconnect from the server
    async fn disconnect(&mut self);

    /// Send a text message
    async fn send(&self, message: &str) -> Result<()>;

    /// Send a ping
    async fn ping(&self) -> Result<()>;

    /// Check if connected
    fn is_connected(&self) -> bool;

    /// Set message callback
    fn on_message(&mut self, callback: MessageCallback);

    /// Set close callback
    fn on_close(&mut self, callback: Box<dyn Fn(Option<u16>, Option<String>) + Send + Sync>);

    /// Set error callback
    fn on_error(&mut self, callback: Box<dyn Fn(String) + Send + Sync>);
}

/// Transport trait for WebSocket connections (WASM version without Send+Sync)
#[async_trait(?Send)]
#[cfg(target_arch = "wasm32")]
pub trait Transport {
    /// Connect to the WebSocket server
    async fn connect(&mut self, url: &str) -> Result<()>;

    /// Disconnect from the server
    async fn disconnect(&mut self);

    /// Send a text message
    async fn send(&self, message: &str) -> Result<()>;

    /// Send a ping
    async fn ping(&self) -> Result<()>;

    /// Check if connected
    fn is_connected(&self) -> bool;

    /// Set message callback
    fn on_message(&mut self, callback: MessageCallback);

    /// Set close callback
    fn on_close(&mut self, callback: Box<dyn Fn(Option<u16>, Option<String>)>);

    /// Set error callback
    fn on_error(&mut self, callback: Box<dyn Fn(String)>);
}
