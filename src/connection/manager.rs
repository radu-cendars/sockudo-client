//! Connection manager for WebSocket lifecycle management.

use parking_lot::RwLock;
use std::sync::Arc;
use std::time::Duration;
#[cfg(not(target_arch = "wasm32"))]
use tokio::sync::mpsc;
use tracing::{debug, error, info};

use super::state::ConnectionState;
use crate::error::{Result, SockudoError};
use crate::events::EventDispatcher;
use crate::options::Config;
use crate::protocol::{Protocol, PusherEvent};
#[cfg(not(target_arch = "wasm32"))]
use crate::transports::{NativeTransport, Transport};
#[cfg(feature = "wasm")]
use serde_json::Value;

/// Commands that can be sent to the connection task
#[derive(Debug)]
enum ConnectionCommand {
    Connect,
    Disconnect,
    Send(String),
    Ping,
    SendPong,
    Shutdown,
}

/// Connection manager handles the WebSocket connection lifecycle
pub struct ConnectionManager {
    /// Configuration
    config: Arc<Config>,
    /// Current state
    state: Arc<RwLock<ConnectionState>>,
    /// Socket ID (assigned by server)
    socket_id: Arc<RwLock<Option<String>>>,
    /// Activity timeout (from server)
    activity_timeout: Arc<RwLock<Duration>>,
    /// Event dispatcher for connection events
    dispatcher: EventDispatcher,
    /// Command sender to connection task
    #[cfg(not(target_arch = "wasm32"))]
    command_tx: Arc<RwLock<Option<mpsc::Sender<ConnectionCommand>>>>,
    /// Message receiver from connection task
    #[cfg(not(target_arch = "wasm32"))]
    message_rx: Arc<RwLock<Option<mpsc::Receiver<PusherEvent>>>>,
    /// Reconnection attempt counter
    reconnect_attempts: Arc<RwLock<u32>>,
    /// Whether TLS is required
    using_tls: Arc<RwLock<bool>>,
}

impl ConnectionManager {
    /// Create a new connection manager
    pub fn new(config: Config) -> Self {
        let activity_timeout = config.activity_timeout;
        let using_tls = config.use_tls;

        Self {
            config: Arc::new(config),
            state: Arc::new(RwLock::new(ConnectionState::Initialized)),
            socket_id: Arc::new(RwLock::new(None)),
            activity_timeout: Arc::new(RwLock::new(activity_timeout)),
            dispatcher: EventDispatcher::new(),
            #[cfg(not(target_arch = "wasm32"))]
            command_tx: Arc::new(RwLock::new(None)),
            #[cfg(not(target_arch = "wasm32"))]
            message_rx: Arc::new(RwLock::new(None)),
            reconnect_attempts: Arc::new(RwLock::new(0)),
            using_tls: Arc::new(RwLock::new(using_tls)),
        }
    }

    /// Get current state
    pub fn state(&self) -> ConnectionState {
        *self.state.read()
    }

    /// Get socket ID
    pub fn socket_id(&self) -> Option<String> {
        self.socket_id.read().clone()
    }

    /// Check if connected
    pub fn is_connected(&self) -> bool {
        self.state().is_connected()
    }

    /// Check if using TLS
    pub fn is_using_tls(&self) -> bool {
        *self.using_tls.read()
    }

    /// Bind to connection events
    pub fn bind(
        &self,
        event_name: impl Into<String>,
        callback: impl Fn(&PusherEvent) + Send + Sync + 'static,
    ) -> u64 {
        self.dispatcher.bind(event_name, callback)
    }

    /// Unbind from connection events
    pub fn unbind(&self, event_name: Option<&str>, callback_id: Option<u64>) {
        self.dispatcher.unbind(event_name, callback_id);
    }

    /// Bind to all connection events (global binding)
    pub fn bind_global(&self, callback: impl Fn(&PusherEvent) + Send + Sync + 'static) -> u64 {
        self.dispatcher.bind_global(callback)
    }

