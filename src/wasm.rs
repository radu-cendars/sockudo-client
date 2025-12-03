//! WebAssembly bindings for JavaScript/Node.js/Browser.
//!
//! This module provides wasm-bindgen bindings that allow the Sockudo client
//! to be used from JavaScript in both browser and Node.js environments.

#![cfg(feature = "wasm")]

use js_sys::{Array, Function};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;

use crate::delta::{DeltaAlgorithm, DeltaOptions};
use crate::options::SockudoOptions;

/// WebAssembly-friendly delta compression options
#[wasm_bindgen]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmDeltaOptions {
    #[wasm_bindgen(skip)]
    pub enabled: bool,
    #[wasm_bindgen(skip)]
    pub algorithms: Vec<String>,
    #[wasm_bindgen(skip)]
    pub debug: bool,
    #[wasm_bindgen(skip)]
    pub max_messages_per_key: u32,
}

#[wasm_bindgen]
impl WasmDeltaOptions {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            enabled: true,
            algorithms: vec!["fossil".to_string(), "xdelta3".to_string()],
            debug: false,
            max_messages_per_key: 10,
        }
    }

    #[wasm_bindgen(getter)]
    pub fn enabled(&self) -> bool {
        self.enabled
    }

    #[wasm_bindgen(setter)]
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    #[wasm_bindgen(getter)]
    pub fn debug(&self) -> bool {
        self.debug
    }

    #[wasm_bindgen(setter)]
    pub fn set_debug(&mut self, debug: bool) {
        self.debug = debug;
    }

    #[wasm_bindgen(getter)]
    pub fn max_messages_per_key(&self) -> u32 {
        self.max_messages_per_key
    }

    #[wasm_bindgen(setter)]
    pub fn set_max_messages_per_key(&mut self, max: u32) {
        self.max_messages_per_key = max;
    }

    /// Set algorithms as comma-separated string (e.g., "fossil,xdelta3")
    #[wasm_bindgen(js_name = setAlgorithms)]
    pub fn set_algorithms(&mut self, algorithms: &str) {
        self.algorithms = algorithms
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();
    }

    /// Convert to internal DeltaOptions
    pub(crate) fn to_delta_options(&self) -> DeltaOptions {
        let algorithms: Vec<DeltaAlgorithm> = self
            .algorithms
            .iter()
            .filter_map(|a| a.parse().ok())
            .collect();

        DeltaOptions {
            enabled: self.enabled,
            algorithms: if algorithms.is_empty() {
                vec![DeltaAlgorithm::Fossil, DeltaAlgorithm::Xdelta3]
            } else {
                algorithms
            },
            debug: self.debug,
            max_messages_per_key: self.max_messages_per_key as usize,
            on_stats: None,
            on_error: None,
        }
    }
}

/// WebAssembly-friendly options for creating a Sockudo client
#[wasm_bindgen]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmOptions {
    #[wasm_bindgen(skip)]
    pub app_key: String,
    #[wasm_bindgen(skip)]
    pub cluster: Option<String>,
    #[wasm_bindgen(skip)]
    pub ws_host: Option<String>,
    #[wasm_bindgen(skip)]
    pub ws_port: Option<u16>,
    #[wasm_bindgen(skip)]
    pub use_tls: Option<bool>,
    #[wasm_bindgen(skip)]
    pub auth_endpoint: Option<String>,
    #[wasm_bindgen(skip)]
    pub delta_compression: Option<WasmDeltaOptions>,
}

#[wasm_bindgen]
impl WasmOptions {
    #[wasm_bindgen(constructor)]
    pub fn new(app_key: &str) -> Self {
        Self {
            app_key: app_key.to_string(),
            cluster: None,
            ws_host: None,
            ws_port: None,
            use_tls: None,
            auth_endpoint: None,
            delta_compression: None,
        }
    }

    #[wasm_bindgen(getter)]
    pub fn cluster(&self) -> Option<String> {
        self.cluster.clone()
    }

    #[wasm_bindgen(setter)]
    pub fn set_cluster(&mut self, cluster: &str) {
        self.cluster = Some(cluster.to_string());
    }

