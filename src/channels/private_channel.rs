//! Private channel implementation.

use super::channel::{AuthorizeFn, Channel, ChannelAuthData, ChannelType, SendEventFn};
use crate::error::Result;
use crate::protocol::PusherEvent;
use std::sync::Arc;

/// Private channel - requires authorization
pub struct PrivateChannel {
    inner: Channel,
}

impl PrivateChannel {
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        assert!(
            name.starts_with("private-"),
            "Private channel name must start with 'private-'"
        );

        Self {
            inner: Channel::new(name),
        }
    }

    /// Set the send event callback
    pub fn set_send_callback(&mut self, callback: SendEventFn) {
        self.inner.set_send_callback(callback);
    }

    /// Set the authorization callback
    pub fn set_authorize_callback(&mut self, callback: AuthorizeFn) {
        self.inner.set_authorize_callback(callback);
    }

    /// Get channel name
    pub fn name(&self) -> &str {
        self.inner.name()
    }

    /// Get channel type
    pub fn channel_type(&self) -> ChannelType {
        ChannelType::Private
    }

    /// Check if subscribed
    pub fn is_subscribed(&self) -> bool {
        self.inner.is_subscribed()
    }

    /// Check if subscription is pending
    pub fn is_subscription_pending(&self) -> bool {
        self.inner.is_subscription_pending()
    }

    /// Bind a callback to an event
    pub fn bind(
        &self,
        event_name: impl Into<String>,
        callback: impl Fn(&PusherEvent) + Send + Sync + 'static,
    ) -> u64 {
        self.inner.bind(event_name, callback)
    }

    /// Unbind callbacks
    pub fn unbind(&self, event_name: Option<&str>, callback_id: Option<u64>) {
        self.inner.unbind(event_name, callback_id);
    }

    /// Unbind all callbacks
    pub fn unbind_all(&self) {
        self.inner.unbind_all();
    }

    /// Authorize the subscription
    pub fn authorize(&self, socket_id: &str) -> Result<ChannelAuthData> {
        self.inner.authorize(socket_id)
    }

    /// Subscribe to the channel
    pub fn subscribe(&self, socket_id: &str) -> Result<()> {
        self.inner.subscribe(socket_id)
    }

    /// Unsubscribe from the channel
    pub fn unsubscribe(&self) {
        self.inner.unsubscribe();
    }

    /// Handle disconnection
    pub fn disconnect(&self) {
        self.inner.disconnect();
    }

    /// Trigger a client event (WASM version)
    #[cfg(feature = "wasm")]
    pub fn trigger(&self, event_name: &str, data: serde_json::Value) -> Result<bool> {
        self.inner.trigger(event_name, data)
    }

    /// Trigger a client event (FFI version)
    #[cfg(not(feature = "wasm"))]
    pub fn trigger(&self, event_name: &str, data: String) -> Result<bool> {
        self.inner.trigger(event_name, data)
    }

    /// Handle an incoming event
    pub fn handle_event(&self, event: &PusherEvent) {
        self.inner.handle_event(event);
    }

    /// Get as base Channel reference
    pub fn as_channel(&self) -> Arc<Channel> {
        // This is a workaround - in a real implementation we'd use
        // proper trait objects or enums
        Arc::new(Channel::new(self.name()))
    }
}

impl std::fmt::Debug for PrivateChannel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PrivateChannel")
            .field("name", &self.name())
            .field("subscribed", &self.is_subscribed())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_private_channel() {
        let channel = PrivateChannel::new("private-test");
        assert_eq!(channel.name(), "private-test");
        assert_eq!(channel.channel_type(), ChannelType::Private);
    }

    #[test]
    #[should_panic]
    fn test_invalid_name() {
        PrivateChannel::new("test-channel");
    }
}
