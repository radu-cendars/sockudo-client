//! Delta compression manager for handling delta-compressed messages.

use parking_lot::RwLock;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, warn};

use super::channel_state::ChannelState;
use super::decoders::{decode_base64, DeltaDecoder, FossilDeltaDecoder};
use super::types::*;
use crate::error::{Result, SockudoError};
use crate::protocol::PusherEvent;

/// Callback for sending events back to the connection
pub type SendEventFn = Arc<dyn Fn(&str, &Value) -> bool + Send + Sync>;

/// Manages delta compression for all channels
pub struct DeltaManager {
    /// Configuration options
    options: DeltaOptions,
    /// Whether delta compression is enabled
    enabled: RwLock<bool>,
    /// Per-channel state
    channel_states: RwLock<HashMap<String, Arc<ChannelState>>>,
    /// Global statistics
    stats: RwLock<DeltaStats>,
    /// Available decoders
    decoders: HashMap<String, Box<dyn DeltaDecoder>>,
    /// Callback for sending events
    send_event: Option<SendEventFn>,
}

impl DeltaManager {
    /// Create a new delta manager
    pub fn new(options: DeltaOptions) -> Self {
        let mut decoders: HashMap<String, Box<dyn DeltaDecoder>> = HashMap::new();

        // Add Fossil decoder if it's in the preferred algorithms
        if options.algorithms.contains(&DeltaAlgorithm::Fossil) {
            decoders.insert("fossil".to_string(), Box::new(FossilDeltaDecoder::new()));
        }

        Self {
            options,
            enabled: RwLock::new(false),
            channel_states: RwLock::new(HashMap::new()),
            stats: RwLock::new(DeltaStats::default()),
            decoders,
            send_event: None,
        }
    }

    /// Set the send event callback
    pub fn set_send_callback(&mut self, callback: SendEventFn) {
        self.send_event = Some(callback);
    }

    /// Get available algorithms
    pub fn available_algorithms(&self) -> Vec<DeltaAlgorithm> {
        self.decoders
            .keys()
            .filter_map(|k| k.parse().ok())
            .collect()
    }

    /// Enable delta compression by sending request to server
    pub fn enable(&self) {
        if !self.options.enabled {
            debug!("Delta compression disabled in options");
            return;
        }

        if *self.enabled.read() {
            debug!("Delta compression already enabled");
            return;
        }

        let available = self.available_algorithms();
        if available.is_empty() {
            warn!("No delta algorithms available");
            return;
        }

        // Filter to only algorithms we want
        let supported: Vec<_> = available
            .iter()
            .filter(|a| self.options.algorithms.contains(a))
            .map(|a| a.to_string())
            .collect();

        if supported.is_empty() {
            warn!("No mutually supported delta algorithms");
            return;
        }

        debug!(
            "Requesting delta compression with algorithms: {:?}",
            supported
        );

        if let Some(ref send) = self.send_event {
            let data = serde_json::json!({ "algorithms": supported });
            send("pusher:enable_delta_compression", &data);
        }
    }

    /// Disable delta compression
    pub fn disable(&self) {
        *self.enabled.write() = false;
        self.channel_states.write().clear();
    }

    /// Handle delta compression enabled confirmation
    pub fn handle_enabled(&self, data: &Value) {
        *self.enabled.write() = true;
        debug!("Delta compression enabled: {:?}", data);
    }

    /// Handle cache sync message
    pub fn handle_cache_sync(&self, channel: &str, data: CacheSyncData) {
        debug!(
            "Received cache sync for channel {}: {:?}",
            channel, data.conflation_key
        );

        let mut states = self.channel_states.write();
        let state = states
            .entry(channel.to_string())
            .or_insert_with(|| Arc::new(ChannelState::new(channel)));

        // We need mutable access to initialize
        if let Some(state_mut) = Arc::get_mut(state) {
            state_mut.initialize_from_cache_sync(&data);
        } else {
            // State is shared, need to create a new one
            let mut new_state = ChannelState::new(channel);
            new_state.initialize_from_cache_sync(&data);
            *state = Arc::new(new_state);
        }

        debug!("Cache sync complete. Groups: {}", state.group_count());
    }