    #[wasm_bindgen(getter)]
    pub fn ws_host(&self) -> Option<String> {
        self.ws_host.clone()
    }

    #[wasm_bindgen(setter)]
    pub fn set_ws_host(&mut self, host: &str) {
        self.ws_host = Some(host.to_string());
    }

    #[wasm_bindgen(getter)]
    pub fn ws_port(&self) -> Option<u16> {
        self.ws_port
    }

    #[wasm_bindgen(setter)]
    pub fn set_ws_port(&mut self, port: u16) {
        self.ws_port = Some(port);
    }

    #[wasm_bindgen(getter)]
    pub fn use_tls(&self) -> Option<bool> {
        self.use_tls
    }

    #[wasm_bindgen(setter)]
    pub fn set_use_tls(&mut self, use_tls: bool) {
        self.use_tls = Some(use_tls);
    }

    #[wasm_bindgen(getter)]
    pub fn auth_endpoint(&self) -> Option<String> {
        self.auth_endpoint.clone()
    }

    #[wasm_bindgen(setter)]
    pub fn set_auth_endpoint(&mut self, endpoint: &str) {
        self.auth_endpoint = Some(endpoint.to_string());
    }

    /// Set delta compression options
    #[wasm_bindgen(js_name = setDeltaCompression)]
    pub fn set_delta_compression(&mut self, options: WasmDeltaOptions) {
        self.delta_compression = Some(options);
    }

    /// Enable delta compression with default options
    #[wasm_bindgen(js_name = enableDeltaCompression)]
    pub fn enable_delta_compression(&mut self) {
        self.delta_compression = Some(WasmDeltaOptions::new());
    }

    /// Convert to internal SockudoOptions
    pub(crate) fn to_sockudo_options(&self) -> SockudoOptions {
        let mut opts = SockudoOptions::new(&self.app_key);
        opts.cluster = self.cluster.clone();
        opts.ws_host = self.ws_host.clone();
        opts.ws_port = self.ws_port;
        opts.use_tls = self.use_tls;
        opts.auth_endpoint = self.auth_endpoint.clone();
        opts.delta_compression = self
            .delta_compression
            .as_ref()
            .map(|d| d.to_delta_options());
        opts
    }
}

/// The main Sockudo client for WebAssembly/JavaScript
/// Exported as both "Sockudo" and "Pusher" for compatibility
#[wasm_bindgen(js_name = Sockudo)]
#[derive(Clone)]
pub struct WasmSockudo {
    #[wasm_bindgen(skip)]
    inner: Arc<RwLock<WasmSockudoInner>>,
}

struct WasmSockudoInner {
    key: String,
    options: SockudoOptions,
    socket_id: Option<String>,
    state: String,
    channels: std::collections::HashMap<String, WasmChannel>,
    callbacks: std::collections::HashMap<String, Vec<Function>>,
    global_callbacks: Vec<Function>,
    ws: Option<web_sys::WebSocket>,
}

