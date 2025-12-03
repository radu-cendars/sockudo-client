//! Base channel implementation.

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, warn};

use crate::error::{Result, SockudoError};
use crate::events::EventDispatcher;
use crate::protocol::{FilterOp, PusherEvent};

/// Channel type enumeration
#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChannelType {
    /// Public channel - no authentication required
    Public,
    /// Private channel - requires authentication
    Private,
    /// Presence channel - private with member tracking
    Presence,
    /// Private encrypted channel - end-to-end encryption
    PrivateEncrypted,
}

impl ChannelType {
    /// Determine channel type from name
    pub fn from_name(name: &str) -> Self {
        if name.starts_with("private-encrypted-") {
            Self::PrivateEncrypted
        } else if name.starts_with("private-") {
            Self::Private
        } else if name.starts_with("presence-") {
            Self::Presence
        } else {
            Self::Public
        }
    }

    /// Check if this channel type requires authentication
    pub fn requires_auth(&self) -> bool {
        matches!(
            self,
            Self::Private | Self::Presence | Self::PrivateEncrypted
        )
    }

    /// Check if this channel type supports client events
    pub fn supports_client_events(&self) -> bool {
        matches!(self, Self::Private | Self::Presence)
    }
}

/// Channel subscription state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelState {
    /// Initial state
    Unsubscribed,
    /// Subscription in progress
    Subscribing,
    /// Successfully subscribed
    Subscribed,
    /// Subscription failed
    Failed,
}

/// Callback for sending events
#[cfg(feature = "wasm")]
pub type SendEventFn = Arc<dyn Fn(&str, &serde_json::Value, Option<&str>) -> bool + Send + Sync>;

/// Callback for sending events (FFI-safe)
#[cfg(not(feature = "wasm"))]
pub type SendEventFn = Arc<dyn Fn(&str, &str, Option<&str>) -> bool + Send + Sync>;

/// Callback for channel authorization
pub type AuthorizeFn = Arc<dyn Fn(&str, &str) -> Result<ChannelAuthData> + Send + Sync>;

/// Channel authorization data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelAuthData {
    pub auth: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_data: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shared_secret: Option<String>,
}

/// Base channel implementation
#[cfg_attr(feature = "uniffi", derive(uniffi::Object))]
pub struct Channel {
    /// Channel name
    name: String,
    /// Channel type
    channel_type: ChannelType,
    /// Current state (shared)
    state: Arc<RwLock<ChannelState>>,
    /// Event dispatcher for this channel
    dispatcher: EventDispatcher,
    /// Optional tags filter for subscription
    tags_filter: RwLock<Option<FilterOp>>,
    /// Callback for sending events
    send_event: Option<SendEventFn>,
    /// Callback for authorization
    authorize_fn: Option<AuthorizeFn>,
    /// Socket ID (set when subscribing)
    socket_id: RwLock<Option<String>>,
    /// Subscription count (if available)
    subscription_count: RwLock<Option<u32>>,
}

impl Channel {
    /// Create a new channel
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        let channel_type = ChannelType::from_name(&name);