    /// Handle a delta message
    pub fn handle_delta(&self, channel: &str, delta_msg: DeltaMessage) -> Result<PusherEvent> {
        let states = self.channel_states.read();
        let state = states.get(channel).ok_or_else(|| {
            let err = format!("No state for channel: {}", channel);
            self.emit_error(&err);
            SockudoError::delta(err)
        })?;

        // Get the base message
        let base = state
            .get_base(delta_msg.conflation_key.as_deref(), delta_msg.base_index)
            .ok_or_else(|| {
                let err = "No base message available";
                self.emit_error(err);
                self.stats.write().errors += 1;
                SockudoError::delta(err)
            })?;

        // Determine algorithm
        let algo = delta_msg.algorithm.as_deref().unwrap_or("fossil");
        let decoder = self.decoders.get(algo).ok_or_else(|| {
            let err = format!("Unknown algorithm: {}", algo);
            self.emit_error(&err);
            self.stats.write().errors += 1;
            SockudoError::delta(err)
        })?;

        // Decode the delta
        let delta_bytes = decode_base64(&delta_msg.delta).map_err(|e| {
            let err = format!("Base64 decode error: {}", e);
            self.emit_error(&err);
            self.stats.write().errors += 1;
            e
        })?;
        let base_bytes = base.as_bytes();

        let decoded = decoder.decode(base_bytes, &delta_bytes).map_err(|e| {
            let err = format!("Delta decode error: {}", e);
            self.emit_error(&err);
            self.stats.write().errors += 1;
            e
        })?;
        let content = String::from_utf8(decoded).map_err(|e| {
            let err = format!("Invalid UTF-8: {}", e);
            self.emit_error(&err);
            self.stats.write().errors += 1;
            SockudoError::delta(err)
        })?;

        // Update statistics
        let compressed_size = delta_msg.delta.len();
        let decompressed_size = content.len();

        {
            let mut stats = self.stats.write();
            stats.total_messages += 1;
            stats.delta_messages += 1;
            stats.total_bytes_with_compression += compressed_size as u64;
            stats.total_bytes_without_compression += decompressed_size as u64;
            stats.calculate_savings();
        }

        state.record_delta_message();

        // Store the reconstructed message as new base
        state.set_base_with_key(&content, delta_msg.seq, delta_msg.conflation_key.as_deref());

        // Emit stats update
        self.emit_stats();

        // Parse content as JSON and create event
        let data: Value = serde_json::from_str(&content).unwrap_or(Value::String(content.clone()));

        let mut event = PusherEvent::new(&delta_msg.event);
        event.channel = Some(channel.to_string());

        #[cfg(feature = "wasm")]
        {
            event.data = Some(data);
        }
        #[cfg(not(feature = "wasm"))]
        {
            event.data = Some(data.to_string());
        }

        Ok(event)
    }

    /// Handle a full message (for tracking and caching)
    pub fn handle_full_message(&self, channel: &str, event: &PusherEvent, sequence: u64) {
        let mut states = self.channel_states.write();
        let state = states
            .entry(channel.to_string())
            .or_insert_with(|| Arc::new(ChannelState::new(channel)));

        // Get raw message content
        let content = event
            .data
            .as_ref()
            .map(|d| d.to_string())
            .unwrap_or_default();

        let message_size = content.len();

        // Extract conflation key if present
        let conflation_key = if let Some(ref data) = event.data {
            if let Some(ref ck) = state.conflation_key {
                #[cfg(feature = "wasm")]
                {
                    data.get("__conflation_key")
                        .or_else(|| data.get(ck))
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                }
                #[cfg(not(feature = "wasm"))]
                {
                    // Parse JSON string to Value first
                    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(data) {
                        parsed
                            .get("__conflation_key")
                            .or_else(|| parsed.get(ck))
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string())
                    } else {
                        None
                    }
                }
            } else {
                None
            }
        } else {
            None
        };