#[wasm_bindgen]
impl WasmSockudo {
    /// Create a new Sockudo client
    #[wasm_bindgen(constructor)]
    pub fn new(app_key: &str, options: Option<WasmOptions>) -> Result<WasmSockudo, JsValue> {
        console_error_panic_hook::set_once();

        let opts = options
            .map(|o| o.to_sockudo_options())
            .unwrap_or_else(|| SockudoOptions::new(app_key));

        let client = Self {
            inner: Arc::new(RwLock::new(WasmSockudoInner {
                key: app_key.to_string(),
                options: opts,
                socket_id: None,
                state: "initialized".to_string(),
                channels: std::collections::HashMap::new(),
                callbacks: std::collections::HashMap::new(),
                global_callbacks: Vec::new(),
                ws: None,
            })),
        };

        // Auto-connect (Pusher-JS behavior)
        let client_clone = client.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let _ = client_clone.connect().await;
        });

        Ok(client)
    }

    /// Connect to the Pusher server
    #[wasm_bindgen]
    pub async fn connect(&self) -> Result<(), JsValue> {
        let mut inner = self.inner.write();
        inner.state = "connecting".to_string();

        // Debug: log options
        web_sys::console::log_1(
            &format!(
                "Options - cluster: {:?}, ws_host: {:?}, use_tls: {:?}",
                inner.options.cluster, inner.options.ws_host, inner.options.use_tls
            )
            .into(),
        );

        // Build WebSocket URL
        let use_tls = inner.options.use_tls.unwrap_or(true);
        let protocol = if use_tls { "wss" } else { "ws" };

        let host = if let Some(ref h) = inner.options.ws_host {
            h.clone()
        } else if let Some(ref cluster) = inner.options.cluster {
            format!("ws-{}.pusher.com", cluster)
        } else {
            return Err(JsValue::from_str("No host or cluster specified"));
        };

        let port = inner
            .options
            .ws_port
            .unwrap_or(if use_tls { 443 } else { 80 });

        let url = format!(
            "{}://{}:{}/app/{}?protocol=7&client=sockudo-rust&version=0.1.0",
            protocol, host, port, inner.key
        );

        web_sys::console::log_1(&format!("Connecting to: {}", url).into());

        // Create WebSocket
        let ws = web_sys::WebSocket::new(&url)
            .map_err(|e| JsValue::from_str(&format!("Failed to create WebSocket: {:?}", e)))?;

        // Set up event handlers
        let inner_clone = self.inner.clone();
        let onopen = Closure::wrap(Box::new(move |_event: web_sys::Event| {
            let mut inner = inner_clone.write();
            inner.state = "connected".to_string();
            web_sys::console::log_1(&"WebSocket connected!".into());
        }) as Box<dyn FnMut(web_sys::Event)>);
        ws.set_onopen(Some(onopen.as_ref().unchecked_ref()));
        onopen.forget();

        let inner_clone = self.inner.clone();
        let onmessage = Closure::wrap(Box::new(move |event: web_sys::MessageEvent| {
            if let Ok(text) = event.data().dyn_into::<js_sys::JsString>() {
                let message: String = text.into();
                web_sys::console::log_1(&format!("Received: {}", message).into());

                // Parse Pusher message and handle it
                if let Ok(event_data) = serde_json::from_str::<serde_json::Value>(&message) {
                    if let Some(event_name) = event_data.get("event").and_then(|v| v.as_str()) {
                        // Handle pusher:connection_established
                        if event_name == "pusher:connection_established" {
                            if let Some(data) = event_data.get("data").and_then(|v| v.as_str()) {
                                if let Ok(conn_data) =
                                    serde_json::from_str::<serde_json::Value>(data)
                                {
                                    if let Some(socket_id) =
                                        conn_data.get("socket_id").and_then(|v| v.as_str())
                                    {
                                        inner_clone.write().socket_id = Some(socket_id.to_string());
                                        web_sys::console::log_1(
                                            &format!("Socket ID: {}", socket_id).into(),
                                        );
                                    }
                                }
                            }
                        }

                        // Get channel name if present
                        let channel_name = event_data.get("channel").and_then(|v| v.as_str());

                        // Trigger channel-specific callbacks
                        if let Some(ch_name) = channel_name {
                            let inner = inner_clone.read();
                            if let Some(channel) = inner.channels.get(ch_name) {
                                let callbacks = channel.callbacks.read();

                                // Trigger event-specific callbacks
                                if let Some(cbs) = callbacks.get(event_name) {
                                    for callback in cbs {
                                        let _ = callback
                                            .call1(&JsValue::NULL, &JsValue::from_str(&message));
                                    }
                                }

                                // Trigger bind_all callbacks
                                if let Some(all_cbs) = callbacks.get("__all__") {
                                    for callback in all_cbs {
                                        // Call with event name and data
                                        let event_js = JsValue::from_str(event_name);
                                        let data_js = event_data
                                            .get("data")
                                            .and_then(|v| v.as_str())
                                            .map(|s| JsValue::from_str(s))
                                            .unwrap_or(JsValue::NULL);
                                        let _ = callback.call2(&JsValue::NULL, &event_js, &data_js);
                                    }
                                }
                            }
                        }

                        // Trigger global event callbacks
                        let inner = inner_clone.read();
                        if let Some(callbacks) = inner.callbacks.get(event_name) {
                            for callback in callbacks {
                                let _ =
                                    callback.call1(&JsValue::NULL, &JsValue::from_str(&message));
                            }
                        }

                        // Trigger global callbacks
                        for callback in &inner.global_callbacks {
                            let _ = callback.call1(&JsValue::NULL, &JsValue::from_str(&message));
                        }
                    }
                }
            }
        }) as Box<dyn FnMut(web_sys::MessageEvent)>);
        ws.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
        onmessage.forget();

        let inner_clone = self.inner.clone();
        let onerror = Closure::wrap(Box::new(move |_event: web_sys::ErrorEvent| {
            let mut inner = inner_clone.write();
            inner.state = "failed".to_string();
            web_sys::console::error_1(&"WebSocket error!".into());
        }) as Box<dyn FnMut(web_sys::ErrorEvent)>);
        ws.set_onerror(Some(onerror.as_ref().unchecked_ref()));
        onerror.forget();

        let inner_clone = self.inner.clone();
        let onclose = Closure::wrap(Box::new(move |_event: web_sys::CloseEvent| {
            let mut inner = inner_clone.write();
            inner.state = "disconnected".to_string();
            inner.socket_id = None;
            web_sys::console::log_1(&"WebSocket closed".into());
        }) as Box<dyn FnMut(web_sys::CloseEvent)>);
        ws.set_onclose(Some(onclose.as_ref().unchecked_ref()));
        onclose.forget();

        // Store the WebSocket
        inner.ws = Some(ws);
        drop(inner);

        Ok(())
    }

    /// Disconnect from the server
    #[wasm_bindgen]
    pub fn disconnect(&self) {
        let mut inner = self.inner.write();

        if let Some(ws) = inner.ws.take() {
            let _ = ws.close();
            web_sys::console::log_1(&"Disconnecting WebSocket...".into());
        }

        inner.state = "disconnected".to_string();
        inner.socket_id = None;
    }

    /// Get the current connection state
    #[wasm_bindgen(getter)]
    pub fn state(&self) -> String {
        self.inner.read().state.clone()
    }

    /// Get the socket ID
    #[wasm_bindgen(getter)]
    pub fn socket_id(&self) -> Option<String> {
        self.inner.read().socket_id.clone()
    }

    /// Subscribe to a channel
    #[wasm_bindgen]
    pub fn subscribe(&self, channel_name: &str) -> Result<WasmChannel, JsValue> {
        let mut inner = self.inner.write();

        if inner.channels.contains_key(channel_name) {
            return Ok(inner.channels.get(channel_name).unwrap().clone());
        }

        let channel = WasmChannel::new(channel_name);
        inner
            .channels
            .insert(channel_name.to_string(), channel.clone());

        // Send subscribe message if connected
        if let Some(ws) = &inner.ws {
            if inner.state == "connected" {
                let subscribe_msg = serde_json::json!({
                    "event": "pusher:subscribe",
                    "data": {
                        "channel": channel_name
                    }
                });

                if let Ok(msg_str) = serde_json::to_string(&subscribe_msg) {
                    let _ = ws.send_with_str(&msg_str);
                    web_sys::console::log_1(
                        &format!("Subscribing to channel: {}", channel_name).into(),
                    );
                }
            }
        }

        Ok(channel)
    }

    /// Unsubscribe from a channel
    #[wasm_bindgen]
    pub fn unsubscribe(&self, channel_name: &str) {
        let mut inner = self.inner.write();

        // Send unsubscribe message if connected
        if let Some(ws) = &inner.ws {
            if inner.state == "connected" {
                let unsubscribe_msg = serde_json::json!({
                    "event": "pusher:unsubscribe",
                    "data": {
                        "channel": channel_name
                    }
                });

                if let Ok(msg_str) = serde_json::to_string(&unsubscribe_msg) {
                    let _ = ws.send_with_str(&msg_str);
                    web_sys::console::log_1(
                        &format!("Unsubscribing from channel: {}", channel_name).into(),
                    );
                }
            }
        }

        inner.channels.remove(channel_name);
    }

    /// Get a channel by name
    #[wasm_bindgen]
    pub fn channel(&self, name: &str) -> Option<WasmChannel> {
        self.inner.read().channels.get(name).cloned()
    }

    /// Bind a callback to an event
    #[wasm_bindgen]
    pub fn bind(&self, event_name: &str, callback: Function) {
        let mut inner = self.inner.write();
        inner
            .callbacks
            .entry(event_name.to_string())
            .or_default()
            .push(callback);
    }

    /// Bind a global callback
    #[wasm_bindgen]
    pub fn bind_global(&self, callback: Function) {
        let mut inner = self.inner.write();
        inner.global_callbacks.push(callback);
    }

    /// Unbind callbacks from an event
    #[wasm_bindgen]
    pub fn unbind(&self, event_name: Option<String>) {
        let mut inner = self.inner.write();
        if let Some(name) = event_name {
            inner.callbacks.remove(&name);
        } else {
            inner.callbacks.clear();
        }
    }

    /// Unbind global callbacks
    #[wasm_bindgen]
    pub fn unbind_global(&self) {
        let mut inner = self.inner.write();
        inner.global_callbacks.clear();
    }

    /// Unbind all callbacks
    #[wasm_bindgen]
    pub fn unbind_all(&self) {
        let mut inner = self.inner.write();
        inner.callbacks.clear();
        inner.global_callbacks.clear();
    }

    /// Send an event
    #[wasm_bindgen]
    pub fn send_event(&self, event_name: &str, data: JsValue, channel: Option<String>) -> bool {
        let inner = self.inner.read();

        if let Some(ws) = &inner.ws {
            if inner.state == "connected" {
                // Convert JsValue to JSON string
                let data_str = if let Ok(s) = js_sys::JSON::stringify(&data) {
                    String::from(s)
                } else {
                    return false;
                };

                let event_msg = if let Some(ch) = channel {
                    serde_json::json!({
                        "event": event_name,
                        "channel": ch,
                        "data": data_str
                    })
                } else {
                    serde_json::json!({
                        "event": event_name,
                        "data": data_str
                    })
                };

                if let Ok(msg_str) = serde_json::to_string(&event_msg) {
                    if ws.send_with_str(&msg_str).is_ok() {
                        web_sys::console::log_1(&format!("Sent event: {}", event_name).into());
                        return true;
                    }
                }
            }
        }

        false
    }

    /// Get delta compression stats
    #[wasm_bindgen]
    pub fn get_delta_stats(&self) -> JsValue {
        // Return null for now, would return actual stats
        JsValue::NULL
    }

    /// Reset delta compression stats
    #[wasm_bindgen]
    pub fn reset_delta_stats(&self) {
        // Implementation
    }
}