        Self {
            name: name.clone(),
            channel_type,
            state: Arc::new(RwLock::new(ChannelState::Unsubscribed)),
            dispatcher: EventDispatcher::with_fail_through(move |event, _| {
                debug!("No callbacks on {} for {}", name, event);
            }),
            tags_filter: RwLock::new(None),
            send_event: None,
            authorize_fn: None,
            socket_id: RwLock::new(None),
            subscription_count: RwLock::new(None),
        }
    }

    /// Create a new channel with a shared dispatcher and state
    pub fn with_dispatcher(
        name: impl Into<String>,
        dispatcher: EventDispatcher,
        state: Arc<RwLock<ChannelState>>,
    ) -> Self {
        let name = name.into();
        let channel_type = ChannelType::from_name(&name);

        Self {
            name,
            channel_type,
            state,
            dispatcher,
            tags_filter: RwLock::new(None),
            send_event: None,
            authorize_fn: None,
            socket_id: RwLock::new(None),
            subscription_count: RwLock::new(None),
        }
    }

    /// Set the send event callback
    pub fn set_send_callback(&mut self, callback: SendEventFn) {
        self.send_event = Some(callback);
    }

    /// Set the authorization callback
    pub fn set_authorize_callback(&mut self, callback: AuthorizeFn) {
        self.authorize_fn = Some(callback);
    }

    /// Set tags filter for subscription
    pub fn set_tags_filter(&self, filter: Option<FilterOp>) {
        *self.tags_filter.write() = filter;
    }

    /// Get channel name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get channel type
    pub fn channel_type(&self) -> ChannelType {
        self.channel_type
    }

    /// Check if subscribed
    pub fn is_subscribed(&self) -> bool {
        *self.state.read() == ChannelState::Subscribed
    }

    /// Check if subscription is pending
    pub fn is_subscription_pending(&self) -> bool {
        *self.state.read() == ChannelState::Subscribing
    }

    /// Get current state
    pub fn state(&self) -> ChannelState {
        *self.state.read()
    }

    /// Get subscription count
    pub fn subscription_count(&self) -> Option<u32> {
        *self.subscription_count.read()
    }

    /// Bind a callback to an event
    pub fn bind(
        &self,
        event_name: impl Into<String>,
        callback: impl Fn(&PusherEvent) + Send + Sync + 'static,
    ) -> u64 {
        self.dispatcher.bind(event_name, callback)
    }

    /// Unbind callbacks
    pub fn unbind(&self, event_name: Option<&str>, callback_id: Option<u64>) {
        self.dispatcher.unbind(event_name, callback_id);
    }

    /// Unbind all callbacks
    pub fn unbind_all(&self) {
        self.dispatcher.unbind_all();
    }

    /// Authorize the subscription (public channels skip authorization)
    pub fn authorize(&self, socket_id: &str) -> Result<ChannelAuthData> {
        if !self.channel_type.requires_auth() {
            // Public channels don't need auth
            return Ok(ChannelAuthData {
                auth: String::new(),
                channel_data: None,
                shared_secret: None,
            });
        }

        if let Some(ref auth_fn) = self.authorize_fn {
            auth_fn(&self.name, socket_id)
        } else {
            Err(SockudoError::authorization(
                "No authorization callback configured",
            ))
        }
    }

    /// Subscribe to the channel
    pub fn subscribe(&self, socket_id: &str) -> Result<()> {
        if self.is_subscribed() {
            return Ok(());
        }

        *self.state.write() = ChannelState::Subscribing;
        *self.socket_id.write() = Some(socket_id.to_string());

        // Authorize
        let auth_data = self.authorize(socket_id)?;

        // Build subscription data
        let mut sub_data = serde_json::json!({
            "channel": self.name,
        });

        if !auth_data.auth.is_empty() {
            sub_data["auth"] = serde_json::Value::String(auth_data.auth);
        }

        if let Some(ref cd) = auth_data.channel_data {
            sub_data["channel_data"] = serde_json::Value::String(cd.clone());
        }

        // Add tags filter if present
        if let Some(ref filter) = *self.tags_filter.read() {
            sub_data["tags_filter"] = filter.to_json();
        }

        // Send subscribe event
        if let Some(ref send) = self.send_event {
            #[cfg(feature = "wasm")]
            send("pusher:subscribe", &sub_data, None);
            #[cfg(not(feature = "wasm"))]
            send("pusher:subscribe", &sub_data.to_string(), None);
        }

        Ok(())
    }

    /// Subscribe to the channel asynchronously (for WASM/async contexts)
    #[cfg(target_arch = "wasm32")]
    pub async fn subscribe_async(
        &self,
        socket_id: &str,
        auth_endpoint: Option<&str>,
    ) -> Result<()> {
        if self.is_subscribed() {
            return Ok(());
        }

        *self.state.write() = ChannelState::Subscribing;
        *self.socket_id.write() = Some(socket_id.to_string());

        // Authorize asynchronously if needed
        let auth_data = if self.channel_type.requires_auth() {
            if let Some(endpoint) = auth_endpoint {
                use crate::auth::AuthClient;
                let auth_client = AuthClient::new(Some(endpoint.to_string()), None, None, None);
                auth_client.authorize_channel(&self.name, socket_id).await?
            } else {
                return Err(SockudoError::authorization(
                    "No auth_endpoint provided for private/presence channel",
                ));
            }
        } else {
            ChannelAuthData {
                auth: String::new(),
                channel_data: None,
                shared_secret: None,
            }
        };

        // Build subscription data
        let mut sub_data = serde_json::json!({
            "channel": self.name,
        });

        if !auth_data.auth.is_empty() {
            sub_data["auth"] = serde_json::Value::String(auth_data.auth);
        }

        if let Some(ref cd) = auth_data.channel_data {
            sub_data["channel_data"] = serde_json::Value::String(cd.clone());
        }

        // Add tags filter if present
        if let Some(ref filter) = *self.tags_filter.read() {
            sub_data["tags_filter"] = filter.to_json();
        }

        // Send subscribe event
        if let Some(ref send) = self.send_event {
            send("pusher:subscribe", &sub_data, None);
        }

        Ok(())
    }

    /// Unsubscribe from the channel
    pub fn unsubscribe(&self) {
        if !self.is_subscribed() && !self.is_subscription_pending() {
            return;
        }

        *self.state.write() = ChannelState::Unsubscribed;

        let data = serde_json::json!({
            "channel": self.name,
        });

        if let Some(ref send) = self.send_event {
            #[cfg(feature = "wasm")]
            send("pusher:unsubscribe", &data, None);
            #[cfg(not(feature = "wasm"))]
            send("pusher:unsubscribe", &data.to_string(), None);
        }
    }

    /// Handle disconnection
    pub fn disconnect(&self) {
        *self.state.write() = ChannelState::Unsubscribed;
    }

    /// Trigger a client event (WASM version)
    #[cfg(feature = "wasm")]
    pub fn trigger(&self, event_name: &str, data: serde_json::Value) -> Result<bool> {
        if !self.channel_type.supports_client_events() {
            return Err(SockudoError::invalid_event(
                "Client events are only supported on private and presence channels",
            ));
        }

        if !event_name.starts_with("client-") {
            return Err(SockudoError::invalid_event(format!(
                "Client events must start with 'client-', got: {}",
                event_name
            )));
        }

        if !self.is_subscribed() {
            warn!("Client event triggered before subscription succeeded");
        }

        if let Some(ref send) = self.send_event {
            Ok(send(event_name, &data, Some(&self.name)))
        } else {
            Err(SockudoError::invalid_state("No send callback configured"))
        }
    }

    /// Trigger a client event with JSON Value (Rust-friendly API)
    #[cfg(not(feature = "wasm"))]
    pub fn trigger_value(&self, event_name: &str, data: serde_json::Value) -> Result<bool> {
        self.trigger(event_name, data.to_string())
    }

    /// Trigger a client event (FFI version - takes String)
    #[cfg(not(feature = "wasm"))]
    pub fn trigger(&self, event_name: &str, data: String) -> Result<bool> {
        if !self.channel_type.supports_client_events() {
            return Err(SockudoError::invalid_event(
                "Client events are only supported on private and presence channels",
            ));
        }

        if !event_name.starts_with("client-") {
            return Err(SockudoError::invalid_event(format!(
                "Client events must start with 'client-', got: {}",
                event_name
            )));
        }

        if !self.is_subscribed() {
            warn!("Client event triggered before subscription succeeded");
        }

        if let Some(ref send) = self.send_event {
            Ok(send(event_name, &data, Some(&self.name)))
        } else {
            Err(SockudoError::invalid_state("No send callback configured"))
        }
    }

    /// Handle an incoming event
    pub fn handle_event(&self, event: &PusherEvent) {
        let event_name = &event.event;

        if event_name == "pusher_internal:subscription_succeeded" {
            self.handle_subscription_succeeded(event);
        } else if event_name == "pusher_internal:subscription_count" {
            self.handle_subscription_count(event);
        } else if !event_name.starts_with("pusher_internal:") {
            // User event - emit to callbacks
            self.dispatcher.emit(event);
        }
    }

    /// Handle subscription succeeded
    fn handle_subscription_succeeded(&self, event: &PusherEvent) {
        *self.state.write() = ChannelState::Subscribed;

        // Emit as pusher:subscription_succeeded
        let mut success_event = event.clone();
        success_event.event = "pusher:subscription_succeeded".to_string();
        self.dispatcher.emit(&success_event);
    }

    /// Handle subscription count
    fn handle_subscription_count(&self, event: &PusherEvent) {
        if let Some(ref data) = event.data {
            #[cfg(feature = "wasm")]
            let count_opt = data.get("subscription_count").and_then(|v| v.as_u64());

            #[cfg(not(feature = "wasm"))]
            let count_opt = serde_json::from_str::<serde_json::Value>(data)
                .ok()
                .and_then(|v| v.get("subscription_count").and_then(|c| c.as_u64()));

            if let Some(count) = count_opt {
                *self.subscription_count.write() = Some(count as u32);
            }
        }

        // Emit as pusher:subscription_count
        let mut count_event = event.clone();
        count_event.event = "pusher:subscription_count".to_string();
        self.dispatcher.emit(&count_event);
    }
}

