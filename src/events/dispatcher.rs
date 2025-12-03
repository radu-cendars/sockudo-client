//! Event dispatcher for managing and emitting events.

use super::callback::CallbackRegistry;
use crate::protocol::PusherEvent;
use parking_lot::RwLock;
use std::sync::Arc;
use tracing::{debug, warn};

/// Callback for when no handlers are registered for an event
pub type FailThroughFn = Arc<dyn Fn(&str, &PusherEvent) + Send + Sync + 'static>;

/// Event dispatcher that manages callback bindings and event emission.
///
/// This is the core event system used by channels and the main client.
#[derive(Clone)]
pub struct EventDispatcher {
    /// Registry of callbacks
    callbacks: Arc<CallbackRegistry>,
    /// Optional callback when no listeners are bound
    fail_through: Arc<RwLock<Option<FailThroughFn>>>,
}

impl Default for EventDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl EventDispatcher {
    /// Create a new event dispatcher
    pub fn new() -> Self {
        Self {
            callbacks: Arc::new(CallbackRegistry::new()),
            fail_through: Arc::new(RwLock::new(None)),
        }
    }

    /// Create a dispatcher with a fail-through callback
    pub fn with_fail_through(
        fail_through: impl Fn(&str, &PusherEvent) + Send + Sync + 'static,
    ) -> Self {
        let dispatcher = Self::new();
        *dispatcher.fail_through.write() = Some(Arc::new(fail_through));
        dispatcher
    }

    /// Bind a callback to a specific event
    pub fn bind(
        &self,
        event_name: impl Into<String>,
        callback: impl Fn(&PusherEvent) + Send + Sync + 'static,
    ) -> u64 {
        let name = event_name.into();
        debug!("Binding callback for event: {}", name);
        self.callbacks.add(name, callback)
    }

    /// Bind a callback to all events (global binding)
    pub fn bind_global(&self, callback: impl Fn(&PusherEvent) + Send + Sync + 'static) -> u64 {
        debug!("Binding global callback");
        self.callbacks.add_global(callback)
    }

    /// Unbind callbacks from an event
    pub fn unbind(&self, event_name: Option<&str>, callback_id: Option<u64>) {
        debug!(
            "Unbinding callback: event={:?}, id={:?}",
            event_name, callback_id
        );
        self.callbacks.remove(event_name, callback_id);
    }

    /// Unbind global callbacks
    pub fn unbind_global(&self, callback_id: Option<u64>) {
        debug!("Unbinding global callback: id={:?}", callback_id);
        self.callbacks.remove_global(callback_id);
    }

    /// Unbind all callbacks
    pub fn unbind_all(&self) {
        debug!("Unbinding all callbacks");
        self.callbacks.clear();
    }

    /// Emit an event to all registered callbacks
    pub fn emit(&self, event: &PusherEvent) {
        let event_name = &event.event;

        // Call global callbacks first
        for callback in self.callbacks.get_global() {
            if let Err(e) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                callback.invoke(event);
            })) {
                warn!("Global callback panicked: {:?}", e);
            }
        }

        // Call event-specific callbacks
        let callbacks = self.callbacks.get(event_name);

        if !callbacks.is_empty() {
            for callback in callbacks {
                if let Err(e) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    callback.invoke(event);
                })) {
                    warn!("Callback for '{}' panicked: {:?}", event_name, e);
                }
            }
        } else {
            // No callbacks registered, call fail-through if set
            if let Some(ref fail_through) = *self.fail_through.read() {
                debug!("No callbacks for '{}', calling fail-through", event_name);
                fail_through(event_name, event);
            }
        }
    }

    /// Emit an event with a specific name and data (WASM version)
    #[cfg(feature = "wasm")]
    pub fn emit_event(&self, event_name: impl Into<String>, data: Option<serde_json::Value>) {
        let mut event = PusherEvent::new(event_name);
        event.data = data;
        self.emit(&event);
    }

    /// Emit an event with a specific name and data (FFI version)
    #[cfg(not(feature = "wasm"))]
    pub fn emit_event(&self, event_name: impl Into<String>, data: Option<String>) {
        let mut event = PusherEvent::new(event_name);
        event.data = data;
        self.emit(&event);
    }

    /// Check if there are any callbacks for an event
    pub fn has_callbacks(&self, event_name: &str) -> bool {
        self.callbacks.has_callbacks(event_name)
    }

    /// Get total number of registered callbacks
    pub fn callback_count(&self) -> usize {
        self.callbacks.callback_count()
    }
}

impl std::fmt::Debug for EventDispatcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EventDispatcher")
            .field("callback_count", &self.callback_count())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn test_bind_and_emit() {
        let dispatcher = EventDispatcher::new();
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();

        dispatcher.bind("test-event", move |_| {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        });

        let event = PusherEvent::new("test-event");
        dispatcher.emit(&event);

        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_global_bind() {
        let dispatcher = EventDispatcher::new();
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();

        dispatcher.bind_global(move |_| {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        });

        dispatcher.emit(&PusherEvent::new("event1"));
        dispatcher.emit(&PusherEvent::new("event2"));

        assert_eq!(counter.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn test_fail_through() {
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();

        let dispatcher = EventDispatcher::with_fail_through(move |name, _| {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        });

        // Emit event with no callbacks - should trigger fail-through
        dispatcher.emit(&PusherEvent::new("unknown-event"));

        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_unbind() {
        let dispatcher = EventDispatcher::new();
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();

        let id = dispatcher.bind("test-event", move |_| {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        });

        dispatcher.emit(&PusherEvent::new("test-event"));
        assert_eq!(counter.load(Ordering::SeqCst), 1);

        dispatcher.unbind(Some("test-event"), Some(id));

        dispatcher.emit(&PusherEvent::new("test-event"));
        assert_eq!(counter.load(Ordering::SeqCst), 1); // Should not increment
    }
}
