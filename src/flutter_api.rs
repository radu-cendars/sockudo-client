//! Flutter Rust Bridge API
//!
//! This module provides Flutter/Dart bindings for the Sockudo client library
//! using flutter_rust_bridge.

#![allow(unexpected_cfgs)]

use flutter_rust_bridge::frb;
use std::sync::Arc;

use crate::delta::DeltaAlgorithm;
use crate::{Result, SockudoClient as CoreClient};

#[cfg(feature = "uniffi")]
use crate::ffi_types::{SockudoOptions as CoreOptions, UniffiDeltaOptions as CoreDeltaOptions};

#[cfg(not(feature = "uniffi"))]
use crate::delta::DeltaOptions as CoreDeltaOptions;

#[cfg(not(feature = "uniffi"))]
use crate::SockudoOptions as CoreOptions;

// ============================================================================
// Configuration Types
// ============================================================================

/// Delta compression options for Flutter
#[frb(dart_metadata=("freezed"), dart_type = "DeltaOptions")]
#[derive(Clone)]
pub struct FlutterDeltaOptions {
    pub enabled: bool,
    pub algorithms: Vec<String>,
    pub debug: bool,
    pub max_messages_per_key: u32,
}

impl From<FlutterDeltaOptions> for CoreDeltaOptions {
    fn from(opts: FlutterDeltaOptions) -> Self {
        let algorithms: Vec<DeltaAlgorithm> = opts
            .algorithms
            .iter()
            .filter_map(|a| a.parse().ok())
            .collect();

        let algorithms = if algorithms.is_empty() {
            vec![DeltaAlgorithm::Fossil, DeltaAlgorithm::Xdelta3]
        } else {
            algorithms
        };

        #[cfg(feature = "uniffi")]
        {
            CoreDeltaOptions {
                enabled: opts.enabled,
                algorithms,
                debug: opts.debug,
                max_messages_per_key: opts.max_messages_per_key,
            }
        }

        #[cfg(not(feature = "uniffi"))]
        {
            CoreDeltaOptions {
                enabled: opts.enabled,
                algorithms,
                debug: opts.debug,
                max_messages_per_key: opts.max_messages_per_key as usize,
                on_stats: None,
                on_error: None,
            }
        }
    }
}

/// Sockudo configuration options for Flutter
#[frb(dart_metadata=("freezed"), dart_type = "SockudoOptions")]
pub struct FlutterSockudoOptions {
    pub app_key: String,
    pub cluster: Option<String>,
    pub ws_host: Option<String>,
    pub ws_port: Option<u16>,
    pub use_tls: Option<bool>,
    pub auth_endpoint: Option<String>,
    pub activity_timeout_ms: Option<u64>,
    pub pong_timeout_ms: Option<u64>,
    pub unavailable_timeout_ms: Option<u64>,
    pub delta_compression: Option<FlutterDeltaOptions>,
    pub enable_stats: Option<bool>,
    pub debug: Option<bool>,
    pub user_auth_endpoint: Option<String>,
    pub disable_reconnection: Option<bool>,
    pub max_reconnection_attempts: Option<u32>,
    pub reconnection_delay_ms: Option<u64>,
    pub max_reconnection_delay_ms: Option<u64>,
}

#[cfg(feature = "uniffi")]
impl From<FlutterSockudoOptions> for CoreOptions {
    fn from(opts: FlutterSockudoOptions) -> Self {
        CoreOptions {
            app_key: opts.app_key,
            cluster: opts.cluster,
            ws_host: opts.ws_host,
            ws_port: opts.ws_port,
            use_tls: opts.use_tls,
            auth_endpoint: opts.auth_endpoint,
            activity_timeout_ms: opts.activity_timeout_ms,
            pong_timeout_ms: opts.pong_timeout_ms,
            unavailable_timeout_ms: opts.unavailable_timeout_ms,
            delta_compression: opts.delta_compression.map(|d| d.into()),
            enable_stats: opts.enable_stats,
            debug: opts.debug,
            user_auth_endpoint: opts.user_auth_endpoint,
            disable_reconnection: opts.disable_reconnection,
            max_reconnection_attempts: opts.max_reconnection_attempts,
            reconnection_delay_ms: opts.reconnection_delay_ms,
            max_reconnection_delay_ms: opts.max_reconnection_delay_ms,
        }
    }
}