impl std::fmt::Debug for Channel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Channel")
            .field("name", &self.name)
            .field("type", &self.channel_type)
            .field("state", &*self.state.read())
            .finish()
    }
}

// FFI exports for Channel - only re-export methods that need FFI access
#[cfg(all(not(feature = "wasm"), feature = "uniffi"))]
#[uniffi::export]
impl Channel {
    /// Get the channel name (FFI wrapper)
    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    /// Get the channel type (FFI wrapper)
    pub fn get_channel_type(&self) -> ChannelType {
        self.channel_type
    }

    /// Check if the channel is subscribed (FFI wrapper)
    pub fn get_is_subscribed(&self) -> bool {
        self.is_subscribed()
    }

    /// Check if subscription is pending (FFI wrapper)
    pub fn get_is_subscription_pending(&self) -> bool {
        self.is_subscription_pending()
    }

    /// Get the subscription count (FFI wrapper)
    pub fn get_subscription_count(&self) -> Option<u32> {
        self.subscription_count()
    }

    /// Bind an event callback (FFI wrapper)
    #[uniffi::method(name = "bind")]
    pub fn ffi_bind(
        &self,
        event_name: String,
        callback: Box<dyn crate::ffi_callbacks::EventCallback>,
    ) -> u64 {
        self.bind(event_name, move |event| {
            callback.on_event(crate::UniffiPusherEvent {
                event: event.event.clone(),
                channel: event.channel.clone(),
                data: event.data.clone(),
                user_id: event.user_id.clone(),
            });
        })
    }

