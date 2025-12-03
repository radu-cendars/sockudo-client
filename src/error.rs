//! Error types for the Sockudo client library.

use thiserror::Error;

/// Result type alias for Sockudo operations
pub type Result<T> = std::result::Result<T, SockudoError>;

/// Main error type for the Sockudo client
#[derive(Error, Debug, Clone)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Error))]
#[cfg_attr(feature = "uniffi", uniffi(flat_error))]
pub enum SockudoError {
    #[error("Connection error: {message}")]
    ConnectionError { message: String },

    #[error("Authorization error: {message}")]
    AuthorizationError { message: String },

    #[error("Channel error: {message}")]
    ChannelError { message: String },

    #[error("Protocol error: {message}")]
    ProtocolError { message: String },

    #[error("Encryption error: {message}")]
    EncryptionError { message: String },

    #[error("Timeout error: {message}")]
    TimeoutError { message: String },

    #[error("Invalid state: {message}")]
    InvalidState { message: String },

    #[error("Invalid channel: {message}")]
    InvalidChannel { message: String },

    #[error("Invalid event: {message}")]
    InvalidEvent { message: String },

    #[error("WebSocket error: {message}")]
    WebSocketError { message: String },

    #[error("Configuration error: {message}")]
    ConfigurationError { message: String },

    #[error("Serialization error: {message}")]
    SerializationError { message: String },

    #[error("Delta compression error: {message}")]
    DeltaError { message: String },
}

impl SockudoError {
    pub fn connection(msg: impl Into<String>) -> Self {
        Self::ConnectionError {
            message: msg.into(),
        }
    }

    pub fn authorization(msg: impl Into<String>) -> Self {
        Self::AuthorizationError {
            message: msg.into(),
        }
    }

    pub fn channel(msg: impl Into<String>) -> Self {
        Self::ChannelError {
            message: msg.into(),
        }
    }

    pub fn protocol(msg: impl Into<String>) -> Self {
        Self::ProtocolError {
            message: msg.into(),
        }
    }

    pub fn encryption(msg: impl Into<String>) -> Self {
        Self::EncryptionError {
            message: msg.into(),
        }
    }

    pub fn timeout(msg: impl Into<String>) -> Self {
        Self::TimeoutError {
            message: msg.into(),
        }
    }

    pub fn invalid_state(msg: impl Into<String>) -> Self {
        Self::InvalidState {
            message: msg.into(),
        }
    }

    pub fn invalid_channel(msg: impl Into<String>) -> Self {
        Self::InvalidChannel {
            message: msg.into(),
        }
    }

    pub fn invalid_event(msg: impl Into<String>) -> Self {
        Self::InvalidEvent {
            message: msg.into(),
        }
    }

    pub fn websocket(msg: impl Into<String>) -> Self {
        Self::WebSocketError {
            message: msg.into(),
        }
    }

    pub fn config(msg: impl Into<String>) -> Self {
        Self::ConfigurationError {
            message: msg.into(),
        }
    }

    pub fn serialization(msg: impl Into<String>) -> Self {
        Self::SerializationError {
            message: msg.into(),
        }
    }

    pub fn delta(msg: impl Into<String>) -> Self {
        Self::DeltaError {
            message: msg.into(),
        }
    }
}

impl From<serde_json::Error> for SockudoError {
    fn from(err: serde_json::Error) -> Self {
        Self::serialization(err.to_string())
    }
}

impl From<url::ParseError> for SockudoError {
    fn from(err: url::ParseError) -> Self {
        Self::config(format!("Invalid URL: {}", err))
    }
}

impl From<base64::DecodeError> for SockudoError {
    fn from(err: base64::DecodeError) -> Self {
        Self::encryption(format!("Base64 decode error: {}", err))
    }
}

#[cfg(feature = "native")]
impl From<tokio_tungstenite::tungstenite::Error> for SockudoError {
    fn from(err: tokio_tungstenite::tungstenite::Error) -> Self {
        Self::websocket(format!("{:?}", err))
    }
}

// UniFFI compatibility - convert to simpler error for FFI
#[cfg(feature = "uniffi")]
impl From<SockudoError> for uniffi::UnexpectedUniFFICallbackError {
    fn from(err: SockudoError) -> Self {
        uniffi::UnexpectedUniFFICallbackError {
            reason: err.to_string(),
        }
    }
}
