//! Configuration options for the Sockudo client.

use crate::delta::DeltaOptions;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration options for creating a Sockudo client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SockudoOptions {
    /// Application key from your Pusher/Sockudo dashboard
    pub app_key: String,

    /// Cluster identifier (e.g., "mt1", "eu", "ap1")
    #[serde(default)]
    pub cluster: Option<String>,

    /// Custom WebSocket host (overrides cluster)
    #[serde(default)]
    pub ws_host: Option<String>,

    /// WebSocket port (default: 80 for ws, 443 for wss)
    #[serde(default)]
    pub ws_port: Option<u16>,

    /// Use TLS/WSS connection
    #[serde(default)]
    pub use_tls: Option<bool>,

    /// Custom authorization endpoint for private/presence channels
    #[serde(default)]
    pub auth_endpoint: Option<String>,

    /// Custom headers to send with authorization requests
    #[serde(default)]
    pub auth_headers: Option<HashMap<String, String>>,

    /// Activity timeout in milliseconds (default: 120000)
    #[serde(default)]
    pub activity_timeout_ms: Option<u64>,

    /// Pong timeout in milliseconds (default: 30000)
    #[serde(default)]
    pub pong_timeout_ms: Option<u64>,

    /// Unavailable timeout in milliseconds (default: 10000)
    #[serde(default)]
    pub unavailable_timeout_ms: Option<u64>,

    /// Delta compression configuration
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delta_compression: Option<DeltaOptions>,

    /// Enable timeline/stats collection
    #[serde(default)]
    pub enable_stats: Option<bool>,

    /// Enable debug logging
    #[serde(default)]
    pub debug: Option<bool>,

    /// User authentication endpoint
    #[serde(default)]
    pub user_auth_endpoint: Option<String>,

    /// User authentication headers
    #[serde(default)]
    pub user_auth_headers: Option<HashMap<String, String>>,

    /// Disable automatic reconnection
    #[serde(default)]
    pub disable_reconnection: Option<bool>,

    /// Maximum reconnection attempts (0 = unlimited)
    #[serde(default)]
    pub max_reconnection_attempts: Option<u32>,

    /// Initial reconnection delay in milliseconds
    #[serde(default)]
    pub reconnection_delay_ms: Option<u64>,

    /// Maximum reconnection delay in milliseconds
    #[serde(default)]
    pub max_reconnection_delay_ms: Option<u64>,
}

impl Default for SockudoOptions {
    fn default() -> Self {
        Self {
            app_key: String::new(),
            cluster: None,
            ws_host: None,
            ws_port: None,
            use_tls: None,
            auth_endpoint: Some("/pusher/auth".to_string()),
            auth_headers: None,
            activity_timeout_ms: Some(120_000),
            pong_timeout_ms: Some(30_000),
            unavailable_timeout_ms: Some(10_000),
            delta_compression: None,
            enable_stats: Some(false),
            debug: Some(false),
            user_auth_endpoint: Some("/pusher/user-auth".to_string()),
            user_auth_headers: None,
            disable_reconnection: Some(false),
            max_reconnection_attempts: Some(0),
            reconnection_delay_ms: Some(1000),
            max_reconnection_delay_ms: Some(30_000),
        }
    }
}

impl SockudoOptions {
    /// Create new options with just the app key
    pub fn new(app_key: impl Into<String>) -> Self {
        Self {
            app_key: app_key.into(),
            ..Default::default()
        }
    }

    /// Builder pattern: set cluster
    pub fn cluster(mut self, cluster: impl Into<String>) -> Self {
        self.cluster = Some(cluster.into());
        self
    }

    /// Builder pattern: set custom WebSocket host
    pub fn ws_host(mut self, host: impl Into<String>) -> Self {
        self.ws_host = Some(host.into());
        self
    }

    /// Builder pattern: set WebSocket port
    pub fn ws_port(mut self, port: u16) -> Self {
        self.ws_port = Some(port);
        self
    }

    /// Builder pattern: enable/disable TLS
    pub fn use_tls(mut self, use_tls: bool) -> Self {
        self.use_tls = Some(use_tls);
        self
    }