    /// Connect to the server
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn connect(&self) -> Result<()> {
        if self.state().is_active() {
            return Ok(());
        }

        self.update_state(ConnectionState::Connecting);

        // Create channels for communication
        let (cmd_tx, cmd_rx) = mpsc::channel(32);
        let (msg_tx, msg_rx) = mpsc::channel(64);

        *self.command_tx.write() = Some(cmd_tx.clone());
        *self.message_rx.write() = Some(msg_rx);

        // Clone Arc references for the connection task
        let config = self.config.clone();
        let state = self.state.clone();
        let socket_id = self.socket_id.clone();
        let activity_timeout = self.activity_timeout.clone();
        let reconnect_attempts = self.reconnect_attempts.clone();
        let using_tls = self.using_tls.clone();

        // Clone cmd_tx for the connection task
        let cmd_tx_for_task = cmd_tx.clone();

        // Spawn the connection task
        tokio::spawn(async move {
            connection_task(
                config,
                state,
                socket_id,
                activity_timeout,
                reconnect_attempts,
                using_tls,
                cmd_rx,
                cmd_tx_for_task,
                msg_tx,
            )
            .await
        });

        // Spawn message processing task
        let dispatcher = self.dispatcher.clone();
        let msg_rx_arc = self.message_rx.clone();
        tokio::spawn(async move {
            loop {
                // Take the receiver out of the Arc temporarily
                let receiver_opt = msg_rx_arc.write().take();

                if let Some(mut rx) = receiver_opt {
                    // Now we can await without holding the lock
                    match rx.recv().await {
                        Some(event) => {
                            // Emit connection-specific events
                            if event.event == "pusher:connection_established" {
                                let mut connected_event = event.clone();
                                connected_event.event = "connected".to_string();
                                dispatcher.emit(&connected_event);
                            } else if event.event == "pusher:error" {
                                let mut error_event = event.clone();
                                error_event.event = "error".to_string();
                                dispatcher.emit(&error_event);
                            }

                            // Also emit the raw event
                            dispatcher.emit(&event);

                            // Put the receiver back
                            *msg_rx_arc.write() = Some(rx);
                        }
                        None => {
                            // Channel closed
                            break;
                        }
                    }
                } else {
                    // No receiver available
                    break;
                }
            }
        });

        // Send connect command
        #[cfg(not(target_arch = "wasm32"))]
        {
            let tx = self.command_tx.read().clone();
            if let Some(tx) = tx {
                tx.send(ConnectionCommand::Connect)
                    .await
                    .map_err(|_| SockudoError::connection("Failed to send connect command"))?;
            }
        }

        Ok(())
    }

    /// Disconnect from the server
    pub async fn disconnect(&self) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let tx = self.command_tx.read().clone();
            if let Some(tx) = tx {
                let _ = tx.send(ConnectionCommand::Disconnect).await;
            }
        }

        self.update_state(ConnectionState::Disconnected);
        *self.socket_id.write() = None;
    }

    /// Send a raw message
    pub fn send(&self, message: &str) -> bool {
        if !self.is_connected() {
            return false;
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Some(ref tx) = self.command_tx.read().as_ref() {
                return tx
                    .try_send(ConnectionCommand::Send(message.to_string()))
                    .is_ok();
            }
        }

        false
    }

    /// Send an event (WASM version)
    #[cfg(feature = "wasm")]
    pub fn send_event(&self, event_name: &str, data: &Value, channel: Option<&str>) -> bool {
        let mut event = PusherEvent::new(event_name);
        event.data = Some(data.clone());
        if let Some(ch) = channel {
            event.channel = Some(ch.to_string());
        }

        match Protocol::encode_message(&event) {
            Ok(msg) => self.send(&msg),
            Err(e) => {
                error!("Failed to encode event: {}", e);
                false
            }
        }
    }

    /// Send an event (FFI version)
    #[cfg(not(feature = "wasm"))]
    pub fn send_event(&self, event_name: &str, data: &str, channel: Option<&str>) -> bool {
        let mut event = PusherEvent::new(event_name);
        event.data = Some(data.to_string());
        if let Some(ch) = channel {
            event.channel = Some(ch.to_string());
        }

        match Protocol::encode_message(&event) {
            Ok(msg) => self.send(&msg),
            Err(e) => {
                error!("Failed to encode event: {}", e);
                false
            }
        }
    }

    /// Update connection state and emit events
    fn update_state(&self, new_state: ConnectionState) {
        let previous = *self.state.read();
        *self.state.write() = new_state;

        if previous != new_state {
            debug!("State changed: {} -> {}", previous, new_state);

            // Emit state_change event
            let mut event = PusherEvent::new("state_change");
            let state_data = serde_json::json!({
                "previous": previous.to_string(),
                "current": new_state.to_string(),
            });

            #[cfg(feature = "wasm")]
            {
                event.data = Some(state_data);
            }
            #[cfg(not(feature = "wasm"))]
            {
                event.data = Some(state_data.to_string());
            }

            self.dispatcher.emit(&event);

            // Emit specific state event
            self.dispatcher
                .emit(&PusherEvent::new(new_state.to_string()));
        }
    }

    /// Process incoming messages
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn process_messages(&self) {
        let mut rx = match self.message_rx.write().take() {
            Some(rx) => rx,
            None => return,
        };

        while let Some(event) = rx.recv().await {
            self.dispatcher.emit(&event);
        }
    }
}

