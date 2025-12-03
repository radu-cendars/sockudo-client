//! Pusher protocol message types and encoding/decoding.

use crate::error::{Result, SockudoError};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Pusher event message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PusherEvent {
    pub event: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel: Option<String>,
    #[cfg(feature = "wasm")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
    #[cfg(not(feature = "wasm"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
}

impl PusherEvent {
    pub fn new(event: impl Into<String>) -> Self {
        Self {
            event: event.into(),
            channel: None,
            data: None,
            user_id: None,
        }
    }

    /// Get data field as Value (for non-wasm builds)
    #[cfg(not(feature = "wasm"))]
    pub fn data_as_value(&self) -> Option<Value> {
        self.data
            .as_ref()
            .and_then(|s| serde_json::from_str(s).ok())
    }

    /// Get data field as Value (for wasm builds)
    #[cfg(feature = "wasm")]
    pub fn data_as_value(&self) -> Option<Value> {
        self.data.clone()
    }

    pub fn with_channel(mut self, channel: impl Into<String>) -> Self {
        self.channel = Some(channel.into());
        self
    }

    pub fn with_data(mut self, data: impl Serialize) -> Result<Self> {
        #[cfg(feature = "wasm")]
        {
            self.data = Some(serde_json::to_value(data)?);
        }
        #[cfg(not(feature = "wasm"))]
        {
            self.data = Some(serde_json::to_string(&data)?);
        }
        Ok(self)
    }

    pub fn with_json_data(mut self, data: Value) -> Self {
        #[cfg(feature = "wasm")]
        {
            self.data = Some(data);
        }
        #[cfg(not(feature = "wasm"))]
        {
            self.data = Some(data.to_string());
        }
        self
    }

    pub fn with_string_data(mut self, data: impl Into<String>) -> Self {
        #[cfg(feature = "wasm")]
        {
            self.data = Some(Value::String(data.into()));
        }
        #[cfg(not(feature = "wasm"))]
        {
            self.data = Some(data.into());
        }
        self
    }

    /// Check if this is an internal Pusher event
    pub fn is_internal(&self) -> bool {
        self.event.starts_with("pusher_internal:") || self.event.starts_with("pusher:")
    }

    /// Get data as a string
    pub fn data_as_string(&self) -> Option<String> {
        #[cfg(feature = "wasm")]
        {
            self.data.as_ref().map(|d| match d {
                Value::String(s) => s.clone(),
                _ => d.to_string(),
            })
        }
        #[cfg(not(feature = "wasm"))]
        {
            self.data.clone()
        }
    }

    /// Parse data as a specific type
    pub fn parse_data<T: for<'de> Deserialize<'de>>(&self) -> Result<T> {
        let data = self
            .data
            .as_ref()
            .ok_or_else(|| SockudoError::protocol("No data in event"))?;

        #[cfg(feature = "wasm")]
        {
            // If data is a string, try to parse it as JSON first
            let value = if let Value::String(s) = data {
                serde_json::from_str(s).unwrap_or_else(|_| data.clone())
            } else {
                data.clone()
            };
            serde_json::from_value(value).map_err(Into::into)
        }
        #[cfg(not(feature = "wasm"))]
        {
            serde_json::from_str(data).map_err(Into::into)
        }
    }
}

/// Connection established event data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionEstablished {
    pub socket_id: String,
    #[serde(default)]
    pub activity_timeout: Option<u64>,
}

/// Subscription succeeded event data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionSucceeded {
    #[serde(default)]
    pub presence: Option<PresenceData>,
}

/// Presence channel data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(all(not(feature = "wasm"), feature = "uniffi"), derive(uniffi::Record))]
pub struct PresenceData {
    pub count: u32,
    pub ids: Vec<String>,
    #[cfg(feature = "wasm")]
    pub hash: std::collections::HashMap<String, Value>,
    #[cfg(not(feature = "wasm"))]
    pub hash: std::collections::HashMap<String, String>,
}

/// Subscription count event data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionCount {
    pub subscription_count: u32,
}

/// Member added/removed event data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberData {
    pub user_id: String,
    #[cfg(feature = "wasm")]
    #[serde(default)]
    pub user_info: Option<Value>,
    #[cfg(not(feature = "wasm"))]
    #[serde(default)]
    pub user_info: Option<String>,
}

/// Error event data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorData {
    pub message: String,
    #[serde(default)]
    pub code: Option<i32>,
}

/// Channel data for subscription
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelData {
    pub user_id: String,
    #[cfg(feature = "wasm")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_info: Option<Value>,
    #[cfg(not(feature = "wasm"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_info: Option<String>,
}

/// Subscribe message data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscribeData {
    pub channel: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_data: Option<String>,
    #[cfg(feature = "wasm")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags_filter: Option<Value>,
    #[cfg(not(feature = "wasm"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags_filter: Option<String>,
}

/// Unsubscribe message data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnsubscribeData {
    pub channel: String,
}

/// Protocol encoder/decoder
pub struct Protocol;

impl Protocol {
    /// Encode a message to JSON string
    pub fn encode_message(event: &PusherEvent) -> Result<String> {
        serde_json::to_string(event).map_err(Into::into)
    }

    /// Decode a message from JSON string
    pub fn decode_message(raw: &str) -> Result<PusherEvent> {
        serde_json::from_str(raw).map_err(Into::into)
    }

