//! Callback registry for managing event callbacks.

use std::sync::Arc;
use parking_lot::RwLock;
use dashmap::DashMap;
use crate::protocol::PusherEvent;

/// Type alias for callback function
pub type CallbackFn = Arc<dyn Fn(&PusherEvent) + Send + Sync + 'static>;

/// A registered callback with optional context
#[derive(Clone)]
pub struct Callback {
    pub id: u64,
    pub callback: CallbackFn,
}

impl Callback {
    pub fn new(id: u64, callback: impl Fn(&PusherEvent) + Send + Sync + 'static) -> Self {
        Self {
            id,
            callback: Arc::new(callback),
        }
    }
    
    pub fn invoke(&self, event: &PusherEvent) {
        (self.callback)(event);
    }
}

impl std::fmt::Debug for Callback {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Callback")
            .field("id", &self.id)
            .finish()
    }
}

/// Registry for storing callbacks per event name
#[derive(Debug, Default)]
pub struct CallbackRegistry {
    /// Event-specific callbacks: event_name -> [callbacks]
    callbacks: DashMap<String, Vec<Callback>>,
    /// Global callbacks that receive all events
    global_callbacks: RwLock<Vec<Callback>>,
    /// Counter for generating unique callback IDs
    next_id: std::sync::atomic::AtomicU64,
}

impl CallbackRegistry {
    pub fn new() -> Self {
        Self {
            callbacks: DashMap::new(),
            global_callbacks: RwLock::new(Vec::new()),
            next_id: std::sync::atomic::AtomicU64::new(1),
        }
    }
    
    /// Generate a unique callback ID
    fn next_id(&self) -> u64 {
        self.next_id.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
    }
    
    /// Add a callback for a specific event
    pub fn add(&self, event_name: impl Into<String>, callback: impl Fn(&PusherEvent) + Send + Sync + 'static) -> u64 {
        let id = self.next_id();
        let cb = Callback::new(id, callback);
        
        self.callbacks
            .entry(event_name.into())
            .or_default()
            .push(cb);
        
        id
    }
    
    /// Add a global callback that receives all events
    pub fn add_global(&self, callback: impl Fn(&PusherEvent) + Send + Sync + 'static) -> u64 {
        let id = self.next_id();
        let cb = Callback::new(id, callback);
        
        self.global_callbacks.write().push(cb);
        
        id
    }
    
    /// Get callbacks for a specific event
    pub fn get(&self, event_name: &str) -> Vec<Callback> {
        self.callbacks
            .get(event_name)
            .map(|v| v.clone())
            .unwrap_or_default()
    }
    
    /// Get global callbacks
    pub fn get_global(&self) -> Vec<Callback> {
        self.global_callbacks.read().clone()
    }
    
    /// Remove a specific callback by ID
    pub fn remove(&self, event_name: Option<&str>, callback_id: Option<u64>) {
        match (event_name, callback_id) {
            (Some(name), Some(id)) => {
                // Remove specific callback from specific event
                if let Some(mut callbacks) = self.callbacks.get_mut(name) {
                    callbacks.retain(|cb| cb.id != id);
                }
            }
            (Some(name), None) => {
                // Remove all callbacks for specific event
                self.callbacks.remove(name);
            }
            (None, Some(id)) => {
                // Remove callback with ID from all events
                for mut entry in self.callbacks.iter_mut() {
                    entry.retain(|cb| cb.id != id);
                }
                self.global_callbacks.write().retain(|cb| cb.id != id);
            }
            (None, None) => {
                // Remove all callbacks
                self.callbacks.clear();
                self.global_callbacks.write().clear();
            }
        }
    }
    
    /// Remove a global callback by ID
    pub fn remove_global(&self, callback_id: Option<u64>) {
        if let Some(id) = callback_id {
            self.global_callbacks.write().retain(|cb| cb.id != id);
        } else {
            self.global_callbacks.write().clear();
        }
    }
    
    /// Remove all callbacks
    pub fn clear(&self) {
        self.callbacks.clear();
        self.global_callbacks.write().clear();
    }
    
    /// Check if there are any callbacks for an event
    pub fn has_callbacks(&self, event_name: &str) -> bool {
        self.callbacks
            .get(event_name)
            .map(|v| !v.is_empty())
            .unwrap_or(false)
    }
    
    /// Get number of registered callbacks
    pub fn callback_count(&self) -> usize {
        let event_count: usize = self.callbacks.iter().map(|v| v.len()).sum();
        let global_count = self.global_callbacks.read().len();
        event_count + global_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn test_add_and_get_callback() {
        let registry = CallbackRegistry::new();
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();
        
        registry.add("test-event", move |_| {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        });
        
        let callbacks = registry.get("test-event");
        assert_eq!(callbacks.len(), 1);
        
        let event = PusherEvent::new("test-event");
        callbacks[0].invoke(&event);
        
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_global_callback() {
        let registry = CallbackRegistry::new();
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();
        
        registry.add_global(move |_| {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        });
        
        let callbacks = registry.get_global();
        assert_eq!(callbacks.len(), 1);
    }

    #[test]
    fn test_remove_callback() {
        let registry = CallbackRegistry::new();
        
        let id = registry.add("test-event", |_| {});
        assert!(registry.has_callbacks("test-event"));
        
        registry.remove(Some("test-event"), Some(id));
        assert!(!registry.has_callbacks("test-event"));
    }

    #[test]
    fn test_clear() {
        let registry = CallbackRegistry::new();
        
        registry.add("event1", |_| {});
        registry.add("event2", |_| {});
        registry.add_global(|_| {});
        
        assert!(registry.callback_count() >= 3);
        
        registry.clear();
        
        assert_eq!(registry.callback_count(), 0);
    }
}