#[cfg(not(feature = "uniffi"))]
impl From<FlutterSockudoOptions> for CoreOptions {
    fn from(opts: FlutterSockudoOptions) -> Self {
        CoreOptions {
            app_key: opts.app_key,
            cluster: opts.cluster,
            ws_host: opts.ws_host,
            ws_port: opts.ws_port,
            use_tls: opts.use_tls,
            auth_endpoint: opts.auth_endpoint,
            auth_headers: None,
            activity_timeout_ms: opts.activity_timeout_ms,
            pong_timeout_ms: opts.pong_timeout_ms,
            unavailable_timeout_ms: opts.unavailable_timeout_ms,
            delta_compression: opts.delta_compression.map(|d| d.into()),
            enable_stats: opts.enable_stats,
            debug: opts.debug,
            user_auth_endpoint: opts.user_auth_endpoint,
            user_auth_headers: None,
            disable_reconnection: opts.disable_reconnection,
            max_reconnection_attempts: opts.max_reconnection_attempts,
            reconnection_delay_ms: opts.reconnection_delay_ms,
            max_reconnection_delay_ms: opts.max_reconnection_delay_ms,
        }
    }
}

// ============================================================================
// Event Types
// ============================================================================

/// Pusher event for Flutter
#[frb(dart_metadata=("freezed"), dart_type = "PusherEvent")]
#[derive(Clone, Debug)]
pub struct FlutterPusherEvent {
    pub event: String,
    pub channel: Option<String>,
    pub data: Option<String>,
    pub user_id: Option<String>,
}

/// Delta compression statistics
#[frb(dart_metadata=("freezed"), dart_type = "DeltaStats")]
#[derive(Clone, Debug)]
pub struct FlutterDeltaStats {
    pub total_messages: u64,
    pub delta_messages: u64,
    pub full_messages: u64,
    pub total_bytes_without_compression: u64,
    pub total_bytes_with_compression: u64,
    pub bandwidth_saved_percent: f64,
    pub errors: u64,
}

/// Member information for presence channels
#[frb(dart_metadata=("freezed"), dart_type = "MemberInfo")]
#[derive(Clone, Debug)]
pub struct FlutterMemberInfo {
    pub user_id: String,
    pub user_info: Option<String>,
}

// ============================================================================
// Main Client
// ============================================================================

/// Sockudo Client for Flutter
#[frb(opaque, dart_type = "SockudoClient")]
pub struct FlutterSockudoClient {
    inner: Arc<CoreClient>,
}

impl FlutterSockudoClient {
    /// Create a new Sockudo client (Pusher-JS compatible: auto-connects)
    ///
    /// # Example (Dart)
    /// ```dart
    /// final client = SockudoClient(
    ///   appKey: "your-app-key",
    ///   options: PusherOptions(cluster: "mt1"),
    /// );
    /// ```
    #[frb(sync)]
    pub fn new(options: FlutterSockudoOptions) -> Result<Self> {
        let opts = options;

        #[cfg(feature = "uniffi")]
        {
            let core_options: CoreOptions = opts.into();
            let client = CoreClient::new(core_options)?;
            Ok(Self {
                inner: Arc::new(client),
            })
        }

        #[cfg(not(feature = "uniffi"))]
        {
            let core_options: CoreOptions = opts.into();
            let client = CoreClient::from_options(core_options)?;

            // Auto-connect in background
            if let Ok(handle) = tokio::runtime::Handle::try_current() {
                let conn = client.connection.clone();
                handle.spawn(async move {
                    let _ = conn.connect().await;
                });
            }

            Ok(Self {
                inner: Arc::new(client),
            })
        }
    }

    /// Connect to the Pusher server
    pub async fn connect(&self) -> Result<()> {
        self.inner.connect().await
    }

    /// Disconnect from the server
    pub async fn disconnect(&self) {
        self.inner.disconnect().await;
    }

    /// Get current connection state
    #[frb(sync)]
    pub fn get_state(&self) -> String {
        format!("{:?}", self.inner.state())
    }

    /// Get socket ID
    #[frb(sync)]
    pub fn get_socket_id(&self) -> Option<String> {
        self.inner.socket_id()
    }

