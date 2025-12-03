//! Per-channel state management for delta compression.

use std::collections::{HashMap, VecDeque};
use parking_lot::RwLock;
use crate::delta::types::*;

/// Manages delta compression state for a single channel
#[derive(Debug)]
pub struct ChannelState {
    /// Channel name
    pub channel_name: String,
    /// Conflation key field name (if any)
    pub conflation_key: Option<String>,
    /// Maximum messages to cache per key
    pub max_messages_per_key: usize,
    /// Cached messages per conflation key value
    /// Key is the conflation key value, value is a FIFO queue of messages
    cached_messages: RwLock<HashMap<String, VecDeque<CachedMessageEntry>>>,
    /// Last sequence number seen
    pub last_sequence: RwLock<u64>,
    /// Statistics for this channel
    pub stats: RwLock<ChannelDeltaStats>,
}

/// Internal cached message entry
#[derive(Debug, Clone)]
struct CachedMessageEntry {
    content: String,
    sequence: u64,
}

impl ChannelState {
    pub fn new(channel_name: impl Into<String>) -> Self {
        let name = channel_name.into();
        Self {
            channel_name: name.clone(),
            conflation_key: None,
            max_messages_per_key: 10,
            cached_messages: RwLock::new(HashMap::new()),
            last_sequence: RwLock::new(0),
            stats: RwLock::new(ChannelDeltaStats {
                channel_name: name,
                ..Default::default()
            }),
        }
    }
    
    /// Initialize from cache sync data received from server
    pub fn initialize_from_cache_sync(&mut self, data: &CacheSyncData) {
        if let Some(ref key) = data.conflation_key {
            self.conflation_key = Some(key.clone());
        }
        
        if let Some(max) = data.max_messages_per_key {
            self.max_messages_per_key = max;
        }
        
        // Load cached states
        if let Some(ref states) = data.states {
            let mut cache = self.cached_messages.write();
            cache.clear();
            
            for (key_value, messages) in states {
                let mut queue = VecDeque::with_capacity(self.max_messages_per_key);
                for msg in messages {
                    queue.push_back(CachedMessageEntry {
                        content: msg.content.clone(),
                        sequence: msg.seq,
                    });
                }
                cache.insert(key_value.clone(), queue);
            }
            
            // Update stats
            let mut stats = self.stats.write();
            stats.conflation_key = self.conflation_key.clone();
            stats.conflation_group_count = cache.len() as u32;
        }
    }
    
    /// Get the base message for delta decoding
    pub fn get_base(&self, conflation_key_value: Option<&str>, base_index: Option<usize>) -> Option<String> {
        let cache = self.cached_messages.read();
        
        let key = conflation_key_value.unwrap_or("__default__");
        let queue = cache.get(key)?;
        
        if queue.is_empty() {
            return None;
        }
        
        // Use base_index if provided, otherwise use the last message
        let index = base_index.unwrap_or(queue.len() - 1);
        queue.get(index).map(|e| e.content.clone())
    }
    
    /// Set a base message (for full messages)
    pub fn set_base(&self, content: impl Into<String>, sequence: u64) {
        self.set_base_with_key(content, sequence, None);
    }
    
    /// Set a base message with a specific conflation key value
    pub fn set_base_with_key(
        &self,
        content: impl Into<String>,
        sequence: u64,
        conflation_key_value: Option<&str>,
    ) {
        let key = conflation_key_value.unwrap_or("__default__").to_string();
        let entry = CachedMessageEntry {
            content: content.into(),
            sequence,
        };
        
        let mut cache = self.cached_messages.write();
        let queue = cache.entry(key).or_insert_with(VecDeque::new);
        
        // FIFO eviction
        while queue.len() >= self.max_messages_per_key {
            queue.pop_front();
        }
        
        queue.push_back(entry);
        
        // Update last sequence
        let mut last_seq = self.last_sequence.write();
        if sequence > *last_seq {
            *last_seq = sequence;
        }
        
        // Update stats
        let mut stats = self.stats.write();
        stats.conflation_group_count = cache.len() as u32;
    }
    
    /// Record a full message received
    pub fn record_full_message(&self) {
        let mut stats = self.stats.write();
        stats.full_message_count += 1;
        stats.total_messages += 1;
    }
    
    /// Record a delta message received
    pub fn record_delta_message(&self) {
        let mut stats = self.stats.write();
        stats.delta_count += 1;
        stats.total_messages += 1;
    }
    
    /// Get statistics for this channel
    pub fn get_stats(&self) -> ChannelDeltaStats {
        self.stats.read().clone()
    }
    
    /// Clear all cached state
    pub fn clear(&self) {
        self.cached_messages.write().clear();
        *self.last_sequence.write() = 0;
    }
    
    /// Get number of cached message groups
    pub fn group_count(&self) -> usize {
        self.cached_messages.read().len()
    }
    
    /// Get total number of cached messages
    pub fn message_count(&self) -> usize {
        self.cached_messages.read()
            .values()
            .map(|q| q.len())
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_state_basic() {
        let state = ChannelState::new("test-channel");
        
        state.set_base("message1", 1);
        assert!(state.get_base(None, None).is_some());
        
        state.set_base("message2", 2);
        assert_eq!(state.get_base(None, None).unwrap(), "message2");
    }

    #[test]
    fn test_conflation_keys() {
        let state = ChannelState::new("test-channel");
        
        state.set_base_with_key("btc-message1", 1, Some("BTC"));
        state.set_base_with_key("eth-message1", 2, Some("ETH"));
        state.set_base_with_key("btc-message2", 3, Some("BTC"));
        
        assert_eq!(state.get_base(Some("BTC"), None).unwrap(), "btc-message2");
        assert_eq!(state.get_base(Some("ETH"), None).unwrap(), "eth-message1");
        assert!(state.get_base(Some("XRP"), None).is_none());
    }

    #[test]
    fn test_fifo_eviction() {
        let mut state = ChannelState::new("test-channel");
        state.max_messages_per_key = 3;
        
        for i in 0..5 {
            state.set_base(format!("message{}", i), i as u64);
        }
        
        // Should only have last 3 messages
        assert_eq!(state.message_count(), 3);
        
        // First message should be message2 (oldest after eviction)
        assert_eq!(state.get_base(None, Some(0)).unwrap(), "message2");
    }

    #[test]
    fn test_cache_sync() {
        let mut state = ChannelState::new("test-channel");
        
        let mut states = HashMap::new();
        states.insert("BTC".to_string(), vec![
            CachedMessage { content: "btc1".to_string(), seq: 1 },
            CachedMessage { content: "btc2".to_string(), seq: 2 },
        ]);
        states.insert("ETH".to_string(), vec![
            CachedMessage { content: "eth1".to_string(), seq: 1 },
        ]);
        
        let sync_data = CacheSyncData {
            conflation_key: Some("asset".to_string()),
            max_messages_per_key: Some(10),
            states: Some(states),
        };
        
        state.initialize_from_cache_sync(&sync_data);
        
        assert_eq!(state.conflation_key, Some("asset".to_string()));
        assert_eq!(state.group_count(), 2);
        assert_eq!(state.get_base(Some("BTC"), None).unwrap(), "btc2");
    }
}
