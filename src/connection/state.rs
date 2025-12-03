//! Connection state management.

use serde::{Deserialize, Serialize};

/// Connection state
#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConnectionState {
    /// Initial state, never transitioned to
    Initialized,
    /// Connection is being established
    Connecting,
    /// Connection has been fully established
    Connected,
    /// Requested disconnection
    Disconnected,
    /// Connection unavailable (no network, timeout)
    Unavailable,
    /// Connection strategy not supported
    Failed,
}

impl ConnectionState {
    /// Check if currently connecting or connected
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Connecting | Self::Connected)
    }

    /// Check if connected
    pub fn is_connected(&self) -> bool {
        matches!(self, Self::Connected)
    }

    /// Check if in a terminal state
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Disconnected | Self::Failed)
    }

    /// Check if should attempt reconnection
    pub fn should_reconnect(&self) -> bool {
        matches!(self, Self::Unavailable)
    }
}

impl Default for ConnectionState {
    fn default() -> Self {
        Self::Initialized
    }
}

impl std::fmt::Display for ConnectionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Initialized => write!(f, "initialized"),
            Self::Connecting => write!(f, "connecting"),
            Self::Connected => write!(f, "connected"),
            Self::Disconnected => write!(f, "disconnected"),
            Self::Unavailable => write!(f, "unavailable"),
            Self::Failed => write!(f, "failed"),
        }
    }
}