/// Connection task that manages the actual WebSocket connection
#[cfg(not(target_arch = "wasm32"))]
async fn connection_task(
    config: Arc<Config>,
    state: Arc<RwLock<ConnectionState>>,
    socket_id: Arc<RwLock<Option<String>>>,
    activity_timeout: Arc<RwLock<Duration>>,
    reconnect_attempts: Arc<RwLock<u32>>,
    using_tls: Arc<RwLock<bool>>,
    mut cmd_rx: mpsc::Receiver<ConnectionCommand>,
    cmd_tx: mpsc::Sender<ConnectionCommand>,
    msg_tx: mpsc::Sender<PusherEvent>,
) {
    use tokio::time::interval;

    let mut transport = NativeTransport::new();
    let mut ping_interval = interval(Duration::from_secs(30));

    loop {
        tokio::select! {
            // Handle commands
            Some(cmd) = cmd_rx.recv() => {
                match cmd {
                    ConnectionCommand::Connect => {
                        info!("Connecting to {}", config.ws_url);

                        // Set up message callback
                        let msg_tx_clone = msg_tx.clone();
                        let state_clone = state.clone();
                        let socket_id_clone = socket_id.clone();
                        let cmd_tx_clone = cmd_tx.clone();

                        transport.on_message(Box::new(move |message| {
                            if let Ok(event) = Protocol::decode_message(message) {
                                // Handle pusher:ping - respond with pusher:pong immediately
                                if event.event == "pusher:ping" {
                                    debug!("Received pusher:ping, sending pusher:pong");
                                    // Send command to send pong
                                    let _ = cmd_tx_clone.try_send(ConnectionCommand::SendPong);
                                }

                                // Handle connection:established event
                                if event.event == "pusher:connection_established" {
                                    if let Some(ref data) = event.data {
                                        #[cfg(feature = "wasm")]
                                        let parsed_data = Some(data.clone());
                                        #[cfg(not(feature = "wasm"))]
                                        let parsed_data = serde_json::from_str::<serde_json::Value>(data).ok();

                                        if let Some(parsed) = parsed_data {
                                            if let Some(sid) = parsed.get("socket_id").and_then(|v| v.as_str()) {
                                                *socket_id_clone.write() = Some(sid.to_string());
                                            }
                                        }
                                    }
                                    *state_clone.write() = ConnectionState::Connected;
                                }

                                let _ = msg_tx_clone.try_send(event);
                            }
                        }));

                        // Set up close callback
                        let state_clone = state.clone();
                        transport.on_close(Box::new(move |_code, _reason| {
                            *state_clone.write() = ConnectionState::Disconnected;
                        }));

                        // Set up error callback
                        let state_clone = state.clone();
                        transport.on_error(Box::new(move |_error| {
                            *state_clone.write() = ConnectionState::Unavailable;
                        }));

                        // Connect
                        match transport.connect(&config.ws_url).await {
                            Ok(_) => {
                                *reconnect_attempts.write() = 0;
                            }
                            Err(e) => {
                                error!("Failed to connect: {:?}", e);
                                *state.write() = ConnectionState::Unavailable;
                            }
                        }
                    }
                    ConnectionCommand::Disconnect => {
                        transport.disconnect().await;
                        *state.write() = ConnectionState::Disconnected;
                        break;
                    }
                    ConnectionCommand::Send(msg) => {
                        if let Err(e) = transport.send(&msg).await {
                            error!("Failed to send message: {:?}", e);
                        }
                    }
                    ConnectionCommand::Ping => {
                        if let Err(e) = transport.ping().await {
                            error!("Failed to send ping: {:?}", e);
                        }
                    }
                    ConnectionCommand::SendPong => {
                        let pong_event = Protocol::create_pong_event();
                        if let Ok(pong_msg) = Protocol::encode_message(&pong_event) {
                            debug!("Sending pusher:pong");
                            if let Err(e) = transport.send(&pong_msg).await {
                                error!("Failed to send pong: {:?}", e);
                            }
                        }
                    }
                    ConnectionCommand::Shutdown => {
                        transport.disconnect().await;
                        break;
                    }
                }
            }

            // Periodic ping
            _ = ping_interval.tick() => {
                if *state.read() == ConnectionState::Connected {
                    let _ = transport.ping().await;
                }
            }
        }
    }
}

impl std::fmt::Debug for ConnectionManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConnectionManager")
            .field("state", &self.state())
            .field("socket_id", &self.socket_id())
            .field("using_tls", &self.is_using_tls())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::options::PusherOptions;

    #[test]
    fn test_connection_manager_creation() {
        let options = PusherOptions::new("test-key").cluster("mt1");
        let config = Config::from(options);
        let manager = ConnectionManager::new(config);

        assert_eq!(manager.state(), ConnectionState::Initialized);
        assert!(manager.socket_id().is_none());
    }
}