    /// Builder pattern: set auth endpoint
    pub fn auth_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.auth_endpoint = Some(endpoint.into());
        self
    }

    /// Builder pattern: add auth header
    pub fn auth_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        let headers = self.auth_headers.get_or_insert_with(HashMap::new);
        headers.insert(key.into(), value.into());
        self
    }

    /// Builder pattern: set delta compression options
    pub fn delta_compression(mut self, options: DeltaOptions) -> Self {
        self.delta_compression = Some(options);
        self
    }

    /// Builder pattern: enable delta compression with default options
    pub fn enable_delta_compression(mut self) -> Self {
        self.delta_compression = Some(DeltaOptions::default());
        self
    }

    /// Builder pattern: enable debug mode
    pub fn debug(mut self, enabled: bool) -> Self {
        self.debug = Some(enabled);
        self
    }

    /// Get the effective WebSocket URL
    pub fn get_ws_url(&self) -> String {
        let use_tls = self.use_tls.unwrap_or(true);
        let scheme = if use_tls { "wss" } else { "ws" };

        let host = if let Some(ref host) = self.ws_host {
            host.clone()
        } else if let Some(ref cluster) = self.cluster {
            format!("ws-{}.pusher.com", cluster)
        } else {
            "ws.pusherapp.com".to_string()
        };

        let port = self.ws_port.unwrap_or(if use_tls { 443 } else { 80 });

        // Don't include port in URL if it's the default for the scheme
        let port_str = if (use_tls && port == 443) || (!use_tls && port == 80) {
            String::new()
        } else {
            format!(":{}", port)
        };

        format!(
            "{}://{}{}/app/{}?protocol=7&client=sockudo-client-rust&version=0.1.0",
            scheme, host, port_str, self.app_key
        )
    }

    /// Get activity timeout duration
    pub fn get_activity_timeout(&self) -> std::time::Duration {
        std::time::Duration::from_millis(self.activity_timeout_ms.unwrap_or(120_000))
    }

    /// Get pong timeout duration
    pub fn get_pong_timeout(&self) -> std::time::Duration {
        std::time::Duration::from_millis(self.pong_timeout_ms.unwrap_or(30_000))
    }

    /// Get unavailable timeout duration
    pub fn get_unavailable_timeout(&self) -> std::time::Duration {
        std::time::Duration::from_millis(self.unavailable_timeout_ms.unwrap_or(10_000))
    }

    /// Check if delta compression is enabled
    pub fn is_delta_compression_enabled(&self) -> bool {
        self.delta_compression
            .as_ref()
            .map(|dc| dc.enabled)
            .unwrap_or(false)
    }

    /// Get delta compression options
    pub fn get_delta_compression(&self) -> Option<&DeltaOptions> {
        self.delta_compression.as_ref()
    }

    /// Check if debug mode is enabled
    pub fn is_debug(&self) -> bool {
        self.debug.unwrap_or(false)
    }
}

/// Pusher-compatible alias for SockudoOptions (for backward compatibility)
pub type PusherOptions = SockudoOptions;

/// Internal configuration derived from SockudoOptions
#[derive(Debug, Clone)]
pub struct Config {
    pub app_key: String,
    pub ws_url: String,
    pub auth_endpoint: String,
    pub auth_headers: HashMap<String, String>,
    pub activity_timeout: std::time::Duration,
    pub pong_timeout: std::time::Duration,
    pub unavailable_timeout: std::time::Duration,
    pub use_tls: bool,
    pub delta_compression: Option<DeltaOptions>,
    pub enable_stats: bool,
    pub debug: bool,
    pub user_auth_endpoint: String,
    pub user_auth_headers: HashMap<String, String>,
    pub disable_reconnection: bool,
    pub max_reconnection_attempts: u32,
    pub reconnection_delay: std::time::Duration,
    pub max_reconnection_delay: std::time::Duration,
}

impl From<PusherOptions> for Config {
    fn from(opts: PusherOptions) -> Self {
        Self {
            app_key: opts.app_key.clone(),
            ws_url: opts.get_ws_url(),
            auth_endpoint: opts
                .auth_endpoint
                .clone()
                .unwrap_or_else(|| "/pusher/auth".to_string()),
            auth_headers: opts.auth_headers.clone().unwrap_or_default(),
            activity_timeout: opts.get_activity_timeout(),
            pong_timeout: opts.get_pong_timeout(),
            unavailable_timeout: opts.get_unavailable_timeout(),
            use_tls: opts.use_tls.unwrap_or(true),
            delta_compression: opts.delta_compression.clone(),
            enable_stats: opts.enable_stats.unwrap_or(false),
            debug: opts.is_debug(),
            user_auth_endpoint: opts
                .user_auth_endpoint
                .unwrap_or_else(|| "/pusher/user-auth".to_string()),
            user_auth_headers: opts.user_auth_headers.unwrap_or_default(),
            disable_reconnection: opts.disable_reconnection.unwrap_or(false),
            max_reconnection_attempts: opts.max_reconnection_attempts.unwrap_or(0),
            reconnection_delay: std::time::Duration::from_millis(
                opts.reconnection_delay_ms.unwrap_or(1000),
            ),
            max_reconnection_delay: std::time::Duration::from_millis(
                opts.max_reconnection_delay_ms.unwrap_or(30_000),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_ws_url() {
        let opts = PusherOptions::new("test-key").cluster("mt1");
        let url = opts.get_ws_url();
        assert!(url.contains("wss://"));
        assert!(url.contains("ws-mt1.pusher.com"));
        assert!(url.contains("test-key"));
    }

    #[test]
    fn test_custom_host_url() {
        let opts = PusherOptions::new("test-key")
            .ws_host("localhost")
            .ws_port(6001)
            .use_tls(false);
        let url = opts.get_ws_url();
        assert!(url.contains("ws://localhost:6001"));
    }
}
