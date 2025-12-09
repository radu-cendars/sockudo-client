//! Main Sockudo/Pusher client implementation.

use parking_lot::RwLock;
use std::sync::Arc;
use tracing::{debug, info, warn};

#[cfg(not(feature = "uniffi"))]
use crate::auth::AuthClient;
use crate::channels::{Channel, Channels, PresenceChannel};
use crate::connection::{ConnectionManager, ConnectionState};
use crate::delta::DeltaManager;
use crate::error::{Result, SockudoError};
use crate::events::EventDispatcher;
#[cfg(feature = "uniffi")]
use crate::ffi_callbacks::EventCallback;
use crate::options::{Config, SockudoOptions};
use crate::protocol::{FilterOp, Protocol};
use crate::PusherEvent;

/// The main Sockudo client for connecting to Pusher-compatible servers.
///
/// This is the primary interface for subscribing to channels and receiving
/// real-time events from a Pusher-compatible WebSocket server.
///
/// # Example
///
/// ```ignore
/// use sockudo_client::{SockudoClient, PusherOptions};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let options = PusherOptions::new("your-app-key")
///         .cluster("mt1")
///         .ws_host("your-server.com")
///         .ws_port(6001)
///         .use_tls(false);
///
///     let mut client = SockudoClient::new(options)?;
///     client.connect().await?;
///
///     let channel = client.subscribe("my-channel")?;
///     // Events are handled via callbacks...
///
///     Ok(())
/// }
/// ```
#[cfg(not(target_arch = "wasm32"))]
#[cfg_attr(feature = "uniffi", derive(uniffi::Object))]
pub struct SockudoClient {
    /// Application key
    key: String,
    /// Configuration
    config: Arc<Config>,
    /// Channel management
    channels: Arc<Channels>,
    /// Global event dispatcher
    global_emitter: EventDispatcher,
    /// Connection manager
    pub(crate) connection: Arc<ConnectionManager>,
    /// Session ID (random per client instance)
    session_id: u32,
    /// Delta compression manager
    delta_manager: Option<Arc<RwLock<DeltaManager>>>,
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(feature = "uniffi")]
#[uniffi::export]
impl SockudoClient {
    /// Create a new Sockudo client with the given options (FFI version).
    #[uniffi::constructor]
    pub fn new(options: crate::ffi_types::SockudoOptions) -> Result<Self> {
        let options: SockudoOptions = options.into();

        if options.app_key.is_empty() {
            return Err(SockudoError::config("App key is required"));
        }

        let config: Config = options.clone().into();
        let config = Arc::new(config);

        // Create channels with callbacks
        let mut channels = Channels::new();

        // Create delta manager if enabled
        let delta_manager = if let Some(delta_opts) = config.delta_compression.clone() {
            if delta_opts.enabled {
                let dm = DeltaManager::new(delta_opts);
                Some(Arc::new(RwLock::new(dm)))
            } else {
                None
            }
        } else {
            None
        };

        // Generate session ID
        let session_id = rand::random::<u32>();

        // Save app_key before moving options
        let app_key = options.app_key.clone();

        info!(
            "Creating Sockudo client for app '{}' (session: {})",
            app_key, session_id
        );

        let connection = Arc::new(ConnectionManager::new(Config::from(options)));

        // Set up send callback for channels
        let connection_clone = connection.clone();
        channels.set_send_callback(Arc::new(move |event_name, data, channel| {
            let mut event = PusherEvent::new(event_name);
            #[cfg(feature = "wasm")]
            {
                event.data = Some(data.clone());
            }
            #[cfg(not(feature = "wasm"))]
            {
                event.data = Some(data.to_string());
            }
            event.channel = channel.map(|s| s.to_string());

            match Protocol::encode_message(&event) {
                Ok(msg) => connection_clone.send(&msg),
                Err(_) => false,
            }
        }));

        // Set up send callback for delta manager
        if let Some(ref dm) = delta_manager {
            let connection_for_delta = connection.clone();
            dm.write()
                .set_send_callback(Arc::new(move |event_name, data| {
                    let mut event = PusherEvent::new(event_name);
                    #[cfg(feature = "wasm")]
                    {
                        event.data = Some(data.clone());
                    }
                    #[cfg(not(feature = "wasm"))]
                    {
                        event.data = Some(data.to_string());
                    }

                    match Protocol::encode_message(&event) {
                        Ok(msg) => connection_for_delta.send(&msg),
                        Err(_) => false,
                    }
                }));
        }

        // Set up authorization callback for private/presence channels
        // Note: uniffi doesn't support async callbacks easily, so we use blocking
        if !config.auth_endpoint.is_empty() {
            let auth_endpoint = config.auth_endpoint.clone();
            let auth_headers = config.auth_headers.clone();

            channels.set_authorize_callback(Arc::new(move |channel_name, socket_id| {
                use crate::auth::AuthClient;

                let auth_client = AuthClient::new(
                    Some(auth_endpoint.clone()),
                    Some(auth_headers.clone()),
                    None,
                    None,
                );

                // Use block_in_place to allow blocking in async context
                // Note: This is only called in non-WASM builds because WASM uses async subscribe
                tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(async {
                        auth_client.authorize_channel(channel_name, socket_id).await
                    })
                })
            }));
        }

        Ok(Self {
            key: app_key,
            config,
            channels: Arc::new(channels),
            global_emitter: EventDispatcher::new(),
            connection,
            session_id,
            delta_manager,
        })
    }

    /// Get the application key.
    pub fn key(&self) -> String {
        self.key.clone()
    }

    /// Get the session ID.
    pub fn session_id(&self) -> u32 {
        self.session_id
    }

    /// Get the current connection state.
    pub fn state(&self) -> ConnectionState {
        self.connection.state()
    }

    /// Get the socket ID assigned by the server.
    pub fn socket_id(&self) -> Option<String> {
        self.connection.socket_id()
    }

    /// Check if the client is connected.
    pub fn is_connected(&self) -> bool {
        self.connection.is_connected()
    }

    /// Connect to the Pusher server.
    ///
    /// This initiates a WebSocket connection to the configured server.
    /// The connection is established asynchronously and events will be
    /// emitted as the connection state changes.
    #[cfg(not(target_arch = "wasm32"))]
    /// Wait for the client to be connected (with timeout).
    ///
    /// This is useful after calling `connect()` to wait for the connection to be established.
    /// Returns an error if the connection is not established within the timeout period.
    pub async fn wait_for_connection(&self, timeout_secs: u64) -> Result<()> {
        use tokio::time::{timeout, Duration};

        let wait_result = timeout(Duration::from_secs(timeout_secs), async {
            while !self.is_connected() {
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        })
        .await;

        if wait_result.is_err() {
            return Err(SockudoError::connection(&format!(
                "Connection timeout - failed to connect within {} seconds",
                timeout_secs
            )));
        }

        Ok(())
    }

    pub async fn connect(&self) -> Result<()> {
        // Set up connection callbacks before connecting
        let channels = self.channels.clone();
        let global_emitter = self.global_emitter.clone();
        let delta_manager = self.delta_manager.clone();

        // Bind to all connection events globally and route them
        let channels_for_events = self.channels.clone();
        let global_emitter_for_events = self.global_emitter.clone();
        let delta_manager_for_events = self.delta_manager.clone();

        self.connection.bind_global(move |event| {
            // Debug: log all events
            debug!(
                "Received event: '{}' on channel {:?}",
                event.event, event.channel
            );

            // Handle delta compression protocol events first
            if let Some(ref dm) = delta_manager_for_events {
                match event.event.as_str() {
                    "pusher:delta_compression_enabled" => {
                        if let Some(ref data) = event.data {
                            #[cfg(feature = "wasm")]
                            {
                                dm.write().handle_enabled(data);
                            }
                            #[cfg(not(feature = "wasm"))]
                            {
                                if let Ok(value) = serde_json::from_str::<serde_json::Value>(data) {
                                    dm.write().handle_enabled(&value);
                                    debug!("Delta compression enabled by server");
                                }
                            }
                        }
                        return;
                    }
                    "pusher:delta_cache_sync" => {
                        if let (Some(ref channel), Some(ref data)) = (&event.channel, &event.data) {
                            #[cfg(feature = "wasm")]
                            let sync_result: std::result::Result<crate::delta::CacheSyncData, _> = serde_json::from_value(data.clone());
                            #[cfg(not(feature = "wasm"))]
                            let sync_result: std::result::Result<crate::delta::CacheSyncData, _> = serde_json::from_str(data);

                            if let Ok(sync_data) = sync_result {
                                dm.write().handle_cache_sync(channel, sync_data);
                                debug!("Delta cache sync for channel: {}", channel);
                            }
                        }
                        return;
                    }
                    "pusher:delta" => {
                        if let Some(ref channel) = event.channel {
                            if let Some(ref data) = event.data {
                                #[cfg(feature = "wasm")]
                                let delta_result: std::result::Result<crate::delta::DeltaMessage, _> = serde_json::from_value(data.clone());
                                #[cfg(not(feature = "wasm"))]
                                let delta_result: std::result::Result<crate::delta::DeltaMessage, _> = serde_json::from_str(data);

                                if let Ok(delta_msg) = delta_result {
                                    match dm.read().handle_delta(channel, delta_msg) {
                                        Ok(decoded_event) => {
                                            // Route the decoded event to the channel
                                            if let Some(ch) = channels_for_events.find(channel) {
                                                ch.handle_event(&decoded_event);
                                            }
                                            // Also emit globally
                                            global_emitter_for_events.emit(&decoded_event);
                                            debug!("Delta decoded and routed for channel: {}", channel);
                                        }
                                        Err(e) => {
                                            warn!("Failed to handle delta: {}", e);
                                            dm.read().request_resync(channel);
                                        }
                                    }
                                    return;
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }

            // Check if this is an internal event (like pusher-js does)
            let is_internal = event.event.starts_with("pusher_internal:");

            // Route to channel if specified
            if let Some(ref channel_name) = event.channel {
                debug!(
                    "Routing event '{}' to channel '{}'",
                    event.event, channel_name
                );

                // Route to channel - dispatchers are now shared so this works correctly
                if let Some(channel) = channels_for_events.find(channel_name) {
                    channel.handle_event(event);
                    debug!("Event routed to channel '{}'", channel_name);
                } else {
                    warn!(
                        "Channel '{}' not found for event '{}'",
                        channel_name, event.event
                    );
                }
            }

            // Emit globally (except internal events, like pusher-js does)
            if !is_internal {
                global_emitter_for_events.emit(event);
            }
        });

        let connection = self.connection.clone();
        let config_for_resubscribe = self.config.clone();
        self.connection.bind("connected", move |_event| {
            info!("Connected to Pusher");

            // Enable delta compression if configured
            if let Some(ref dm) = delta_manager {
                dm.read().enable();
            }

            // Resubscribe to all channels
            if let Some(socket_id) = connection.socket_id() {
                let all_channels = channels.all();
                info!(
                    "Resubscribing {} channels after connection established",
                    all_channels.len()
                );

                #[cfg(not(target_arch = "wasm32"))]
                {
                    // Native: Use synchronous subscribe with callback-based auth
                    for channel in all_channels {
                        info!(
                            "Channel '{}' state: subscribed={}, pending={}",
                            channel.name(),
                            channel.is_subscribed(),
                            channel.is_subscription_pending()
                        );

                        if !channel.is_subscribed() && !channel.is_subscription_pending() {
                            info!("Attempting to subscribe to channel: {}", channel.name());
                            if let Err(e) = channel.subscribe(&socket_id) {
                                warn!("Failed to resubscribe to channel {}: {}", channel.name(), e);
                            } else {
                                info!(
                                    "Successfully sent subscription for channel: {}",
                                    channel.name()
                                );
                            }
                        } else if channel.is_subscription_pending() {
                            info!(
                                "Channel {} already has subscription pending",
                                channel.name()
                            );
                        } else {
                            info!("Channel {} already subscribed", channel.name());
                        }
                    }
                }

                #[cfg(target_arch = "wasm32")]
                {
                    // WASM: Spawn async tasks for subscription with async auth
                    let config_clone = config_for_resubscribe.clone();
                    for channel in all_channels {
                        if !channel.is_subscribed() && !channel.is_subscription_pending() {
                            let channel = channel.clone();
                            let socket_id = socket_id.clone();
                            let auth_endpoint = config_clone.auth_endpoint.clone();

                            wasm_bindgen_futures::spawn_local(async move {
                                let auth_ep = if !auth_endpoint.is_empty() {
                                    Some(auth_endpoint.as_str())
                                } else {
                                    None
                                };

                                if let Err(e) = channel.subscribe_async(&socket_id, auth_ep).await {
                                    warn!(
                                        "Failed to resubscribe to channel {}: {}",
                                        channel.name(),
                                        e
                                    );
                                }
                            });
                        }
                    }
                }
            } else {
                warn!("No socket_id available for resubscription");
            }
        });

        // Now connect
        self.connection.connect().await?;

        Ok(())
    }

    /// Disconnect from the server.
    pub async fn disconnect(&self) {
        info!("Disconnecting from Pusher");

        // Call disconnect - no lock held across await since disconnect() uses &self
        self.connection.disconnect().await;
        self.channels.disconnect();
    }

    /// Bind a callback to a global event (across all channels).
    ///
    /// # Example
    ///
    /// ```ignore
    /// # use sockudo_client::{SockudoClient, PusherOptions};
    /// # let client = SockudoClient::new(PusherOptions::new("key").into()).unwrap();
    /// client.bind("my-event", |event| {
    ///     println!("Received: {:?}", event);
    /// });
    /// ```
    pub fn bind(&self, event_name: String, callback: Box<dyn EventCallback>) {
        let callback = Arc::new(callback);
        self.global_emitter.bind(event_name, move |event| {
            let ffi_event = crate::UniffiPusherEvent {
                event: event.event.clone(),
                channel: event.channel.clone(),
                data: event.data.as_ref().map(|v| v.to_string()),
                user_id: event.user_id.clone(),
            };
            callback.on_event(ffi_event);
        });
    }

    /// Bind a callback to all events globally (FFI version).
    ///
    /// Note: This is the FFI-specific version. Rust code should use the
    /// bind_global method that accepts closures directly.
    pub fn bind_global_ffi(&self, callback: Box<dyn EventCallback>) {
        let callback = Arc::new(callback);
        self.global_emitter.bind_global(move |event| {
            let ffi_event = crate::UniffiPusherEvent {
                event: event.event.clone(),
                channel: event.channel.clone(),
                data: event.data.as_ref().map(|v| v.to_string()),
                user_id: event.user_id.clone(),
            };
            callback.on_event(ffi_event);
        });
    }

    /// Unbind callbacks from an event.
    pub fn unbind(&self, event_name: Option<String>, callback_id: Option<u64>) {
        self.global_emitter
            .unbind(event_name.as_deref(), callback_id);
    }

    /// Unbind global callbacks.
    pub fn unbind_global(&self, callback_id: Option<u64>) {
        self.global_emitter.unbind_global(callback_id);
    }

    /// Unbind all callbacks.
    pub fn unbind_all(&self) {
        self.global_emitter.unbind_all();
    }

    /// Subscribe to a channel.
    ///
    /// # Example
    ///
    /// ```ignore
    /// # use sockudo_client::{SockudoClient, PusherOptions};
    /// # let client = SockudoClient::new(PusherOptions::new("key").into()).unwrap();
    /// let channel = client.subscribe("my-channel").unwrap();
    /// channel.bind("my-event", |event| {
    ///     println!("Event on channel: {:?}", event);
    /// });
    /// ```
    pub fn subscribe(&self, channel_name: &str) -> Result<Arc<Channel>> {
        self.subscribe_with_filter(channel_name, None)
    }

    /// Subscribe to a channel with a tags filter.
    ///
    /// Tags filtering allows the server to only send events that match
    /// the specified filter criteria.
    ///
    /// # Example
    ///
    /// ```ignore
    /// # use sockudo_client::{SockudoClient, PusherOptions, FilterOp};
    /// # let client = SockudoClient::new(PusherOptions::new("key").into()).unwrap();
    /// let filter = FilterOp::eq("type", "goal");
    /// let channel = client.subscribe_with_filter("sports", Some(filter)).unwrap();
    /// ```
    pub fn subscribe_with_filter(
        &self,
        channel_name: &str,
        filter: Option<FilterOp>,
    ) -> Result<Arc<Channel>> {
        // Validate channel name
        if channel_name.starts_with('#') {
            return Err(SockudoError::invalid_channel(format!(
                "Channel names cannot start with '#': {}",
                channel_name
            )));
        }

        // Get or create channel
        let channel = self.channels.add(channel_name)?;

        // Set filter if provided
        if let Some(f) = filter {
            channel.set_tags_filter(Some(f));
        }

        // Subscribe if connected
        if let Some(socket_id) = self.socket_id() {
            channel.subscribe(&socket_id)?;
        }

        debug!("Subscribed to channel: {}", channel_name);
        Ok(channel)
    }

    /// Unsubscribe from a channel.
    pub fn unsubscribe(&self, channel_name: &str) {
        if let Some(channel) = self.channels.find(channel_name) {
            channel.unsubscribe();
        }
        self.channels.remove(channel_name);
        debug!("Unsubscribed from channel: {}", channel_name);
    }

    /// Get a channel by name.
    pub fn channel(&self, name: &str) -> Option<Arc<Channel>> {
        self.channels.find(name)
    }

    /// Subscribe to a presence channel and return the PresenceChannel instance.
    ///
    /// This is useful when you need access to presence-specific features like
    /// member tracking. For presence channels, use this instead of `subscribe()`.
    ///
    /// # Example
    ///
    /// ```ignore
    /// # use sockudo_client::{SockudoClient, PusherOptions};
    /// # let client = SockudoClient::new(PusherOptions::new("key").into()).unwrap();
    /// let channel = client.subscribe_presence("presence-chat-room").unwrap();
    /// channel.bind("pusher:subscription_succeeded", |event| {
    ///     println!("Joined presence channel!");
    /// });
    /// ```
    pub fn subscribe_presence(&self, channel_name: &str) -> Result<Arc<PresenceChannel>> {
        // Validate it's a presence channel
        if !channel_name.starts_with("presence-") {
            return Err(SockudoError::invalid_channel(format!(
                "Channel name must start with 'presence-': {}",
                channel_name
            )));
        }

        // Create/get the channel
        let _ = self.channels.add(channel_name)?;

        // Get the presence channel
        let presence_channel = self.channels.find_presence(channel_name).ok_or_else(|| {
            SockudoError::invalid_channel(format!(
                "Failed to get presence channel: {}",
                channel_name
            ))
        })?;

        // Subscribe if connected
        if let Some(socket_id) = self.socket_id() {
            presence_channel.subscribe(&socket_id)?;
        }

        debug!("Subscribed to presence channel: {}", channel_name);
        Ok(presence_channel)
    }

    /// Get all subscribed channels.
    pub fn all_channels(&self) -> Vec<Arc<Channel>> {
        self.channels.all()
    }

    /// Send a custom event over the connection (FFI version).
    ///
    /// This is used for client events on private/presence channels.
    pub fn send_event(&self, event_name: String, data: String, channel: Option<String>) -> bool {
        #[cfg(feature = "wasm")]
        {
            let value: serde_json::Value =
                serde_json::from_str(&data).unwrap_or(serde_json::Value::String(data));
            self.connection
                .send_event(&event_name, &value, channel.as_deref())
        }
        #[cfg(not(feature = "wasm"))]
        {
            self.connection
                .send_event(&event_name, &data, channel.as_deref())
        }
    }

    /// Get delta compression statistics.
    pub fn get_delta_stats(&self) -> Option<crate::UniffiDeltaStats> {
        self.delta_manager
            .as_ref()
            .map(|dm| dm.read().get_stats().into())
    }

    /// Reset delta compression statistics.
    pub fn reset_delta_stats(&self) {
        if let Some(ref dm) = self.delta_manager {
            dm.write().reset_stats();
        }
    }

    /// Check if delta compression is enabled and active.
    pub fn is_delta_compression_enabled(&self) -> bool {
        self.delta_manager
            .as_ref()
            .map(|dm| dm.read().is_enabled())
            .unwrap_or(false)
    }
}

// Rust-native methods that accept closures (always available)
#[cfg(not(target_arch = "wasm32"))]
impl SockudoClient {
    /// Bind a callback to all events globally.
    ///
    /// This is the primary method for Rust code to bind global event handlers.
    /// It accepts closures for convenient event handling.
    ///
    /// # Example
    ///
    /// ```ignore
    /// # use sockudo_client::{SockudoClient, PusherOptions};
    /// # let client = SockudoClient::new(PusherOptions::new("key").into()).unwrap();
    /// client.bind_global(|event| {
    ///     println!("Event: {} on {:?}", event.event, event.channel);
    /// });
    /// ```
    #[cfg(feature = "uniffi")]
    pub fn bind_global(&self, callback: impl Fn(&PusherEvent) + Send + Sync + 'static) -> u64 {
        self.global_emitter.bind_global(callback)
    }

    /// Bind a callback to all events globally (non-uniffi version).
    #[cfg(not(feature = "uniffi"))]
    pub fn bind_global(&self, callback: impl Fn(&PusherEvent) + Send + Sync + 'static) -> u64 {
        self.global_emitter.bind_global(callback)
    }
}

// Private methods (not exported via uniffi)
#[cfg(not(target_arch = "wasm32"))]
#[cfg(feature = "uniffi")]
impl SockudoClient {
    /// Handle an incoming message from the connection.
    fn handle_message(&self, event: &PusherEvent) {
        let event_name = &event.event;

        // Handle delta compression protocol events
        if let Some(ref dm) = self.delta_manager {
            match event_name.as_str() {
                "pusher:delta_compression_enabled" => {
                    if let Some(ref data) = event.data {
                        #[cfg(feature = "wasm")]
                        {
                            dm.write().handle_enabled(data);
                        }
                        #[cfg(not(feature = "wasm"))]
                        {
                            if let Ok(value) = serde_json::from_str::<serde_json::Value>(data) {
                                dm.write().handle_enabled(&value);
                            }
                        }
                    }
                    return;
                }
                "pusher:delta_cache_sync" => {
                    if let (Some(ref channel), Some(ref data)) = (&event.channel, &event.data) {
                        #[cfg(feature = "wasm")]
                        let sync_result = serde_json::from_value(data.clone());
                        #[cfg(not(feature = "wasm"))]
                        let sync_result = serde_json::from_str(data);

                        if let Ok(sync_data) = sync_result {
                            dm.write().handle_cache_sync(channel, sync_data);
                        }
                    }
                    return;
                }
                "pusher:delta" => {
                    if let Some(ref channel) = event.channel {
                        if let Some(ref data) = event.data {
                            #[cfg(feature = "wasm")]
                            let delta_result = serde_json::from_value(data.clone());
                            #[cfg(not(feature = "wasm"))]
                            let delta_result = serde_json::from_str(data);

                            if let Ok(delta_msg) = delta_result {
                                match dm.read().handle_delta(channel, delta_msg) {
                                    Ok(decoded_event) => {
                                        // Route the decoded event to the channel
                                        if let Some(ch) = self.channels.find(channel) {
                                            ch.handle_event(&decoded_event);
                                        }
                                        // Also emit globally
                                        self.global_emitter.emit(&decoded_event);
                                    }
                                    Err(e) => {
                                        warn!("Failed to handle delta: {}", e);
                                        dm.read().request_resync(channel);
                                    }
                                }
                                return;
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        // Route to channel if specified
        if let Some(ref channel_name) = event.channel {
            if let Some(channel) = self.channels.find(channel_name) {
                channel.handle_event(event);

                // Track full message for delta compression
                if let Some(ref dm) = self.delta_manager {
                    #[cfg(feature = "wasm")]
                    let seq_opt = event
                        .data
                        .as_ref()
                        .and_then(|d| d.get("__delta_seq"))
                        .and_then(|v| v.as_u64());

                    #[cfg(not(feature = "wasm"))]
                    let seq_opt = event.data.as_ref().and_then(|d| {
                        serde_json::from_str::<serde_json::Value>(d)
                            .ok()
                            .and_then(|v| v.get("__delta_seq").and_then(|s| s.as_u64()))
                    });

                    if let Some(seq) = seq_opt {
                        dm.write().handle_full_message(channel_name, event, seq);
                    }
                }
            }
        }

        // Emit to global listeners (except internal events)
        if !event_name.starts_with("pusher_internal:") {
            self.global_emitter.emit(event);
        }
    }
}

// WASM-specific methods (outside uniffi export)
#[cfg(not(target_arch = "wasm32"))]
#[cfg(all(feature = "wasm", not(feature = "uniffi")))]
impl SockudoClient {
    /// Send an event to the server (WASM version).
    ///
    /// This is used for client events on private/presence channels.
    pub fn send_event(
        &self,
        event_name: &str,
        data: &serde_json::Value,
        channel: Option<&str>,
    ) -> bool {
        self.connection.send_event(event_name, data, channel)
    }
}

// Non-uniffi methods (for WASM and other non-FFI builds)
#[cfg(not(target_arch = "wasm32"))]
#[cfg(not(feature = "uniffi"))]
impl SockudoClient {
    /// Create a new Sockudo client (Pusher-JS compatible API).
    ///
    /// # Example
    /// ```no_run
    /// use sockudo::{SockudoClient, SockudoOptions};
    ///
    /// let client = SockudoClient::new("app-key", SockudoOptions {
    ///     cluster: Some("mt1".to_string()),
    ///     ..Default::default()
    /// }).await.unwrap();
    /// ```
    pub async fn new(app_key: impl Into<String>, mut options: SockudoOptions) -> Result<Self> {
        let app_key = app_key.into();
        if app_key.is_empty() {
            return Err(SockudoError::config("App key is required"));
        }

        // Set the app_key in options
        options.app_key = app_key.clone();

        // Create the client
        let client = Self::from_options(options)?;

        // Auto-connect (Pusher-JS behavior)
        #[cfg(not(target_arch = "wasm32"))]
        {
            client.connect().await?;

            // Wait for connection to be established (with timeout)
            use tokio::time::{timeout, Duration};
            let wait_result = timeout(Duration::from_secs(10), async {
                while !client.is_connected() {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            })
            .await;

            if wait_result.is_err() {
                return Err(SockudoError::connection(
                    "Connection timeout - failed to connect within 10 seconds",
                ));
            }
        }

        Ok(client)
    }

    /// Create a new Sockudo client from options without auto-connecting.
    ///
    /// This is useful for testing or when you want manual control over the connection.
    /// For normal use, prefer `SockudoClient::new()` which auto-connects like Pusher-JS.
    pub fn from_options(options: SockudoOptions) -> Result<Self> {
        if options.app_key.is_empty() {
            return Err(SockudoError::config("App key is required"));
        }

        let config: Config = options.clone().into();
        let config = Arc::new(config);

        // Create channels with callbacks
        let mut channels = Channels::new();

        // Create delta manager if enabled
        let delta_manager = if let Some(delta_opts) = config.delta_compression.clone() {
            if delta_opts.enabled {
                Some(Arc::new(RwLock::new(DeltaManager::new(delta_opts))))
            } else {
                None
            }
        } else {
            None
        };

        // Generate session ID
        let session_id = rand::random::<u32>();

        info!(
            "Creating Sockudo client for app '{}' (session: {})",
            options.app_key, session_id
        );

        // Create event dispatcher
        let global_emitter = EventDispatcher::new();

        // Create connection manager
        let connection = Arc::new(ConnectionManager::new((*config).clone()));

        // Set up send callback for channels
        let connection_clone = connection.clone();
        channels.set_send_callback(Arc::new(move |event_name, data, channel| {
            let mut event = PusherEvent::new(event_name);
            event.data = Some(data.clone());
            event.channel = channel.map(|s| s.to_string());

            match Protocol::encode_message(&event) {
                Ok(msg) => connection_clone.send(&msg),
                Err(_) => false,
            }
        }));

        // Set up send callback for delta manager
        if let Some(ref dm) = delta_manager {
            let connection_for_delta = connection.clone();
            dm.write()
                .set_send_callback(Arc::new(move |event_name, data| {
                    let mut event = PusherEvent::new(event_name);
                    #[cfg(feature = "wasm")]
                    {
                        event.data = Some(data.clone());
                    }
                    #[cfg(not(feature = "wasm"))]
                    {
                        event.data = Some(data.to_string());
                    }

                    match Protocol::encode_message(&event) {
                        Ok(msg) => connection_for_delta.send(&msg),
                        Err(_) => false,
                    }
                }));
        }

        // Set up authorization callback for private/presence channels
        // Authorization callback is only needed for native builds
        // WASM uses async authorization directly in subscribe_async
        #[cfg(not(target_arch = "wasm32"))]
        if !config.auth_endpoint.is_empty() {
            let auth_client = Arc::new(AuthClient::new(
                Some(config.auth_endpoint.clone()),
                Some(config.auth_headers.clone()),
                None,
                None,
            ));

            channels.set_authorize_callback(Arc::new(move |channel_name, socket_id| {
                tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(async {
                        auth_client.authorize_channel(channel_name, socket_id).await
                    })
                })
            }));
        }

        Ok(Self {
            key: options.app_key,
            config,
            channels: Arc::new(channels),
            connection,
            delta_manager,
            global_emitter,
            session_id,
        })
    }

    /// Get the current connection state.
    pub fn state(&self) -> ConnectionState {
        self.connection.state()
    }

    /// Get the socket ID assigned by the server.
    pub fn socket_id(&self) -> Option<String> {
        self.connection.socket_id()
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl std::fmt::Debug for SockudoClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SockudoClient")
            .field("key", &self.key)
            .field("session_id", &self.session_id)
            .field("state", &self.state())
            .field("socket_id", &self.socket_id())
            .field("channel_count", &self.channels.len())
            .finish()
    }
}

// Make SockudoClient Send + Sync for use across threads
#[cfg(not(target_arch = "wasm32"))]
unsafe impl Send for SockudoClient {}
#[cfg(not(target_arch = "wasm32"))]
unsafe impl Sync for SockudoClient {}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use crate::ChannelType;

    use super::*;

    #[test]
    fn test_client_creation() {
        let options = SockudoOptions::new("test-key").cluster("mt1");
        #[cfg(feature = "uniffi")]
        let client = SockudoClient::new(options.into()).unwrap();
        #[cfg(not(feature = "uniffi"))]
        let client = SockudoClient::from_options(options).unwrap();

        assert_eq!(client.key(), "test-key");
        assert_eq!(client.state(), ConnectionState::Initialized);
    }

    #[test]
    fn test_client_requires_key() {
        let options = SockudoOptions::default();
        #[cfg(feature = "uniffi")]
        let result = SockudoClient::new(options.into());
        #[cfg(not(feature = "uniffi"))]
        let result = SockudoClient::new(options);
        assert!(result.is_err());
    }

    #[test]
    fn test_subscribe() {
        let options = SockudoOptions::new("test-key");
        #[cfg(feature = "uniffi")]
        let client = SockudoClient::new(options.into()).unwrap();
        #[cfg(not(feature = "uniffi"))]
        let client = SockudoClient::from_options(options).unwrap();

        let channel = client.subscribe("test-channel").unwrap();
        assert_eq!(channel.name(), "test-channel");
        assert_eq!(channel.channel_type(), ChannelType::Public);
    }

    #[test]
    fn test_invalid_channel_name() {
        let options = SockudoOptions::new("test-key");
        #[cfg(feature = "uniffi")]
        let client = SockudoClient::new(options.into()).unwrap();
        #[cfg(not(feature = "uniffi"))]
        let client = SockudoClient::from_options(options).unwrap();

        let result = client.subscribe("#invalid");
        assert!(result.is_err());
    }
}

/// Pusher-compatible alias for SockudoClient (for backward compatibility)
#[cfg(not(target_arch = "wasm32"))]
pub type Pusher = SockudoClient;
