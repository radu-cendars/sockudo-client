//! Delta compression types and data structures.

use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Available delta compression algorithms
#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DeltaAlgorithm {
    /// Fossil Delta algorithm - fast and compact
    Fossil,
    /// Xdelta3/VCDIFF algorithm - better compression for large diffs
    Xdelta3,
}

impl Default for DeltaAlgorithm {
    fn default() -> Self {
        Self::Fossil
    }
}

impl std::fmt::Display for DeltaAlgorithm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Fossil => write!(f, "fossil"),
            Self::Xdelta3 => write!(f, "xdelta3"),
        }
    }
}

impl std::str::FromStr for DeltaAlgorithm {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "fossil" => Ok(Self::Fossil),
            "xdelta3" | "vcdiff" => Ok(Self::Xdelta3),
            _ => Err(format!("Unknown algorithm: {}", s)),
        }
    }
}

/// Callback type for stats updates
pub type StatsCallback = Arc<dyn Fn(&DeltaStats) + Send + Sync>;

/// Callback type for error notifications
pub type ErrorCallback = Arc<dyn Fn(&str) + Send + Sync>;

/// Delta compression configuration options
#[derive(Clone, Serialize, Deserialize)]
pub struct DeltaOptions {
    /// Enable delta compression
    pub enabled: bool,
    /// Preferred algorithms in order of preference
    pub algorithms: Vec<DeltaAlgorithm>,
    /// Enable debug logging
    pub debug: bool,
    /// Maximum messages per conflation key (default: 10)
    pub max_messages_per_key: usize,
    /// Callback for stats updates (optional)
    #[serde(skip)]
    pub on_stats: Option<StatsCallback>,
    /// Callback for error notifications (optional)
    #[serde(skip)]
    pub on_error: Option<ErrorCallback>,
}

impl std::fmt::Debug for DeltaOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DeltaOptions")
            .field("enabled", &self.enabled)
            .field("algorithms", &self.algorithms)
            .field("debug", &self.debug)
            .field("max_messages_per_key", &self.max_messages_per_key)
            .field("on_stats", &self.on_stats.is_some())
            .field("on_error", &self.on_error.is_some())
            .finish()
    }
}

impl Default for DeltaOptions {
    fn default() -> Self {
        Self {
            enabled: true,
            algorithms: vec![DeltaAlgorithm::Fossil, DeltaAlgorithm::Xdelta3],
            debug: false,
            max_messages_per_key: 10,
            on_stats: None,
            on_error: None,
        }
    }
}

/// Statistics for delta compression performance
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeltaStats {
    /// Total messages processed
    pub total_messages: u64,
    /// Messages received as deltas
    pub delta_messages: u64,
    /// Messages received as full messages
    pub full_messages: u64,
    /// Total bytes without compression
    pub total_bytes_without_compression: u64,
    /// Total bytes with compression
    pub total_bytes_with_compression: u64,
    /// Bandwidth saved in bytes
    pub bandwidth_saved: u64,
    /// Bandwidth saved as percentage
    pub bandwidth_saved_percent: f64,
    /// Number of errors encountered
    pub errors: u64,
    /// Number of channels using delta compression
    pub channel_count: u64,
    /// Per-channel statistics
    pub channels: Vec<ChannelDeltaStats>,
}

impl DeltaStats {
    pub fn new() -> Self {
        Self::default()
    }

    /// Calculate bandwidth savings
    pub fn calculate_savings(&mut self) {
        if self.total_bytes_without_compression > 0 {
            self.bandwidth_saved = self
                .total_bytes_without_compression
                .saturating_sub(self.total_bytes_with_compression);
            self.bandwidth_saved_percent =
                (self.bandwidth_saved as f64 / self.total_bytes_without_compression as f64) * 100.0;
        }
    }

    /// Merge another stats into this one
    pub fn merge(&mut self, other: &DeltaStats) {
        self.total_messages += other.total_messages;
        self.delta_messages += other.delta_messages;
        self.full_messages += other.full_messages;
        self.total_bytes_without_compression += other.total_bytes_without_compression;
        self.total_bytes_with_compression += other.total_bytes_with_compression;
        self.errors += other.errors;
        self.calculate_savings();
    }

    /// Reset statistics
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

/// Per-channel delta statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(all(not(feature = "wasm"), feature = "uniffi"), derive(uniffi::Record))]
pub struct ChannelDeltaStats {
    pub channel_name: String,
    pub conflation_key: Option<String>,
    pub conflation_group_count: u32,
    pub delta_count: u64,
    pub full_message_count: u64,
    pub total_messages: u64,
}

/// Delta message from server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeltaMessage {
    /// Original event name
    pub event: String,
    /// Base64-encoded delta
    pub delta: String,
    /// Sequence number
    pub seq: u64,
    /// Algorithm used
    #[serde(default)]
    pub algorithm: Option<String>,
    /// Conflation key value
    #[serde(default)]
    pub conflation_key: Option<String>,
    /// Index into the cache for base message
    #[serde(default)]
    pub base_index: Option<usize>,
}

/// Cache sync data from server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheSyncData {
    /// The conflation key field name
    #[serde(default)]
    pub conflation_key: Option<String>,
    /// Maximum messages per key
    #[serde(default)]
    pub max_messages_per_key: Option<usize>,
    /// Cached states per conflation key value
    #[serde(default)]
    pub states: Option<std::collections::HashMap<String, Vec<CachedMessage>>>,
}

/// A cached message entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedMessage {
    /// Message content as JSON string
    pub content: String,
    /// Sequence number
    pub seq: u64,
}

/// Result of delta decoding
#[derive(Debug, Clone)]
pub struct DecodedMessage {
    /// Reconstructed message content
    pub content: String,
    /// Sequence number
    pub sequence: u64,
    /// Original compressed size
    pub compressed_size: usize,
    /// Decompressed size
    pub decompressed_size: usize,
}