    /// Process handshake response
    pub fn process_handshake(event: &PusherEvent) -> Result<HandshakeResult> {
        match event.event.as_str() {
            "pusher:connection_established" => {
                let data: ConnectionEstablished = event.parse_data()?;
                Ok(HandshakeResult::Connected {
                    socket_id: data.socket_id,
                    activity_timeout: data.activity_timeout,
                })
            }
            "pusher:error" => {
                let data: ErrorData = event.parse_data()?;
                let action = Self::get_error_action(data.code);
                Ok(HandshakeResult::Error {
                    action,
                    message: data.message,
                    code: data.code,
                })
            }
            _ => Err(SockudoError::protocol(format!(
                "Unexpected handshake event: {}",
                event.event
            ))),
        }
    }

    /// Get action based on close code
    pub fn get_close_action(code: Option<u16>) -> CloseAction {
        match code {
            Some(1000) => CloseAction::Connected,
            Some(4000) => CloseAction::TlsOnly,
            Some(4001) => CloseAction::Refused,
            Some(4002) => CloseAction::Refused,
            Some(4003) => CloseAction::Refused,
            Some(4004) => CloseAction::Refused,
            Some(4100) => CloseAction::Backoff,
            Some(4200) => CloseAction::Retry,
            Some(4201) => CloseAction::Backoff,
            Some(4202) => CloseAction::Retry,
            Some(4300) => CloseAction::Retry,
            _ => CloseAction::Retry,
        }
    }

    /// Get action based on error code
    fn get_error_action(code: Option<i32>) -> CloseAction {
        match code {
            Some(4000) => CloseAction::TlsOnly,
            Some(4001) => CloseAction::Refused,
            Some(4002) => CloseAction::Refused,
            Some(4003) => CloseAction::Refused,
            Some(4004) => CloseAction::Refused,
            Some(4100) => CloseAction::Backoff,
            _ => CloseAction::Retry,
        }
    }

    /// Create a subscribe event
    #[cfg(feature = "wasm")]
    pub fn create_subscribe_event(
        channel: &str,
        auth: Option<String>,
        channel_data: Option<String>,
        tags_filter: Option<Value>,
    ) -> PusherEvent {
        let data = SubscribeData {
            channel: channel.to_string(),
            auth,
            channel_data,
            tags_filter,
        };

        PusherEvent::new("pusher:subscribe").with_json_data(serde_json::to_value(data).unwrap())
    }

    /// Create a subscribe event
    #[cfg(not(feature = "wasm"))]
    pub fn create_subscribe_event(
        channel: &str,
        auth: Option<String>,
        channel_data: Option<String>,
        tags_filter: Option<String>,
    ) -> PusherEvent {
        let data = SubscribeData {
            channel: channel.to_string(),
            auth,
            channel_data,
            tags_filter,
        };

        PusherEvent::new("pusher:subscribe").with_json_data(serde_json::to_value(data).unwrap())
    }

    /// Create an unsubscribe event
    pub fn create_unsubscribe_event(channel: &str) -> PusherEvent {
        let data = UnsubscribeData {
            channel: channel.to_string(),
        };

        PusherEvent::new("pusher:unsubscribe").with_json_data(serde_json::to_value(data).unwrap())
    }

    /// Create a ping event
    pub fn create_ping_event() -> PusherEvent {
        PusherEvent::new("pusher:ping").with_json_data(serde_json::json!({}))
    }

    /// Create a pong event
    pub fn create_pong_event() -> PusherEvent {
        PusherEvent::new("pusher:pong").with_json_data(serde_json::json!({}))
    }

    /// Create a client event
    pub fn create_client_event(
        event_name: &str,
        channel: &str,
        data: Value,
    ) -> Result<PusherEvent> {
        if !event_name.starts_with("client-") {
            return Err(SockudoError::invalid_event(format!(
                "Client events must start with 'client-', got: {}",
                event_name
            )));
        }

        Ok(PusherEvent::new(event_name)
            .with_channel(channel)
            .with_json_data(data))
    }
}

/// Result of handshake processing
#[derive(Debug, Clone)]
pub enum HandshakeResult {
    Connected {
        socket_id: String,
        activity_timeout: Option<u64>,
    },
    Error {
        action: CloseAction,
        message: String,
        code: Option<i32>,
    },
}

/// Action to take after connection close
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CloseAction {
    /// Connection was successful
    Connected,
    /// Must use TLS
    TlsOnly,
    /// Connection refused, don't retry
    Refused,
    /// Backoff and retry later
    Backoff,
    /// Retry immediately
    Retry,
}

/// Pusher protocol version
pub const PROTOCOL_VERSION: u8 = 7;

/// Client identifier
pub const CLIENT_NAME: &str = "sockudo-client-rust";

/// Client version
pub const CLIENT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_connection_established() {
        let raw = r#"{"event":"pusher:connection_established","data":"{\"socket_id\":\"123.456\",\"activity_timeout\":120}"}"#;
        let event = Protocol::decode_message(raw).unwrap();
        assert_eq!(event.event, "pusher:connection_established");

        let result = Protocol::process_handshake(&event).unwrap();
        match result {
            HandshakeResult::Connected {
                socket_id,
                activity_timeout,
            } => {
                assert_eq!(socket_id, "123.456");
                assert_eq!(activity_timeout, Some(120));
            }
            _ => panic!("Expected Connected result"),
        }
    }

    #[test]
    fn test_encode_subscribe() {
        let event = Protocol::create_subscribe_event(
            "test-channel",
            Some("auth-token".to_string()),
            None,
            None,
        );
        let json = Protocol::encode_message(&event).unwrap();
        assert!(json.contains("pusher:subscribe"));
        assert!(json.contains("test-channel"));
    }
}
