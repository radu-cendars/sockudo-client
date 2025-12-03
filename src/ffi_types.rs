//! Simplified FFI-friendly types for UniFFI bindings
//!
//! This module contains simplified versions of internal types that are designed
//! to work seamlessly with UniFFI's foreign function interface constraints.

#![cfg(feature = "uniffi")]

use crate::delta::DeltaAlgorithm;

/// UniFFI-friendly delta compression options
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[cfg_attr(feature = "uniffi", uniffi(name = "DeltaOptions"))]
#[derive(Clone, Default)]
pub struct UniffiDeltaOptions {
    pub enabled: bool,
    pub algorithms: Vec<DeltaAlgorithm>,
    pub debug: bool,
    pub max_messages_per_key: u32,
}

impl From<UniffiDeltaOptions> for crate::delta::DeltaOptions {
    fn from(opts: UniffiDeltaOptions) -> Self {
        crate::delta::DeltaOptions {
            enabled: opts.enabled,
            algorithms: opts.algorithms,
            debug: opts.debug,
            max_messages_per_key: opts.max_messages_per_key as usize,
            on_stats: None,
            on_error: None,
        }
    }
}

impl From<crate::delta::DeltaOptions> for UniffiDeltaOptions {
    fn from(opts: crate::delta::DeltaOptions) -> Self {
        UniffiDeltaOptions {
            enabled: opts.enabled,
            algorithms: opts.algorithms,
            debug: opts.debug,
            max_messages_per_key: opts.max_messages_per_key as u32,
        }
    }
}

/// UniFFI-friendly Sockudo options
/// Exported as "SockudoOptions" in Kotlin/Swift
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[derive(Clone)]
pub struct SockudoOptions {
    pub app_key: String,
    pub cluster: Option<String>,
    pub ws_host: Option<String>,
    pub ws_port: Option<u16>,
    pub use_tls: Option<bool>,
    pub auth_endpoint: Option<String>,
    pub activity_timeout_ms: Option<u64>,
    pub pong_timeout_ms: Option<u64>,
    pub unavailable_timeout_ms: Option<u64>,
    pub delta_compression: Option<UniffiDeltaOptions>,
    pub enable_stats: Option<bool>,
    pub debug: Option<bool>,
    pub user_auth_endpoint: Option<String>,
    pub disable_reconnection: Option<bool>,
    pub max_reconnection_attempts: Option<u32>,
    pub reconnection_delay_ms: Option<u64>,
    pub max_reconnection_delay_ms: Option<u64>,
}

impl From<SockudoOptions> for crate::options::SockudoOptions {
    fn from(opts: SockudoOptions) -> Self {
        crate::SockudoOptions {
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

impl From<crate::options::SockudoOptions> for SockudoOptions {
    fn from(opts: crate::options::SockudoOptions) -> Self {
        SockudoOptions {
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

/// UniFFI-friendly Pusher event
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[cfg_attr(feature = "uniffi", uniffi(name = "PusherEvent"))]
#[derive(Clone)]
pub struct UniffiPusherEvent {
    pub event: String,
    pub channel: Option<String>,
    pub data: Option<String>,
    pub user_id: Option<String>,
}

/// UniFFI-friendly member info
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[cfg_attr(feature = "uniffi", uniffi(name = "MemberInfo"))]
#[derive(Clone)]
pub struct UniffiMemberInfo {
    pub user_id: String,
    pub user_info_json: Option<String>,
}

/// UniFFI-friendly delta statistics
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[cfg_attr(feature = "uniffi", uniffi(name = "DeltaStats"))]
#[derive(Clone, Default, Debug)]
pub struct UniffiDeltaStats {
    pub total_messages: u64,
    pub delta_messages: u64,
    pub full_messages: u64,
    pub total_bytes_without_compression: u64,
    pub total_bytes_with_compression: u64,
    pub bandwidth_saved: u64,
    pub bandwidth_saved_percent: f64,
    pub errors: u64,
    pub channel_count: u64,
}

impl From<crate::DeltaStats> for UniffiDeltaStats {
    fn from(stats: crate::DeltaStats) -> Self {
        Self {
            total_messages: stats.total_messages,
            delta_messages: stats.delta_messages,
            full_messages: stats.full_messages,
            total_bytes_without_compression: stats.total_bytes_without_compression,
            total_bytes_with_compression: stats.total_bytes_with_compression,
            bandwidth_saved: stats.bandwidth_saved,
            bandwidth_saved_percent: stats.bandwidth_saved_percent,
            errors: stats.errors,
            channel_count: stats.channel_count,
        }
    }
}

// Pusher-compatible type aliases for backward compatibility
// Note: UniFFI doesn't support type aliases directly, so these are for Rust code only
// The actual UniFFI exports use the names specified in the uniffi(name = "...") attributes
/// Pusher-compatible alias for SockudoOptions
pub type PusherOptions = SockudoOptions;
