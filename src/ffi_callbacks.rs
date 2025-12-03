//! FFI callback interfaces for UniFFI bindings

#![cfg(feature = "uniffi")]

use crate::connection::ConnectionState;
use crate::UniffiPusherEvent;

/// Callback for receiving events
#[cfg(feature = "uniffi")]
#[uniffi::export(callback_interface)]
pub trait EventCallback: Send + Sync {
    /// Called when an event is received
    fn on_event(&self, event: UniffiPusherEvent);
}

/// Callback for connection state changes
#[cfg(feature = "uniffi")]
#[uniffi::export(callback_interface)]
pub trait ConnectionCallback: Send + Sync {
    /// Called when connection state changes
    fn on_state_change(&self, previous: ConnectionState, current: ConnectionState);

    /// Called when a connection error occurs
    fn on_error(&self, error_type: String, message: String);
}

/// Callback for channel events
#[cfg(feature = "uniffi")]
#[uniffi::export(callback_interface)]
pub trait ChannelCallback: Send + Sync {
    /// Called when subscription succeeds
    fn on_subscription_succeeded(&self, data: Option<String>);

    /// Called when subscription fails
    fn on_subscription_error(&self, error_type: String, message: String);

    /// Called when an event is received on the channel
    fn on_event(&self, event_name: String, data: Option<String>, user_id: Option<String>);
}

/// Callback for presence channel events
#[cfg(feature = "uniffi")]
#[uniffi::export(callback_interface)]
pub trait PresenceCallback: Send + Sync {
    /// Called when subscription succeeds with initial member list
    fn on_subscription_succeeded(&self, my_id: String, member_ids: Vec<String>);

    /// Called when a member is added
    fn on_member_added(&self, user_id: String);

    /// Called when a member is removed
    fn on_member_removed(&self, user_id: String);
}