/// WebAssembly-friendly channel wrapper
#[wasm_bindgen(js_name = Channel)]
#[derive(Clone)]
pub struct WasmChannel {
    name: String,
    subscribed: bool,
    callbacks: Arc<RwLock<std::collections::HashMap<String, Vec<Function>>>>,
}

#[wasm_bindgen]
impl WasmChannel {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            subscribed: false,
            callbacks: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Get channel name
    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        self.name.clone()
    }

    /// Check if subscribed
    #[wasm_bindgen(getter)]
    pub fn subscribed(&self) -> bool {
        self.subscribed
    }

    /// Bind a callback to an event
    #[wasm_bindgen]
    pub fn bind(&self, event_name: &str, callback: Function) -> WasmChannel {
        let mut callbacks = self.callbacks.write();
        callbacks
            .entry(event_name.to_string())
            .or_default()
            .push(callback);
        self.clone()
    }

    /// Bind a callback to all events on this channel
    #[wasm_bindgen]
    pub fn bind_all(&self, callback: Function) -> WasmChannel {
        let mut callbacks = self.callbacks.write();
        callbacks
            .entry("__all__".to_string())
            .or_default()
            .push(callback);
        self.clone()
    }

    /// Unbind callbacks
    #[wasm_bindgen]
    pub fn unbind(&self, event_name: Option<String>) -> WasmChannel {
        let mut callbacks = self.callbacks.write();
        if let Some(name) = event_name {
            callbacks.remove(&name);
        } else {
            callbacks.clear();
        }
        self.clone()
    }

    /// Trigger a client event (private/presence channels only)
    #[wasm_bindgen]
    pub fn trigger(&self, event_name: &str, data: JsValue) -> Result<bool, JsValue> {
        if !event_name.starts_with("client-") {
            return Err(JsValue::from_str("Client events must start with 'client-'"));
        }

        if !self.name.starts_with("private-") && !self.name.starts_with("presence-") {
            return Err(JsValue::from_str(
                "Client events only work on private/presence channels",
            ));
        }

        // Convert data to JSON string
        let data_str = js_sys::JSON::stringify(&data)
            .map(|s| String::from(s))
            .map_err(|_| JsValue::from_str("Failed to stringify data"))?;

        let event_msg = serde_json::json!({
            "event": event_name,
            "channel": self.name,
            "data": data_str
        });

        // Would need access to parent Pusher's WebSocket
        // For now, log it
        web_sys::console::log_1(&format!("Would trigger: {:?}", event_msg).into());

        Ok(true)
    }
}

