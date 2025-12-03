//! Channel collection management.

use std::sync::Arc;
use dashmap::DashMap;
use tracing::debug;

use crate::error::{Result, SockudoError};
use super::channel::{Channel, ChannelType, SendEventFn, AuthorizeFn};
use super::presence_channel::PresenceChannel;
use super::encrypted_channel::EncryptedChannel;

/// Manages a collection of channels
pub struct Channels {
    /// Map of channel name to channel
    channels: DashMap<String, ChannelEntry>,
    /// Send event callback
    send_event: Option<SendEventFn>,
    /// Authorization callback
    authorize_fn: Option<AuthorizeFn>,
    /// Encryption key callback for encrypted channels
    encryption_callback: Option<Arc<dyn Fn() -> Option<[u8; 32]> + Send + Sync>>,
}

/// Entry that can hold different channel types
enum ChannelEntry {
    Basic(Arc<Channel>),
    Presence(Arc<PresenceChannel>),
    Encrypted(Arc<EncryptedChannel>),
}

impl Channels {
    pub fn new() -> Self {
        Self {
            channels: DashMap::new(),
            send_event: None,
            authorize_fn: None,
            encryption_callback: None,
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
    
    /// Set the encryption key callback
    pub fn set_encryption_callback(&mut self, callback: impl Fn() -> Option<[u8; 32]> + Send + Sync + 'static) {
        self.encryption_callback = Some(Arc::new(callback));
    }
    
    /// Add or get a channel by name
    pub fn add(&self, name: &str) -> Result<Arc<Channel>> {
        if let Some(entry) = self.channels.get(name) {
            return match &*entry {
                ChannelEntry::Basic(ch) => Ok(ch.clone()),
                ChannelEntry::Presence(ch) => Ok(ch.as_channel()),
                ChannelEntry::Encrypted(ch) => Ok(ch.as_channel()),
            };
        }
        
        let channel_type = ChannelType::from_name(name);
        
        let entry = match channel_type {
            ChannelType::PrivateEncrypted => {
                if self.encryption_callback.is_none() {
                    return Err(SockudoError::config(
                        "Encrypted channels require nacl/encryption support"
                    ));
                }
                let mut channel = EncryptedChannel::new(name);
                if let Some(ref cb) = self.send_event {
                    channel.set_send_callback(cb.clone());
                }
                if let Some(ref cb) = self.authorize_fn {
                    channel.set_authorize_callback(cb.clone());
                }
                ChannelEntry::Encrypted(Arc::new(channel))
            }
            ChannelType::Presence => {
                let mut channel = PresenceChannel::new(name);
                if let Some(ref cb) = self.send_event {
                    channel.set_send_callback(cb.clone());
                }
                if let Some(ref cb) = self.authorize_fn {
                    channel.set_authorize_callback(cb.clone());
                }
                ChannelEntry::Presence(Arc::new(channel))
            }
            _ => {
                let mut channel = Channel::new(name);
                if let Some(ref cb) = self.send_event {
                    channel.set_send_callback(cb.clone());
                }
                if let Some(ref cb) = self.authorize_fn {
                    channel.set_authorize_callback(cb.clone());
                }
                ChannelEntry::Basic(Arc::new(channel))
            }
        };
        
        // Get the channel reference before inserting
        let channel = match &entry {
            ChannelEntry::Basic(ch) => ch.clone(),
            ChannelEntry::Presence(ch) => ch.as_channel(),
            ChannelEntry::Encrypted(ch) => ch.as_channel(),
        };
        
        self.channels.insert(name.to_string(), entry);
        debug!("Created channel: {}", name);
        
        Ok(channel)
    }
    
    /// Find a channel by name
    pub fn find(&self, name: &str) -> Option<Arc<Channel>> {
        self.channels.get(name).map(|entry| {
            match &*entry {
                ChannelEntry::Basic(ch) => ch.clone(),
                ChannelEntry::Presence(ch) => ch.as_channel(),
                ChannelEntry::Encrypted(ch) => ch.as_channel(),
            }
        })
    }
    
    /// Find a presence channel by name
    pub fn find_presence(&self, name: &str) -> Option<Arc<PresenceChannel>> {
        self.channels.get(name).and_then(|entry| {
            match &*entry {
                ChannelEntry::Presence(ch) => Some(ch.clone()),
                _ => None,
            }
        })
    }
    
    /// Find an encrypted channel by name
    pub fn find_encrypted(&self, name: &str) -> Option<Arc<EncryptedChannel>> {
        self.channels.get(name).and_then(|entry| {
            match &*entry {
                ChannelEntry::Encrypted(ch) => Some(ch.clone()),
                _ => None,
            }
        })
    }
    
    /// Remove a channel
    pub fn remove(&self, name: &str) -> Option<Arc<Channel>> {
        self.channels.remove(name).map(|(_, entry)| {
            debug!("Removed channel: {}", name);
            match entry {
                ChannelEntry::Basic(ch) => ch,
                ChannelEntry::Presence(ch) => ch.as_channel(),
                ChannelEntry::Encrypted(ch) => ch.as_channel(),
            }
        })
    }
    
    /// Get all channels
    pub fn all(&self) -> Vec<Arc<Channel>> {
        self.channels.iter().map(|entry| {
            match &*entry {
                ChannelEntry::Basic(ch) => ch.clone(),
                ChannelEntry::Presence(ch) => ch.as_channel(),
                ChannelEntry::Encrypted(ch) => ch.as_channel(),
            }
        }).collect()
    }
    
    /// Get channel count
    pub fn len(&self) -> usize {
        self.channels.len()
    }
    
    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.channels.is_empty()
    }
    
    /// Disconnect all channels
    pub fn disconnect(&self) {
        for entry in self.channels.iter() {
            match &*entry {
                ChannelEntry::Basic(ch) => ch.disconnect(),
                ChannelEntry::Presence(ch) => ch.disconnect(),
                ChannelEntry::Encrypted(ch) => ch.disconnect(),
            }
        }
    }
    
    /// Clear all channels
    pub fn clear(&self) {
        self.channels.clear();
    }
}

impl Default for Channels {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for Channels {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Channels")
            .field("count", &self.len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_find() {
        let channels = Channels::new();
        
        let ch1 = channels.add("test-channel").unwrap();
        assert_eq!(ch1.name(), "test-channel");
        
        let ch2 = channels.find("test-channel").unwrap();
        assert_eq!(ch1.name(), ch2.name());
    }

    #[test]
    fn test_channel_type_creation() {
        let channels = Channels::new();
        
        let public = channels.add("my-channel").unwrap();
        assert_eq!(public.channel_type(), ChannelType::Public);
        
        let private = channels.add("private-channel").unwrap();
        assert_eq!(private.channel_type(), ChannelType::Private);
    }

    #[test]
    fn test_presence_channel() {
        let channels = Channels::new();
        
        channels.add("presence-room").unwrap();
        let presence = channels.find_presence("presence-room");
        assert!(presence.is_some());
    }

    #[test]
    fn test_remove() {
        let channels = Channels::new();
        
        channels.add("test-channel").unwrap();
        assert_eq!(channels.len(), 1);
        
        channels.remove("test-channel");
        assert_eq!(channels.len(), 0);
    }
}
