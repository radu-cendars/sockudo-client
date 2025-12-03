//! Native WebSocket transport implementation using tokio-tungstenite.

#![cfg(not(target_arch = "wasm32"))]

use async_trait::async_trait;
use futures_util::{SinkExt, StreamExt};
use parking_lot::RwLock;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tracing::{debug, error, info};

use super::transport::{MessageCallback, Transport};
use crate::error::{Result, SockudoError};

/// Command to send to the WebSocket writer task
enum WriteCommand {
    SendText(String),
    SendPing,
    Close,
}

/// Native WebSocket transport
pub struct NativeTransport {
    /// Channel to send commands to writer
    write_tx: Arc<RwLock<Option<mpsc::Sender<WriteCommand>>>>,
    /// Connected flag
    connected: Arc<RwLock<bool>>,
    /// Message callback
    on_message: Arc<RwLock<Option<MessageCallback>>>,
    /// Close callback
    on_close: Arc<RwLock<Option<Box<dyn Fn(Option<u16>, Option<String>) + Send + Sync>>>>,
    /// Error callback
    on_error: Arc<RwLock<Option<Box<dyn Fn(String) + Send + Sync>>>>,
}

impl NativeTransport {
    /// Create a new native transport
    pub fn new() -> Self {
        Self {
            write_tx: Arc::new(RwLock::new(None)),
            connected: Arc::new(RwLock::new(false)),
            on_message: Arc::new(RwLock::new(None)),
            on_close: Arc::new(RwLock::new(None)),
            on_error: Arc::new(RwLock::new(None)),
        }
    }

    /// Spawn reader and writer tasks
    fn spawn_tasks(&self, url: String) -> Result<()> {
        let on_message = self.on_message.clone();
        let on_close = self.on_close.clone();
        let on_error = self.on_error.clone();
        let connected = self.connected.clone();
        let write_tx_arc = self.write_tx.clone();

        tokio::spawn(async move {
            // Connect
            let ws_stream = match connect_async(&url).await {
                Ok((stream, _)) => stream,
                Err(e) => {
                    error!("WebSocket connection failed: {:?}", e);
                    *connected.write() = false;
                    if let Some(ref callback) = *on_error.read() {
                        callback(format!("Connection failed: {:?}", e));
                    }
                    return;
                }
            };

            info!("WebSocket connected successfully");
            *connected.write() = true;

            // Split into reader and writer
            let (mut writer, mut reader) = ws_stream.split();

            // Create channel for write commands
            let (write_tx, mut write_rx) = mpsc::channel::<WriteCommand>(100);
            *write_tx_arc.write() = Some(write_tx);

            // Spawn writer task
            let connected_clone = connected.clone();
            tokio::spawn(async move {
                while let Some(cmd) = write_rx.recv().await {
                    let result = match cmd {
                        WriteCommand::SendText(text) => {
                            debug!("Sending text: {}", text);
                            writer.send(Message::Text(text)).await
                        }
                        WriteCommand::SendPing => {
                            debug!("Sending ping");
                            writer.send(Message::Ping(vec![])).await
                        }
                        WriteCommand::Close => {
                            debug!("Closing connection");
                            let _ = writer.send(Message::Close(None)).await;
                            break;
                        }
                    };

                    if let Err(e) = result {
                        error!("Write error: {:?}", e);
                        *connected_clone.write() = false;
                        break;
                    }
                }
                debug!("Writer task ended");
            });

            // Reader task (runs in this task)
            loop {
                match reader.next().await {
                    Some(Ok(message)) => match message {
                        Message::Text(text) => {
                            debug!("Received text message: {}", text);
                            if let Some(ref callback) = *on_message.read() {
                                callback(&text);
                            }
                        }
                        Message::Binary(_) => {
                            debug!("Received binary message (ignored)");
                        }
                        Message::Close(frame) => {
                            info!("Received close frame");
                            *connected.write() = false;

                            let (code, reason) = if let Some(cf) = frame {
                                (Some(cf.code.into()), Some(cf.reason.to_string()))
                            } else {
                                (None, None)
                            };

                            if let Some(ref callback) = *on_close.read() {
                                callback(code, reason);
                            }
                            break;
                        }
                        Message::Ping(_) => {
                            debug!("Received ping");
                        }
                        Message::Pong(_) => {
                            debug!("Received pong");
                        }
                        Message::Frame(_) => {
                            debug!("Received raw frame");
                        }
                    },
                    Some(Err(e)) => {
                        error!("WebSocket receive error: {:?}", e);
                        *connected.write() = false;

                        if let Some(ref callback) = *on_error.read() {
                            callback(format!("Receive error: {:?}", e));
                        }

                        if let Some(ref callback) = *on_close.read() {
                            callback(None, Some(format!("Error: {:?}", e)));
                        }
                        break;
                    }
                    None => {
                        info!("WebSocket stream ended");
                        *connected.write() = false;

                        if let Some(ref callback) = *on_close.read() {
                            callback(None, Some("Stream ended".to_string()));
                        }
                        break;
                    }
                }
            }

            debug!("Reader task ended");
        });

        Ok(())
    }
}

impl Default for NativeTransport {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Transport for NativeTransport {
    async fn connect(&mut self, url: &str) -> Result<()> {
        if self.is_connected() {
            return Err(SockudoError::invalid_state("Already connected"));
        }

        info!("Connecting to WebSocket: {}", url);

        self.spawn_tasks(url.to_string())?;

        // Wait a bit for connection to establish
        for _ in 0..50 {
            if self.is_connected() {
                return Ok(());
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }

        Err(SockudoError::connection("Connection timeout"))
    }

    async fn disconnect(&mut self) {
        if !self.is_connected() {
            return;
        }

        info!("Disconnecting WebSocket");

        let tx = self.write_tx.read().as_ref().cloned();
        if let Some(tx) = tx {
            let _ = tx.send(WriteCommand::Close).await;
        }

        *self.write_tx.write() = None;
        *self.connected.write() = false;

        info!("WebSocket disconnected");
    }

    async fn send(&self, message: &str) -> Result<()> {
        if !self.is_connected() {
            return Err(SockudoError::invalid_state("Not connected"));
        }

        debug!("Sending message: {}", message);

        let tx = self.write_tx.read().as_ref().cloned();
        if let Some(tx) = tx {
            tx.send(WriteCommand::SendText(message.to_string()))
                .await
                .map_err(|e| SockudoError::websocket(format!("Send failed: {:?}", e)))?;
            Ok(())
        } else {
            Err(SockudoError::invalid_state("Writer not available"))
        }
    }

    async fn ping(&self) -> Result<()> {
        if !self.is_connected() {
            return Err(SockudoError::invalid_state("Not connected"));
        }

        debug!("Sending ping");

        let tx = self.write_tx.read().as_ref().cloned();
        if let Some(tx) = tx {
            tx.send(WriteCommand::SendPing)
                .await
                .map_err(|e| SockudoError::websocket(format!("Ping failed: {:?}", e)))?;
            Ok(())
        } else {
            Err(SockudoError::invalid_state("Writer not available"))
        }
    }

    fn is_connected(&self) -> bool {
        *self.connected.read()
    }

    fn on_message(&mut self, callback: MessageCallback) {
        *self.on_message.write() = Some(callback);
    }

    fn on_close(&mut self, callback: Box<dyn Fn(Option<u16>, Option<String>) + Send + Sync>) {
        *self.on_close.write() = Some(callback);
    }

    fn on_error(&mut self, callback: Box<dyn Fn(String) + Send + Sync>) {
        *self.on_error.write() = Some(callback);
    }
}
