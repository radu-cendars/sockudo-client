//! Cross-platform signal handling utilities.
//!
//! This module provides a unified interface for handling OS signals across
//! Windows and Unix-like platforms (Linux, macOS, etc.).
//!
//! Note: This module is not available for WASM targets and requires tokio with signal support.

#![cfg(all(not(target_arch = "wasm32"), feature = "native"))]

use std::io;

#[cfg(unix)]
use tokio::signal::unix::{signal, SignalKind};

/// Wait for a termination signal (Ctrl+C or SIGTERM on Unix).
///
/// This function works across platforms:
/// - On Windows: Waits for Ctrl+C
/// - On Unix: Waits for SIGINT (Ctrl+C) or SIGTERM
///
/// # Examples
///
/// ```no_run
/// use sockudo_client::utils::wait_for_signal;
///
/// #[tokio::main]
/// async fn main() {
///     println!("Running... Press Ctrl+C to stop");
///     wait_for_signal().await;
///     println!("Shutting down gracefully...");
/// }
/// ```
#[cfg(windows)]
pub async fn wait_for_signal() {
    use tokio::signal::windows;

    let mut ctrl_c = windows::ctrl_c().expect("Failed to register Ctrl+C handler");

    ctrl_c.recv().await;
    tracing::info!("Received Ctrl+C signal");
}

/// Wait for a termination signal (Ctrl+C or SIGTERM on Unix).
///
/// This function works across platforms:
/// - On Windows: Waits for Ctrl+C
/// - On Unix: Waits for SIGINT (Ctrl+C) or SIGTERM
///
/// # Examples
///
/// ```no_run
/// use sockudo_client::utils::wait_for_signal;
///
/// #[tokio::main]
/// async fn main() {
///     println!("Running... Press Ctrl+C to stop");
///     wait_for_signal().await;
///     println!("Shutting down gracefully...");
/// }
/// ```
#[cfg(unix)]
pub async fn wait_for_signal() {
    let mut sigterm = signal(SignalKind::terminate()).expect("Failed to register SIGTERM handler");
    let mut sigint = signal(SignalKind::interrupt()).expect("Failed to register SIGINT handler");

    tokio::select! {
        _ = sigterm.recv() => {
            tracing::info!("Received SIGTERM signal");
        }
        _ = sigint.recv() => {
            tracing::info!("Received SIGINT signal");
        }
    }
}

/// Wait for a termination signal with a result type.
///
/// Similar to `wait_for_signal()` but returns a Result for easier error handling.
///
/// # Examples
///
/// ```no_run
/// use sockudo_client::utils::wait_for_signal_result;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     println!("Running... Press Ctrl+C to stop");
///     wait_for_signal_result().await?;
///     println!("Shutting down gracefully...");
///     Ok(())
/// }
/// ```
#[cfg(windows)]
pub async fn wait_for_signal_result() -> io::Result<()> {
    use tokio::signal::windows;

    let mut ctrl_c = windows::ctrl_c()?;
    ctrl_c.recv().await;
    tracing::info!("Received Ctrl+C signal");
    Ok(())
}

/// Wait for a termination signal with a result type.
///
/// Similar to `wait_for_signal()` but returns a Result for easier error handling.
///
/// # Examples
///
/// ```no_run
/// use sockudo_client::utils::wait_for_signal_result;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     println!("Running... Press Ctrl+C to stop");
///     wait_for_signal_result().await?;
///     println!("Shutting down gracefully...");
///     Ok(())
/// }
/// ```
#[cfg(unix)]
pub async fn wait_for_signal_result() -> io::Result<()> {
    let mut sigterm = signal(SignalKind::terminate())?;
    let mut sigint = signal(SignalKind::interrupt())?;

    tokio::select! {
        _ = sigterm.recv() => {
            tracing::info!("Received SIGTERM signal");
        }
        _ = sigint.recv() => {
            tracing::info!("Received SIGINT signal");
        }
    }
    Ok(())
}

/// A cross-platform signal handler that can wait for multiple types of signals.
///
/// # Examples
///
/// ```no_run
/// use sockudo_client::utils::SignalHandler;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let mut signal_handler = SignalHandler::new()?;
///
///     println!("Running... Press Ctrl+C to stop");
///     signal_handler.wait().await;
///     println!("Shutting down gracefully...");
///
///     Ok(())
/// }
/// ```
pub struct SignalHandler {
    #[cfg(windows)]
    ctrl_c: tokio::signal::windows::CtrlC,
    #[cfg(windows)]
    ctrl_break: Option<tokio::signal::windows::CtrlBreak>,

    #[cfg(unix)]
    sigterm: tokio::signal::unix::Signal,
    #[cfg(unix)]
    sigint: tokio::signal::unix::Signal,
    #[cfg(unix)]
    sighup: Option<tokio::signal::unix::Signal>,
}

impl SignalHandler {
    /// Create a new signal handler with default signals.
    ///
    /// On Windows: Handles Ctrl+C and Ctrl+Break
    /// On Unix: Handles SIGINT, SIGTERM, and SIGHUP
    pub fn new() -> io::Result<Self> {
        Self::with_options(true)
    }

    /// Create a new signal handler with custom options.
    ///
    /// # Arguments
    ///
    /// * `include_optional` - Whether to include optional signals (Ctrl+Break on Windows, SIGHUP on Unix)
    pub fn with_options(include_optional: bool) -> io::Result<Self> {
        #[cfg(windows)]
        {
            use tokio::signal::windows;

            let ctrl_c = windows::ctrl_c()?;
            let ctrl_break = if include_optional {
                Some(windows::ctrl_break()?)
            } else {
                None
            };

            Ok(Self { ctrl_c, ctrl_break })
        }

        #[cfg(unix)]
        {
            let sigterm = signal(SignalKind::terminate())?;
            let sigint = signal(SignalKind::interrupt())?;
            let sighup = if include_optional {
                Some(signal(SignalKind::hangup())?)
            } else {
                None
            };

            Ok(Self {
                sigterm,
                sigint,
                sighup,
            })
        }
    }

    /// Wait for any registered signal.
    ///
    /// This method will block until one of the registered signals is received.
    pub async fn wait(&mut self) {
        #[cfg(windows)]
        {
            if let Some(ref mut ctrl_break) = self.ctrl_break {
                tokio::select! {
                    _ = self.ctrl_c.recv() => {
                        tracing::info!("Received Ctrl+C signal");
                    }
                    _ = ctrl_break.recv() => {
                        tracing::info!("Received Ctrl+Break signal");
                    }
                }
            } else {
                self.ctrl_c.recv().await;
                tracing::info!("Received Ctrl+C signal");
            }
        }

        #[cfg(unix)]
        {
            if let Some(ref mut sighup) = self.sighup {
                tokio::select! {
                    _ = self.sigterm.recv() => {
                        tracing::info!("Received SIGTERM signal");
                    }
                    _ = self.sigint.recv() => {
                        tracing::info!("Received SIGINT signal");
                    }
                    _ = sighup.recv() => {
                        tracing::info!("Received SIGHUP signal");
                    }
                }
            } else {
                tokio::select! {
                    _ = self.sigterm.recv() => {
                        tracing::info!("Received SIGTERM signal");
                    }
                    _ = self.sigint.recv() => {
                        tracing::info!("Received SIGINT signal");
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_signal_handler_creation() {
        // Just test that we can create a signal handler without panicking
        let result = SignalHandler::new();
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_signal_handler_with_options() {
        let result = SignalHandler::with_options(false);
        assert!(result.is_ok());

        let result = SignalHandler::with_options(true);
        assert!(result.is_ok());
    }
}
