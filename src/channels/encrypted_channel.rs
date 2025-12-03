//! End-to-end encrypted channel implementation.

use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use parking_lot::RwLock;
use std::sync::Arc;
use tracing::{debug, error, warn};

use super::channel::{
    AuthorizeFn, Channel, ChannelAuthData, ChannelState, ChannelType, SendEventFn,
};
use crate::error::{Result, SockudoError};
use crate::events::EventDispatcher;
use crate::protocol::PusherEvent;

/// Size constants for NaCl secretbox
const NONCE_LENGTH: usize = 24;
const KEY_LENGTH: usize = 32;

/// Encrypted channel - end-to-end encrypted private channel
pub struct EncryptedChannel {
    /// Channel name
    name: String,
    /// Channel state (shared)
    state: Arc<RwLock<ChannelState>>,
    /// Event dispatcher
    dispatcher: EventDispatcher,
    /// Encryption key (from auth endpoint)
    key: RwLock<Option<[u8; KEY_LENGTH]>>,
    /// Send event callback
    send_event: Option<SendEventFn>,
    /// Authorization callback
    authorize_fn: Option<AuthorizeFn>,
    /// Socket ID
    socket_id: RwLock<Option<String>>,
}

impl EncryptedChannel {
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        assert!(
            name.starts_with("private-encrypted-"),
            "Encrypted channel name must start with 'private-encrypted-'"
        );

