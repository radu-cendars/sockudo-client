//! Presence channel implementation with member tracking.

use parking_lot::RwLock;
use std::sync::Arc;
use tracing::debug;

use super::channel::{
    AuthorizeFn, Channel, ChannelAuthData, ChannelState, ChannelType, SendEventFn,
};
use super::members::{MemberInfo, Members};
use crate::error::Result;
use crate::events::EventDispatcher;
use crate::protocol::PusherEvent;

/// Presence channel - private channel with member tracking
#[cfg_attr(feature = "uniffi", derive(uniffi::Object))]
pub struct PresenceChannel {
    /// Channel name
    name: String,
    /// Channel state (shared)
    state: Arc<RwLock<ChannelState>>,
    /// Event dispatcher
    dispatcher: EventDispatcher,
    /// Members management
    pub members: Members,
    /// Send event callback
    send_event: Option<SendEventFn>,
    /// Authorization callback
    authorize_fn: Option<AuthorizeFn>,
    /// Socket ID
    socket_id: RwLock<Option<String>>,
}

impl PresenceChannel {
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        assert!(
            name.starts_with("presence-"),
            "Presence channel name must start with 'presence-'"
        );

        Self {
            name: name.clone(),
            state: Arc::new(RwLock::new(ChannelState::Unsubscribed)),
            dispatcher: EventDispatcher::with_fail_through(move |event, _| {
                debug!("No callbacks on {} for {}", name, event);
            }),
            members: Members::new(),
            send_event: None,
            authorize_fn: None,
            socket_id: RwLock::new(None),
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

    /// Get channel name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get channel type
    pub fn channel_type(&self) -> ChannelType {
        ChannelType::Presence
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

    /// Get all members
    pub fn get_members(&self) -> Vec<MemberInfo> {
        self.members.all()
    }

    /// Get current user's member info
    pub fn get_me(&self) -> Option<MemberInfo> {
        self.members.me()
    }

    /// Get member count
    pub fn member_count(&self) -> usize {
        self.members.count()
    }

    /// Get a specific member
    pub fn get_member(&self, user_id: &str) -> Option<MemberInfo> {
        self.members.get(user_id)
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

    /// Authorize the subscription
    pub fn authorize(&self, socket_id: &str) -> Result<ChannelAuthData> {
        if let Some(ref auth_fn) = self.authorize_fn {
            let auth_data = auth_fn(&self.name, socket_id)?;

            // Extract user_id from channel_data
            if let Some(ref channel_data) = auth_data.channel_data {
                if let Ok(data) = serde_json::from_str::<serde_json::Value>(channel_data) {
                    if let Some(user_id) = data.get("user_id").and_then(|v| v.as_str()) {
                        self.members.set_my_id(user_id);
                    }
                }
            }

            Ok(auth_data)
        } else {
            Err(crate::error::SockudoError::authorization(
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
        let sub_data = serde_json::json!({
            "channel": self.name,
            "auth": auth_data.auth,
            "channel_data": auth_data.channel_data,
        });

        // Send subscribe event
        if let Some(ref send) = self.send_event {
            #[cfg(feature = "wasm")]
            send("pusher:subscribe", &sub_data, None);
            #[cfg(not(feature = "wasm"))]
            send("pusher:subscribe", &sub_data.to_string(), None);
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
        self.members.reset();
    }

    /// Trigger a client event (WASM version)
    #[cfg(feature = "wasm")]
    pub fn trigger(&self, event_name: &str, data: serde_json::Value) -> Result<bool> {
        if !event_name.starts_with("client-") {
            return Err(crate::error::SockudoError::invalid_event(format!(
                "Client events must start with 'client-', got: {}",
                event_name
            )));
        }

        if let Some(ref send) = self.send_event {
            Ok(send(event_name, &data, Some(&self.name)))
        } else {
            Err(crate::error::SockudoError::invalid_state(
                "No send callback configured",
            ))
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
        if !event_name.starts_with("client-") {
            return Err(crate::error::SockudoError::invalid_event(format!(
                "Client events must start with 'client-', got: {}",
                event_name
            )));
        }

        if let Some(ref send) = self.send_event {
            Ok(send(event_name, &data, Some(&self.name)))
        } else {
            Err(crate::error::SockudoError::invalid_state(
                "No send callback configured",
            ))
        }
    }

    /// Handle an incoming event
    pub fn handle_event(&self, event: &PusherEvent) {
        let event_name = &event.event;

        if event_name.starts_with("pusher_internal:") {
            self.handle_internal_event(event);
        } else {
            // User event - emit with user_id metadata
            self.dispatcher.emit(event);
        }
    }

    /// Handle internal events
    fn handle_internal_event(&self, event: &PusherEvent) {
        match event.event.as_str() {
            "pusher_internal:subscription_succeeded" => {
                self.handle_subscription_succeeded(event);
            }
            "pusher_internal:subscription_count" => {
                // Emit as pusher:subscription_count
                let mut count_event = event.clone();
                count_event.event = "pusher:subscription_count".to_string();
                self.dispatcher.emit(&count_event);
            }
            "pusher_internal:member_added" => {
                self.handle_member_added(event);
            }
            "pusher_internal:member_removed" => {
                self.handle_member_removed(event);
            }
            _ => {}
        }
    }

    /// Handle subscription succeeded
    fn handle_subscription_succeeded(&self, event: &PusherEvent) {
        *self.state.write() = ChannelState::Subscribed;

        // Initialize members from presence data
        if let Some(ref data) = event.data {
            #[cfg(feature = "wasm")]
            {
                self.members.on_subscription(data);
            }
            #[cfg(not(feature = "wasm"))]
            {
                if let Ok(value) = serde_json::from_str::<serde_json::Value>(data) {
                    self.members.on_subscription(&value);
                }
            }
        }

        // Emit as pusher:subscription_succeeded with members
        let mut success_event = PusherEvent::new("pusher:subscription_succeeded");
        success_event.channel = Some(self.name.clone());

        // Include members info in the event
        let members_data = serde_json::json!({
            "members": self.members.all(),
            "count": self.members.count(),
            "myID": self.members.my_id(),
        });

        #[cfg(feature = "wasm")]
        {
            success_event.data = Some(members_data);
        }
        #[cfg(not(feature = "wasm"))]
        {
            success_event.data = Some(members_data.to_string());
        }

        self.dispatcher.emit(&success_event);
    }

    /// Handle member added
    fn handle_member_added(&self, event: &PusherEvent) {
        if let Some(ref data) = event.data {
            #[cfg(feature = "wasm")]
            let member_opt = self.members.add_member(data);
            #[cfg(not(feature = "wasm"))]
            let member_opt = if let Ok(value) = serde_json::from_str::<serde_json::Value>(data) {
                self.members.add_member(&value)
            } else {
                None
            };

            if let Some(member) = member_opt {
                let mut added_event = PusherEvent::new("pusher:member_added");
                added_event.channel = Some(self.name.clone());

                #[cfg(feature = "wasm")]
                {
                    added_event.data = Some(serde_json::to_value(&member).unwrap());
                }
                #[cfg(not(feature = "wasm"))]
                {
                    added_event.data = Some(serde_json::to_string(&member).unwrap());
                }

                self.dispatcher.emit(&added_event);
            }
        }
    }

    /// Handle member removed
    fn handle_member_removed(&self, event: &PusherEvent) {
        if let Some(ref data) = event.data {
            #[cfg(feature = "wasm")]
            let member_opt = self.members.remove_member(data);
            #[cfg(not(feature = "wasm"))]
            let member_opt = if let Ok(value) = serde_json::from_str::<serde_json::Value>(data) {
                self.members.remove_member(&value)
            } else {
                None
            };

            if let Some(member) = member_opt {
                let mut removed_event = PusherEvent::new("pusher:member_removed");
                removed_event.channel = Some(self.name.clone());

                #[cfg(feature = "wasm")]
                {
                    removed_event.data = Some(serde_json::to_value(&member).unwrap());
                }
                #[cfg(not(feature = "wasm"))]
                {
                    removed_event.data = Some(serde_json::to_string(&member).unwrap());
                }

                self.dispatcher.emit(&removed_event);
            }
        }
    }

    /// Get as base Channel reference (for unified handling)
    pub fn as_channel(&self) -> Arc<Channel> {
        // Create a channel that shares the same dispatcher and state
        let mut channel =
            Channel::with_dispatcher(&self.name, self.dispatcher.clone(), self.state.clone());

        // Copy callbacks from presence channel
        if let Some(ref send_cb) = self.send_event {
            channel.set_send_callback(send_cb.clone());
        }
        if let Some(ref auth_cb) = self.authorize_fn {
            channel.set_authorize_callback(auth_cb.clone());
        }

        Arc::new(channel)
    }
}

impl std::fmt::Debug for PresenceChannel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PresenceChannel")
            .field("name", &self.name)
            .field("state", &*self.state.read())
            .field("member_count", &self.member_count())
            .finish()
    }
}

// FFI exports for PresenceChannel - only re-export methods that need FFI access
#[cfg(all(not(feature = "wasm"), feature = "uniffi"))]
#[uniffi::export]
impl PresenceChannel {
    /// Get the channel name (FFI wrapper)
    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    /// Get the channel type (FFI wrapper)
    pub fn get_channel_type(&self) -> ChannelType {
        ChannelType::Presence
    }

    /// Check if subscribed (FFI wrapper)
    pub fn get_is_subscribed(&self) -> bool {
        self.is_subscribed()
    }

    /// Check if subscription is pending (FFI wrapper)
    pub fn get_is_subscription_pending(&self) -> bool {
        self.is_subscription_pending()
    }

    /// Get all member IDs
    pub fn get_member_ids(&self) -> Vec<String> {
        self.members
            .all()
            .iter()
            .map(|m| m.user_id.clone())
            .collect()
    }

    /// Get current user's ID
    pub fn get_my_id(&self) -> Option<String> {
        self.members.my_id()
    }

    /// Get member count (FFI wrapper)
    pub fn get_member_count(&self) -> u32 {
        self.member_count() as u32
    }

    /// Get all members with their info (FFI wrapper)
    #[uniffi::method(name = "getMembers")]
    pub fn ffi_get_members(&self) -> Vec<crate::UniffiMemberInfo> {
        self.members
            .all()
            .iter()
            .map(|m| crate::UniffiMemberInfo {
                user_id: m.user_id.clone(),
                user_info_json: m.user_info.as_ref().map(|v| v.to_string()),
            })
            .collect()
    }

    /// Get specific member by user ID (FFI wrapper)
    #[uniffi::method(name = "getMember")]
    pub fn ffi_get_member(&self, user_id: String) -> Option<crate::UniffiMemberInfo> {
        self.members.get(&user_id).map(|m| crate::UniffiMemberInfo {
            user_id: m.user_id.clone(),
            user_info_json: m.user_info.as_ref().map(|v| v.to_string()),
        })
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
    fn test_presence_channel() {
        let channel = PresenceChannel::new("presence-room");
        assert_eq!(channel.name(), "presence-room");
        assert_eq!(channel.channel_type(), ChannelType::Presence);
        assert_eq!(channel.member_count(), 0);
    }

    #[test]
    fn test_member_tracking() {
        let channel = PresenceChannel::new("presence-room");

        // Simulate subscription succeeded
        let data = serde_json::json!({
            "presence": {
                "count": 2,
                "ids": ["user1", "user2"],
                "hash": {
                    "user1": {"name": "User One"},
                    "user2": {"name": "User Two"}
                }
            }
        });

        channel.members.on_subscription(&data);

        assert_eq!(channel.member_count(), 2);
        assert!(channel.get_member("user1").is_some());
    }

    #[test]
    #[should_panic]
    fn test_invalid_name() {
        PresenceChannel::new("private-channel");
    }
}
