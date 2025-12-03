//! Timer utilities.

use parking_lot::Mutex;
use std::sync::Arc;
use std::time::Duration;

#[cfg(not(target_arch = "wasm32"))]
use tokio::sync::oneshot;
#[cfg(not(target_arch = "wasm32"))]
use tokio::time::sleep;

/// A timer that can be cancelled
#[cfg(not(target_arch = "wasm32"))]
pub struct CancellableTimer {
    cancel_tx: Option<oneshot::Sender<()>>,
}

#[cfg(not(target_arch = "wasm32"))]
impl CancellableTimer {
    /// Create a new timer that executes a callback after the specified duration
    pub fn new<F>(duration: Duration, callback: F) -> Self
    where
        F: FnOnce() + Send + 'static,
    {
        let (cancel_tx, cancel_rx) = oneshot::channel();

        tokio::spawn(async move {
            tokio::select! {
                _ = sleep(duration) => {
                    callback();
                }
                _ = cancel_rx => {
                    // Timer was cancelled
                }
            }
        });

        Self {
            cancel_tx: Some(cancel_tx),
        }
    }

    /// Cancel the timer
    pub fn cancel(&mut self) {
        if let Some(tx) = self.cancel_tx.take() {
            let _ = tx.send(());
        }
    }

    /// Check if the timer is still active
    pub fn is_active(&self) -> bool {
        self.cancel_tx.is_some()
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl Drop for CancellableTimer {
    fn drop(&mut self) {
        self.cancel();
    }
}

/// A repeating timer
#[cfg(not(target_arch = "wasm32"))]
pub struct PeriodicTimer {
    cancel_tx: Option<oneshot::Sender<()>>,
}

#[cfg(not(target_arch = "wasm32"))]
impl PeriodicTimer {
    /// Create a new periodic timer
    pub fn new<F>(interval: Duration, mut callback: F) -> Self
    where
        F: FnMut() + Send + 'static,
    {
        let (cancel_tx, mut cancel_rx) = oneshot::channel();

        tokio::spawn(async move {
            let mut timer = tokio::time::interval(interval);
            loop {
                tokio::select! {
                    _ = timer.tick() => {
                        callback();
                    }
                    _ = &mut cancel_rx => {
                        break;
                    }
                }
            }
        });

        Self {
            cancel_tx: Some(cancel_tx),
        }
    }

    /// Stop the periodic timer
    pub fn stop(&mut self) {
        if let Some(tx) = self.cancel_tx.take() {
            let _ = tx.send(());
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl Drop for PeriodicTimer {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Debounce a function call
#[cfg(not(target_arch = "wasm32"))]
pub struct Debouncer<F>
where
    F: Fn() + Send + Sync + 'static,
{
    callback: Arc<F>,
    delay: Duration,
    timer: Arc<Mutex<Option<CancellableTimer>>>,
}

#[cfg(not(target_arch = "wasm32"))]
impl<F> Debouncer<F>
where
    F: Fn() + Send + Sync + 'static,
{
    pub fn new(delay: Duration, callback: F) -> Self {
        Self {
            callback: Arc::new(callback),
            delay,
            timer: Arc::new(Mutex::new(None)),
        }
    }

    pub fn call(&self) {
        let mut timer_guard = self.timer.lock();

        // Cancel existing timer
        if let Some(ref mut timer) = *timer_guard {
            timer.cancel();
        }

        // Create new timer
        let callback = self.callback.clone();
        *timer_guard = Some(CancellableTimer::new(self.delay, move || {
            callback();
        }));
    }
}