        Self {
            name: name.clone(),
            state: Arc::new(RwLock::new(ChannelState::Unsubscribed)),
            dispatcher: EventDispatcher::with_fail_through(move |event, _| {
                debug!("No callbacks on {} for {}", name, event);
            }),
            key: RwLock::new(None),
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
        ChannelType::PrivateEncrypted
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

    /// Authorize the subscription and get encryption key
    pub fn authorize(&self, socket_id: &str) -> Result<ChannelAuthData> {
        if let Some(ref auth_fn) = self.authorize_fn {
            let auth_data = auth_fn(&self.name, socket_id)?;

            // Extract and store the shared secret
            if let Some(ref secret_b64) = auth_data.shared_secret {
                let secret_bytes = BASE64.decode(secret_b64).map_err(|e| {
                    SockudoError::encryption(format!("Invalid shared_secret: {}", e))
                })?;

                if secret_bytes.len() != KEY_LENGTH {
                    return Err(SockudoError::encryption(format!(
                        "shared_secret must be {} bytes, got {}",
                        KEY_LENGTH,
                        secret_bytes.len()
                    )));
                }

                let mut key = [0u8; KEY_LENGTH];
                key.copy_from_slice(&secret_bytes);
                *self.key.write() = Some(key);
            } else {
                return Err(SockudoError::encryption(
                    "No shared_secret in auth response for encrypted channel",
                ));
            }

            Ok(auth_data)
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

        // Authorize (this will set the encryption key)
        let auth_data = self.authorize(socket_id)?;

        // Build subscription data (don't send shared_secret to server)
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
        *self.key.write() = None;

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
        // Keep the key for reconnection
    }

    /// Client events are NOT supported on encrypted channels
    #[cfg(feature = "wasm")]
    pub fn trigger(&self, _event_name: &str, _data: serde_json::Value) -> Result<bool> {
        Err(SockudoError::invalid_event(
            "Client events are not supported on encrypted channels",
        ))
    }

    #[cfg(not(feature = "wasm"))]
    pub fn trigger(&self, _event_name: &str, _data: String) -> Result<bool> {
        Err(SockudoError::invalid_event(
            "Client events are not supported on encrypted channels",
        ))
    }

    /// Handle an incoming event
    pub fn handle_event(&self, event: &PusherEvent) {
        let event_name = &event.event;

        if event_name.starts_with("pusher_internal:") || event_name.starts_with("pusher:") {
            // Internal events are not encrypted
            self.handle_internal_event(event);
        } else {
            // User events are encrypted
            self.handle_encrypted_event(event);
        }
    }

    /// Handle internal events (not encrypted)
    fn handle_internal_event(&self, event: &PusherEvent) {
        match event.event.as_str() {
            "pusher_internal:subscription_succeeded" => {
                *self.state.write() = ChannelState::Subscribed;

                let mut success_event = event.clone();
                success_event.event = "pusher:subscription_succeeded".to_string();
                self.dispatcher.emit(&success_event);
            }
            "pusher_internal:subscription_count" => {
                let mut count_event = event.clone();
                count_event.event = "pusher:subscription_count".to_string();
                self.dispatcher.emit(&count_event);
            }
            _ => {}
        }
    }

    /// Handle encrypted events
    fn handle_encrypted_event(&self, event: &PusherEvent) {
        let key = match *self.key.read() {
            Some(k) => k,
            None => {
                warn!("Received encrypted event before key was retrieved");
                return;
            }
        };

        let data = match &event.data {
            Some(d) => d,
            None => {
                error!("Encrypted event has no data");
                return;
            }
        };

        // Parse data as JSON and extract ciphertext and nonce
        #[cfg(feature = "wasm")]
        let (ciphertext_b64, nonce_b64) = {
            let ciphertext = match data.get("ciphertext").and_then(|v| v.as_str()) {
                Some(c) => c,
                None => {
                    error!("Encrypted event missing ciphertext");
                    return;
                }
            };

            let nonce = match data.get("nonce").and_then(|v| v.as_str()) {
                Some(n) => n,
                None => {
                    error!("Encrypted event missing nonce");
                    return;
                }
            };

            (ciphertext, nonce)
        };

        #[cfg(not(feature = "wasm"))]
        let (ciphertext_b64, nonce_b64) = {
            let parsed: serde_json::Value = match serde_json::from_str(data) {
                Ok(v) => v,
                Err(e) => {
                    error!("Failed to parse encrypted event data: {}", e);
                    return;
                }
            };

            let ciphertext = match parsed.get("ciphertext").and_then(|v| v.as_str()) {
                Some(c) => c.to_string(),
                None => {
                    error!("Encrypted event missing ciphertext");
                    return;
                }
            };

            let nonce = match parsed.get("nonce").and_then(|v| v.as_str()) {
                Some(n) => n.to_string(),
                None => {
                    error!("Encrypted event missing nonce");
                    return;
                }
            };

            (ciphertext, nonce)
        };

        // Decode from base64
        let ciphertext = match BASE64.decode(ciphertext_b64) {
            Ok(c) => c,
            Err(e) => {
                error!("Failed to decode ciphertext: {}", e);
                return;
            }
        };

        let nonce_bytes = match BASE64.decode(nonce_b64) {
            Ok(n) => n,
            Err(e) => {
                error!("Failed to decode nonce: {}", e);
                return;
            }
        };

        if nonce_bytes.len() != NONCE_LENGTH {
            error!(
                "Invalid nonce length: {} (expected {})",
                nonce_bytes.len(),
                NONCE_LENGTH
            );
            return;
        }

        // Decrypt using NaCl secretbox
        let plaintext = match decrypt_secretbox(&ciphertext, &nonce_bytes, &key) {
            Ok(p) => p,
            Err(e) => {
                warn!(
                    "Failed to decrypt event: {}. Attempting to refresh key...",
                    e
                );

                // Try to get a new key
                if let Some(ref socket_id) = *self.socket_id.read() {
                    if let Ok(_) = self.authorize(socket_id) {
                        // Try decryption again with new key
                        let new_key = match *self.key.read() {
                            Some(k) => k,
                            None => return,
                        };

                        match decrypt_secretbox(&ciphertext, &nonce_bytes, &new_key) {
                            Ok(p) => p,
                            Err(_) => {
                                error!("Failed to decrypt event even with new key");
                                return;
                            }
                        }
                    } else {
                        error!("Failed to refresh encryption key");
                        return;
                    }
                } else {
                    return;
                }
            }
        };

        // Parse decrypted data
        let decrypted_data = match String::from_utf8(plaintext) {
            Ok(s) => {
                // Try to parse as JSON
                serde_json::from_str(&s).unwrap_or(serde_json::Value::String(s))
            }
            Err(_) => {
                error!("Decrypted data is not valid UTF-8");
                return;
            }
        };

        // Emit decrypted event
        let mut decrypted_event = PusherEvent::new(&event.event);
        decrypted_event.channel = event.channel.clone();

        #[cfg(feature = "wasm")]
        {
            decrypted_event.data = Some(decrypted_data);
        }
        #[cfg(not(feature = "wasm"))]
        {
            decrypted_event.data = Some(decrypted_data.to_string());
        }

        decrypted_event.user_id = event.user_id.clone();

        self.dispatcher.emit(&decrypted_event);
    }

    /// Get as base Channel reference
    pub fn as_channel(&self) -> Arc<Channel> {
        // Create a channel that shares the same dispatcher and state
        let mut channel =
            Channel::with_dispatcher(&self.name, self.dispatcher.clone(), self.state.clone());

        // Copy callbacks from encrypted channel
        if let Some(ref send_cb) = self.send_event {
            channel.set_send_callback(send_cb.clone());
        }
        if let Some(ref auth_cb) = self.authorize_fn {
            channel.set_authorize_callback(auth_cb.clone());
        }

        Arc::new(channel)
    }
}

/// Decrypt using NaCl secretbox (XSalsa20-Poly1305)
fn decrypt_secretbox(ciphertext: &[u8], nonce: &[u8], key: &[u8; KEY_LENGTH]) -> Result<Vec<u8>> {
    use nacl::aead::generic_array::GenericArray;
    use nacl::{aead::Aead, KeyInit};

    type SecretBoxCipher = nacl::XSalsa20Poly1305;

    let cipher = SecretBoxCipher::new(GenericArray::from_slice(key));
    let nonce = GenericArray::from_slice(nonce);

    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| SockudoError::encryption("Decryption failed"))
}

impl std::fmt::Debug for EncryptedChannel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EncryptedChannel")
            .field("name", &self.name)
            .field("state", &*self.state.read())
            .field("has_key", &self.key.read().is_some())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypted_channel() {
        let channel = EncryptedChannel::new("private-encrypted-test");
        assert_eq!(channel.name(), "private-encrypted-test");
        assert_eq!(channel.channel_type(), ChannelType::PrivateEncrypted);
    }

    #[test]
    #[should_panic]
    fn test_invalid_name() {
        EncryptedChannel::new("private-channel");
    }

    #[test]
    fn test_trigger_not_supported() {
        let channel = EncryptedChannel::new("private-encrypted-test");
        #[cfg(feature = "wasm")]
        let result = channel.trigger("client-event", serde_json::json!({}));
        #[cfg(not(feature = "wasm"))]
        let result = channel.trigger("client-event", serde_json::json!({}).to_string());
        assert!(result.is_err());
    }
}