        // Store as base
        state.set_base_with_key(&content, sequence, conflation_key.as_deref());
        state.record_full_message();

        // Update stats
        {
            let mut stats = self.stats.write();
            stats.total_messages += 1;
            stats.full_messages += 1;
            stats.total_bytes_without_compression += message_size as u64;
            stats.total_bytes_with_compression += message_size as u64;
        }

        self.emit_stats();
    }

    /// Request resync for a channel
    pub fn request_resync(&self, channel: &str) {
        warn!("Requesting resync for channel: {}", channel);

        if let Some(ref send) = self.send_event {
            let data = serde_json::json!({ "channel": channel });
            send("pusher:delta_sync_error", &data);
        }

        // Clear channel state
        self.channel_states.write().remove(channel);
    }

    /// Get current statistics
    pub fn get_stats(&self) -> DeltaStats {
        let mut stats = self.stats.read().clone();

        // Include per-channel stats
        let channel_stats: Vec<ChannelDeltaStats> = self
            .channel_states
            .read()
            .values()
            .map(|s| s.get_stats())
            .collect();

        stats.channel_count = channel_stats.len() as u64;
        stats.channels = channel_stats;

        stats
    }

    /// Reset statistics
    pub fn reset_stats(&self) {
        self.stats.write().reset();
    }

    /// Clear state for a specific channel
    pub fn clear_channel(&self, channel: &str) {
        self.channel_states.write().remove(channel);
    }

    /// Clear all state
    pub fn clear_all(&self) {
        self.channel_states.write().clear();
        self.stats.write().reset();
    }

    /// Check if delta compression is enabled
    pub fn is_enabled(&self) -> bool {
        *self.enabled.read()
    }

    /// Emit stats update to callback
    fn emit_stats(&self) {
        if let Some(ref callback) = self.options.on_stats {
            callback(&self.get_stats());
        }
    }

    /// Emit error to callback
    fn emit_error(&self, error: &str) {
        if let Some(ref callback) = self.options.on_error {
            callback(error);
        }
    }
}

impl std::fmt::Debug for DeltaManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DeltaManager")
            .field("enabled", &*self.enabled.read())
            .field("channel_count", &self.channel_states.read().len())
            .field("algorithms", &self.available_algorithms())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delta_manager_creation() {
        let options = DeltaOptions::default();
        let manager = DeltaManager::new(options);

        assert!(!manager.is_enabled());
        assert!(!manager.available_algorithms().is_empty());
    }

    #[test]
    fn test_full_message_tracking() {
        let options = DeltaOptions::default();
        let manager = DeltaManager::new(options);

        let event =
            PusherEvent::new("test-event").with_json_data(serde_json::json!({"price": 100}));

        manager.handle_full_message("test-channel", &event, 1);

        let stats = manager.get_stats();
        assert_eq!(stats.full_messages, 1);
        assert_eq!(stats.total_messages, 1);
    }

    #[test]
    fn test_cache_sync() {
        let options = DeltaOptions::default();
        let manager = DeltaManager::new(options);

        let mut states = HashMap::new();
        states.insert(
            "BTC".to_string(),
            vec![CachedMessage {
                content: r#"{"price":100}"#.to_string(),
                seq: 1,
            }],
        );

        let sync_data = CacheSyncData {
            conflation_key: Some("asset".to_string()),
            max_messages_per_key: Some(10),
            states: Some(states),
        };

        manager.handle_cache_sync("market-data", sync_data);

        let states = manager.channel_states.read();
        assert!(states.contains_key("market-data"));
    }
}