    /// Unbind event callback(s) (FFI wrapper)
    #[uniffi::method(name = "unbind")]
    pub fn ffi_unbind(&self, event_name: Option<String>, callback_id: Option<u64>) {
        self.unbind(event_name.as_deref(), callback_id);
    }

    /// Unbind all event callbacks (FFI wrapper)
    #[uniffi::method(name = "unbindAll")]
    pub fn ffi_unbind_all(&self) {
        self.unbind_all();
    }

    /// Trigger a client event (FFI wrapper)
    /// Returns true if the event was sent successfully
    #[uniffi::method(name = "trigger")]
    pub fn ffi_trigger(&self, event_name: String, data: String) -> crate::Result<bool> {
        self.trigger(&event_name, data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_type_from_name() {
        assert_eq!(ChannelType::from_name("test"), ChannelType::Public);
        assert_eq!(ChannelType::from_name("private-test"), ChannelType::Private);
        assert_eq!(
            ChannelType::from_name("presence-test"),
            ChannelType::Presence
        );
        assert_eq!(
            ChannelType::from_name("private-encrypted-test"),
            ChannelType::PrivateEncrypted
        );
    }

    #[test]
    fn test_channel_creation() {
        let channel = Channel::new("test-channel");
        assert_eq!(channel.name(), "test-channel");
        assert_eq!(channel.channel_type(), ChannelType::Public);
        assert!(!channel.is_subscribed());
    }

    #[test]
    fn test_channel_bind() {
        let channel = Channel::new("test-channel");
        let counter = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let counter_clone = counter.clone();

        channel.bind("test-event", move |_| {
            counter_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        });

        let event = PusherEvent::new("test-event");
        channel.handle_event(&event);

        assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 1);
    }
}