    /// Check if connected
    #[frb(sync)]
    pub fn is_connected(&self) -> bool {
        self.inner.is_connected()
    }

    /// Subscribe to a channel
    #[frb(sync)]
    pub fn subscribe(&self, channel_name: String) -> Result<FlutterChannel> {
        self.inner.subscribe(&channel_name)?;
        Ok(FlutterChannel {
            name: channel_name,
            client: self.inner.clone(),
        })
    }

    /// Unsubscribe from a channel
    #[frb(sync)]
    pub fn unsubscribe(&self, channel_name: String) {
        self.inner.unsubscribe(&channel_name);
    }

    /// Get delta compression statistics
    #[frb(sync)]
    pub fn get_delta_stats(&self) -> Option<FlutterDeltaStats> {
        self.inner.get_delta_stats().map(|stats| FlutterDeltaStats {
            total_messages: stats.total_messages,
            delta_messages: stats.delta_messages,
            full_messages: stats.full_messages,
            total_bytes_without_compression: stats.total_bytes_without_compression,
            total_bytes_with_compression: stats.total_bytes_with_compression,
            bandwidth_saved_percent: stats.bandwidth_saved_percent,
            errors: stats.errors,
        })
    }

    /// Reset delta compression statistics
    #[frb(sync)]
    pub fn reset_delta_stats(&self) {
        self.inner.reset_delta_stats();
    }
}

// ============================================================================
// Channel
// ============================================================================

/// Channel for Flutter
#[frb(opaque, dart_type = "Channel")]
pub struct FlutterChannel {
    name: String,
    client: Arc<CoreClient>,
}

impl FlutterChannel {
    /// Get channel name
    #[frb(sync)]
    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    /// Check if channel is subscribed
    #[frb(sync)]
    pub fn is_subscribed(&self) -> bool {
        self.client
            .channel(&self.name)
            .map(|ch| ch.is_subscribed())
            .unwrap_or(false)
    }

    /// Trigger a client event (for private/presence channels)
    #[frb(sync)]
    pub fn trigger(&self, event_name: String, data: String) -> Result<bool> {
        Ok(self
            .client
            .send_event(event_name, data, Some(self.name.clone())))
    }

    /// Unbind all events from this channel
    #[frb(sync)]
    pub fn unbind_all(&self) {
        if let Some(channel) = self.client.channel(&self.name) {
            channel.unbind_all();
        }
    }
}

// ============================================================================
// Stream API for Events
// ============================================================================
// Note: Stream functions are commented out as they require additional setup
// and are not supported in the current flutter_rust_bridge version being used.
// Uncomment and implement when upgrading to a version that supports #[frb(stream)]

// /// Create a stream of connection state changes
// pub async fn connection_state_stream(
//     client: FlutterSockudoClient,
// ) -> impl futures::Stream<Item = String> {
//     // This is a simplified implementation
//     // In a real implementation, you'd want to hook into the actual connection state changes
//     futures::stream::iter(vec!["Connected".to_string()])
// }
//
// /// Create a stream of events for a specific channel
// pub async fn channel_event_stream(
//     client: FlutterSockudoClient,
//     channel_name: String,
// ) -> impl futures::Stream<Item = FlutterPusherEvent> {
//     // This is a simplified implementation
//     // In a real implementation, you'd want to hook into the actual event dispatcher
//     futures::stream::iter(vec![])
// }

// ============================================================================
// Utility Functions
// ============================================================================

/// Initialize logging for the Rust library
#[frb(sync)]
pub fn init_logging(level: String) {
    // Logging initialization - simplified for Flutter
    // In production, you might want to use a different logging approach
    log::set_max_level(match level.as_str() {
        "trace" => log::LevelFilter::Trace,
        "debug" => log::LevelFilter::Debug,
        "info" => log::LevelFilter::Info,
        "warn" => log::LevelFilter::Warn,
        "error" => log::LevelFilter::Error,
        _ => log::LevelFilter::Info,
    });
}

/// Get library version
#[frb(sync)]
pub fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

// ============================================================================
// Error Handling
// ============================================================================

// flutter_rust_bridge automatically handles Result types and converts them
// to Dart exceptions. The SockudoError type will be converted automatically.