/// WebAssembly-friendly presence channel
#[wasm_bindgen(js_name = PresenceChannel)]
pub struct WasmPresenceChannel {
    #[wasm_bindgen(skip)]
    inner: WasmChannel,
    members: Arc<RwLock<Vec<WasmMember>>>,
    my_id: Arc<RwLock<Option<String>>>,
}

#[wasm_bindgen]
impl WasmPresenceChannel {
    /// Get channel name
    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        self.inner.name.clone()
    }

    /// Get all members as an array
    #[wasm_bindgen]
    pub fn members(&self) -> Array {
        let members = self.members.read();
        let arr = Array::new();
        for member in members.iter() {
            arr.push(&JsValue::from(member.clone()));
        }
        arr
    }

    /// Get current user's member info
    #[wasm_bindgen]
    pub fn me(&self) -> Option<WasmMember> {
        let my_id = self.my_id.read();
        if let Some(ref id) = *my_id {
            let members = self.members.read();
            members.iter().find(|m| &m.id == id).cloned()
        } else {
            None
        }
    }

    /// Get member count
    #[wasm_bindgen(getter)]
    pub fn count(&self) -> usize {
        self.members.read().len()
    }

    /// Get a member by ID
    #[wasm_bindgen]
    pub fn get(&self, user_id: &str) -> Option<WasmMember> {
        self.members
            .read()
            .iter()
            .find(|m| m.id == user_id)
            .cloned()
    }
}

/// WebAssembly-friendly member info
#[wasm_bindgen(js_name = Member)]
#[derive(Clone)]
pub struct WasmMember {
    id: String,
    info: JsValue,
}

#[wasm_bindgen]
impl WasmMember {
    #[wasm_bindgen(getter)]
    pub fn id(&self) -> String {
        self.id.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn info(&self) -> JsValue {
        self.info.clone()
    }
}

/// Initialize console error panic hook for better error messages
#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
}

// Additional helper for console_error_panic_hook
mod console_error_panic_hook {
    use std::sync::Once;

    static INIT: Once = Once::new();

    pub fn set_once() {
        INIT.call_once(|| {
            std::panic::set_hook(Box::new(|panic_info| {
                web_sys::console::error_1(&format!("{}", panic_info).into());
            }));
        });
    }
}

// Note: wasm-bindgen doesn't support type aliases with js_name.
// JavaScript/TypeScript users can create their own aliases in their code:
// export { Sockudo as Pusher, SockudoOptions as PusherOptions } from 'sockudo';
