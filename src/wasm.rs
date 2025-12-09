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

use crate::delta::{decoders, DeltaAlgorithm, DeltaOptions, DeltaStats};
use crate::options::SockudoOptions;
use crate::protocol::filter::FilterOp as InternalFilterOp;

/// Response from authorization endpoint
#[derive(Debug, Deserialize)]
struct AuthResponse {
    auth: String,
    #[serde(default)]
    channel_data: Option<String>,
    #[serde(default)]
    shared_secret: Option<String>,
}

/// Auth data for channel subscription
#[derive(Debug)]
struct AuthData {
    auth: String,
    channel_data: Option<String>,
}

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

/// WebAssembly-friendly filter operations for tag filtering
#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct WasmFilterOp {
    inner: InternalFilterOp,
}

#[wasm_bindgen]
impl WasmFilterOp {
    /// Create an equality filter: field == value
    #[wasm_bindgen(js_name = eq)]
    pub fn eq(field: &str, value: &str) -> WasmFilterOp {
        WasmFilterOp {
            inner: InternalFilterOp::eq(field, value),
        }
    }

    /// Create a not-equal filter: field != value
    #[wasm_bindgen(js_name = neq)]
    pub fn neq(field: &str, value: &str) -> WasmFilterOp {
        WasmFilterOp {
            inner: InternalFilterOp::neq(field, value),
        }
    }

    /// Create a less-than filter: field < value
    #[wasm_bindgen(js_name = lt)]
    pub fn lt(field: &str, value: &str) -> WasmFilterOp {
        WasmFilterOp {
            inner: InternalFilterOp::lt(field, value),
        }
    }

    /// Create a less-than-or-equal filter: field <= value
    #[wasm_bindgen(js_name = lte)]
    pub fn lte(field: &str, value: &str) -> WasmFilterOp {
        WasmFilterOp {
            inner: InternalFilterOp::lte(field, value),
        }
    }

    /// Create a greater-than filter: field > value
    #[wasm_bindgen(js_name = gt)]
    pub fn gt(field: &str, value: &str) -> WasmFilterOp {
        WasmFilterOp {
            inner: InternalFilterOp::gt(field, value),
        }
    }

    /// Create a greater-than-or-equal filter: field >= value
    #[wasm_bindgen(js_name = gte)]
    pub fn gte(field: &str, value: &str) -> WasmFilterOp {
        WasmFilterOp {
            inner: InternalFilterOp::gte(field, value),
        }
    }

    /// Create an IN filter: field in [values]
    #[wasm_bindgen(js_name = inSet)]
    pub fn in_set(field: &str, values: Vec<String>) -> WasmFilterOp {
        WasmFilterOp {
            inner: InternalFilterOp::in_set(field, values),
        }
    }

    /// Create a NOT IN filter: field not in [values]
    #[wasm_bindgen(js_name = notIn)]
    pub fn not_in(field: &str, values: Vec<String>) -> WasmFilterOp {
        WasmFilterOp {
            inner: InternalFilterOp::not_in(field, values),
        }
    }

    /// Create an EXISTS filter
    #[wasm_bindgen(js_name = exists)]
    pub fn exists(field: &str) -> WasmFilterOp {
        WasmFilterOp {
            inner: InternalFilterOp::exists(field),
        }
    }

    /// Create a NOT EXISTS filter
    #[wasm_bindgen(js_name = notExists)]
    pub fn not_exists(field: &str) -> WasmFilterOp {
        WasmFilterOp {
            inner: InternalFilterOp::not_exists(field),
        }
    }

    /// Create an AND filter combining multiple filters
    #[wasm_bindgen(js_name = and)]
    pub fn and(filters: Vec<WasmFilterOp>) -> WasmFilterOp {
        let inner_filters: Vec<InternalFilterOp> = filters.into_iter().map(|f| f.inner).collect();
        WasmFilterOp {
            inner: InternalFilterOp::and(inner_filters),
        }
    }

    /// Create an OR filter combining multiple filters
    #[wasm_bindgen(js_name = or)]
    pub fn or(filters: Vec<WasmFilterOp>) -> WasmFilterOp {
        let inner_filters: Vec<InternalFilterOp> = filters.into_iter().map(|f| f.inner).collect();
        WasmFilterOp {
            inner: InternalFilterOp::or(inner_filters),
        }
    }

    /// Convert to JSON string for debugging
    #[wasm_bindgen(js_name = toJSON)]
    pub fn to_json(&self) -> String {
        serde_json::to_string(&self.inner).unwrap_or_else(|_| "{}".to_string())
    }

    /// Get the internal filter (for internal use)
    pub(crate) fn into_inner(self) -> InternalFilterOp {
        self.inner
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
#[wasm_bindgen]
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
    delta_stats: DeltaStats,
    delta_compression_enabled: bool,
    /// Store base messages for delta decoding: channel -> base message string
    delta_base_messages: std::collections::HashMap<String, String>,
}

#[wasm_bindgen]
impl WasmSockudo {
    /// Create a new Sockudo client
    #[wasm_bindgen(constructor)]
    pub fn new(app_key: &str, options: Option<WasmOptions>) -> Result<WasmSockudo, JsValue> {
        console_error_panic_hook::set_once();

        web_sys::console::log_1(&format!("Rust received options: {:?}", options).into());

        let opts = options
            .map(|o| {
                web_sys::console::log_1(&format!("Converting options: {:?}", o).into());
                o.to_sockudo_options()
            })
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
                delta_stats: DeltaStats::new(),
                delta_compression_enabled: false,
                delta_base_messages: std::collections::HashMap::new(),
            })),
        };

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
                let message_size = message.len();
                web_sys::console::log_1(&format!("Received: {}", message).into());

                // Parse Pusher message and handle it
                if let Ok(event_data) = serde_json::from_str::<serde_json::Value>(&message) {
                    if let Some(event_name) = event_data.get("event").and_then(|v| v.as_str()) {
                        // Track delta stats for non-internal messages
                        if !event_name.starts_with("pusher:")
                            && !event_name.starts_with("pusher_internal:")
                        {
                            let mut inner = inner_clone.write();
                            inner.delta_stats.total_messages += 1;

                            // Check if this is a delta message (has delta field in data)
                            let is_delta = event_data
                                .get("data")
                                .and_then(|d| d.as_str())
                                .and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok())
                                .map(|parsed| parsed.get("delta").is_some())
                                .unwrap_or(false);

                            if is_delta {
                                inner.delta_stats.delta_messages += 1;
                                // For delta messages, compressed size is the message size
                                // decompressed would be larger (estimate 3x for now)
                                inner.delta_stats.total_bytes_with_compression +=
                                    message_size as u64;
                                inner.delta_stats.total_bytes_without_compression +=
                                    (message_size * 3) as u64;
                            } else {
                                inner.delta_stats.full_messages += 1;
                                inner.delta_stats.total_bytes_with_compression +=
                                    message_size as u64;
                                inner.delta_stats.total_bytes_without_compression +=
                                    message_size as u64;
                            }

                            inner.delta_stats.calculate_savings();
                            drop(inner);
                        }
                        // Handle pusher:ping - respond with pusher:pong immediately
                        if event_name == "pusher:ping" {
                            web_sys::console::log_1(
                                &"Received pusher:ping, sending pusher:pong".into(),
                            );
                            let inner = inner_clone.read();
                            if let Some(ref ws) = inner.ws {
                                let pong = serde_json::json!({
                                    "event": "pusher:pong",
                                    "data": {}
                                });
                                if let Ok(pong_str) = serde_json::to_string(&pong) {
                                    let _ = ws.send_with_str(&pong_str);
                                }
                            }
                        }

                        // Handle pusher:delta_compression_enabled
                        if event_name == "pusher:delta_compression_enabled" {
                            let mut inner = inner_clone.write();
                            inner.delta_compression_enabled = true;
                            web_sys::console::log_1(&"Delta compression enabled!".into());
                        }

                        // Handle pusher:delta - decode and re-emit as original event
                        if event_name == "pusher:delta" {
                            if let Some(channel) =
                                event_data.get("channel").and_then(|v| v.as_str())
                            {
                                if let Some(data) = event_data.get("data") {
                                    // Decode the delta message
                                    match Self::decode_delta_message(
                                        &inner_clone,
                                        channel,
                                        data.clone(),
                                    ) {
                                        Ok(reconstructed_message) => {
                                            // Parse the reconstructed message and re-emit it
                                            if let Ok(reconstructed_event) =
                                                serde_json::from_str::<serde_json::Value>(
                                                    &reconstructed_message,
                                                )
                                            {
                                                // Store the reconstructed message as new base
                                                inner_clone.write().delta_base_messages.insert(
                                                    channel.to_string(),
                                                    reconstructed_message.clone(),
                                                );

                                                // Extract the original event name and data
                                                if let Some(orig_event) = reconstructed_event
                                                    .get("event")
                                                    .and_then(|v| v.as_str())
                                                {
                                                    // Trigger channel callbacks with the decoded event
                                                    let inner = inner_clone.read();
                                                    if let Some(ch) = inner.channels.get(channel) {
                                                        let callbacks = ch.callbacks.read();

                                                        // Build reconstructed message JSON
                                                        let reconstructed_msg = serde_json::json!({
                                                            "event": orig_event,
                                                            "channel": channel,
                                                            "data": reconstructed_event.get("data")
                                                        })
                                                        .to_string();

                                                        // Trigger event-specific callbacks
                                                        if let Some(cbs) = callbacks.get(orig_event)
                                                        {
                                                            for callback in cbs {
                                                                let _ = callback.call1(
                                                                    &JsValue::NULL,
                                                                    &JsValue::from_str(
                                                                        &reconstructed_msg,
                                                                    ),
                                                                );
                                                            }
                                                        }

                                                        // Trigger bind_all callbacks
                                                        if let Some(all_cbs) =
                                                            callbacks.get("__all__")
                                                        {
                                                            for callback in all_cbs {
                                                                let event_js =
                                                                    JsValue::from_str(orig_event);
                                                                let data_js = reconstructed_event
                                                                    .get("data")
                                                                    .and_then(|v| {
                                                                        serde_json::to_string(v)
                                                                            .ok()
                                                                    })
                                                                    .map(|s| JsValue::from_str(&s))
                                                                    .unwrap_or(JsValue::NULL);
                                                                let _ = callback.call2(
                                                                    &JsValue::NULL,
                                                                    &event_js,
                                                                    &data_js,
                                                                );
                                                            }
                                                        }
                                                    }

                                                    // Trigger global callbacks with decoded event
                                                    let inner = inner_clone.read();
                                                    if let Some(callbacks) =
                                                        inner.callbacks.get(orig_event)
                                                    {
                                                        let reconstructed_msg = serde_json::json!({
                                                            "event": orig_event,
                                                            "channel": channel,
                                                            "data": reconstructed_event.get("data")
                                                        })
                                                        .to_string();

                                                        for callback in callbacks {
                                                            let _ = callback.call1(
                                                                &JsValue::NULL,
                                                                &JsValue::from_str(
                                                                    &reconstructed_msg,
                                                                ),
                                                            );
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            web_sys::console::error_1(
                                                &format!("Delta decode failed: {}", e).into(),
                                            );
                                        }
                                    }
                                }
                            }
                        }

                        // Store base messages for delta compression (non-pusher events with sequence)
                        if !event_name.starts_with("pusher:")
                            && !event_name.starts_with("pusher_internal:")
                        {
                            if let Some(channel) =
                                event_data.get("channel").and_then(|v| v.as_str())
                            {
                                // Create sanitized base message (without sequence field)
                                let sanitized = serde_json::json!({
                                    "event": event_data.get("event"),
                                    "channel": event_data.get("channel"),
                                    "data": event_data.get("data"),
                                });
                                if let Ok(base_msg) = serde_json::to_string(&sanitized) {
                                    inner_clone
                                        .write()
                                        .delta_base_messages
                                        .insert(channel.to_string(), base_msg);
                                }
                            }
                        }

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

                        // Don't propagate pusher:delta events through normal channels
                        // (they've already been decoded and re-emitted above)
                        if event_name != "pusher:delta" {
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
                                            let _ = callback.call1(
                                                &JsValue::NULL,
                                                &JsValue::from_str(&message),
                                            );
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
                                            let _ =
                                                callback.call2(&JsValue::NULL, &event_js, &data_js);
                                        }
                                    }
                                }
                            }

                            // Trigger global event callbacks
                            let inner = inner_clone.read();
                            if let Some(callbacks) = inner.callbacks.get(event_name) {
                                for callback in callbacks {
                                    let _ = callback
                                        .call1(&JsValue::NULL, &JsValue::from_str(&message));
                                }
                            }

                            // Trigger global callbacks
                            for callback in &inner.global_callbacks {
                                let _ =
                                    callback.call1(&JsValue::NULL, &JsValue::from_str(&message));
                            }
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
    pub fn subscribe(
        &self,
        channel_name: &str,
        filter: Option<WasmFilterOp>,
    ) -> Result<WasmChannel, JsValue> {
        let mut inner = self.inner.write();

        if inner.channels.contains_key(channel_name) {
            return Ok(inner.channels.get(channel_name).unwrap().clone());
        }

        let channel = WasmChannel::new(channel_name);
        inner
            .channels
            .insert(channel_name.to_string(), channel.clone());

        // Check if this is a private or presence channel that requires authentication
        let requires_auth = channel_name.starts_with("private-")
            || channel_name.starts_with("presence-")
            || channel_name.starts_with("private-encrypted-");

        // Send subscribe message if connected
        if let Some(ws) = &inner.ws {
            if inner.state == "connected" {
                // For channels requiring authentication, we need to call the auth endpoint first
                if requires_auth {
                    let socket_id = inner.socket_id.clone();
                    let auth_endpoint = inner.options.auth_endpoint.clone();
                    let ws_clone = ws.clone();
                    let channel_name_owned = channel_name.to_string();
                    let filter_inner = filter.map(|f| f.inner);

                    // Drop the lock before spawning async task
                    drop(inner);

                    // Spawn async task to authenticate and subscribe
                    wasm_bindgen_futures::spawn_local(async move {
                        if let Some(socket_id) = socket_id {
                            if let Some(auth_endpoint) = auth_endpoint {
                                // Call auth endpoint
                                match Self::authenticate_channel(
                                    &auth_endpoint,
                                    &channel_name_owned,
                                    &socket_id,
                                )
                                .await
                                {
                                    Ok(auth_data) => {
                                        // Build subscribe data with auth
                                        let mut subscribe_data = serde_json::json!({
                                            "channel": channel_name_owned,
                                            "auth": auth_data.auth
                                        });

                                        // Add channel_data if present (for presence channels)
                                        if let Some(channel_data) = auth_data.channel_data {
                                            subscribe_data["channel_data"] =
                                                serde_json::json!(channel_data);
                                        }

                                        // Add filter if provided
                                        if let Some(f) = filter_inner {
                                            if let Ok(filter_json) = serde_json::to_value(&f) {
                                                subscribe_data["filter"] = filter_json;
                                            }
                                        }

                                        let subscribe_msg = serde_json::json!({
                                            "event": "pusher:subscribe",
                                            "data": subscribe_data
                                        });

                                        if let Ok(msg_str) = serde_json::to_string(&subscribe_msg) {
                                            let _ = ws_clone.send_with_str(&msg_str);
                                            web_sys::console::log_1(
                                                &format!(
                                                    "Subscribing to authenticated channel: {} with auth: {}",
                                                    channel_name_owned,
                                                    auth_data.auth
                                                )
                                                .into(),
                                            );
                                        }
                                    }
                                    Err(e) => {
                                        web_sys::console::error_1(
                                            &format!("Failed to authenticate channel: {:?}", e)
                                                .into(),
                                        );
                                    }
                                }
                            } else {
                                web_sys::console::error_1(
                                    &"No auth_endpoint configured for private/presence channel"
                                        .into(),
                                );
                            }
                        } else {
                            web_sys::console::error_1(
                                &"No socket_id available for authentication".into(),
                            );
                        }
                    });
                } else {
                    // Public channel - subscribe immediately
                    let mut subscribe_data = serde_json::json!({
                        "channel": channel_name
                    });

                    // Add filter if provided
                    if let Some(f) = filter {
                        if let Ok(filter_json) = serde_json::to_value(&f.inner) {
                            subscribe_data["filter"] = filter_json;
                        }
                    }

                    let subscribe_msg = serde_json::json!({
                        "event": "pusher:subscribe",
                        "data": subscribe_data
                    });

                    if let Ok(msg_str) = serde_json::to_string(&subscribe_msg) {
                        let _ = ws.send_with_str(&msg_str);
                        web_sys::console::log_1(
                            &format!(
                                "Subscribing to public channel: {} with filter: {:?}",
                                channel_name,
                                subscribe_data.get("filter")
                            )
                            .into(),
                        );
                    }
                }
            }
        }

        Ok(channel)
    }

    /// Helper method to authenticate a channel via the auth endpoint
    async fn authenticate_channel(
        auth_endpoint: &str,
        channel_name: &str,
        socket_id: &str,
    ) -> Result<AuthData, JsValue> {
        // Build form-encoded body manually
        let body = format!(
            "socket_id={}&channel_name={}",
            urlencoding::encode(socket_id),
            urlencoding::encode(channel_name)
        );

        web_sys::console::log_1(
            &format!("Auth request to: {} with body: {}", auth_endpoint, body).into(),
        );

        // Make HTTP POST request with form-urlencoded content type
        let request = gloo_net::http::Request::post(auth_endpoint)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .map_err(|e| JsValue::from_str(&format!("Failed to build request: {}", e)))?;

        // Send request
        let response = request
            .send()
            .await
            .map_err(|e| JsValue::from_str(&format!("Failed to send request: {}", e)))?;

        if !response.ok() {
            return Err(JsValue::from_str(&format!(
                "Authorization failed with status: {}",
                response.status()
            )));
        }

        // Parse response
        let auth_response: AuthResponse = response
            .json()
            .await
            .map_err(|e| JsValue::from_str(&format!("Failed to parse response: {}", e)))?;

        Ok(AuthData {
            auth: auth_response.auth,
            channel_data: auth_response.channel_data,
        })
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

    /// Unbind callbacks from a specific event
    #[wasm_bindgen]
    pub fn unbind(&self, event_name: Option<String>) {
        let mut inner = self.inner.write();
        if let Some(name) = event_name {
            inner.callbacks.remove(&name);
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
        let inner = self.inner.read();
        let stats = &inner.delta_stats;

        // Check if delta compression is enabled (runtime flag from server)
        let enabled = inner.delta_compression_enabled;

        // Create a JS object with the stats
        let obj = js_sys::Object::new();

        js_sys::Reflect::set(&obj, &"enabled".into(), &JsValue::from_bool(enabled)).ok();
        js_sys::Reflect::set(
            &obj,
            &"totalMessages".into(),
            &JsValue::from_f64(stats.total_messages as f64),
        )
        .ok();
        js_sys::Reflect::set(
            &obj,
            &"deltaMessages".into(),
            &JsValue::from_f64(stats.delta_messages as f64),
        )
        .ok();
        js_sys::Reflect::set(
            &obj,
            &"fullMessages".into(),
            &JsValue::from_f64(stats.full_messages as f64),
        )
        .ok();
        js_sys::Reflect::set(
            &obj,
            &"totalBytesWithoutCompression".into(),
            &JsValue::from_f64(stats.total_bytes_without_compression as f64),
        )
        .ok();
        js_sys::Reflect::set(
            &obj,
            &"totalBytesWithCompression".into(),
            &JsValue::from_f64(stats.total_bytes_with_compression as f64),
        )
        .ok();
        js_sys::Reflect::set(
            &obj,
            &"bandwidthSaved".into(),
            &JsValue::from_f64(stats.bandwidth_saved as f64),
        )
        .ok();
        js_sys::Reflect::set(
            &obj,
            &"bandwidthSavedPercent".into(),
            &JsValue::from_f64(stats.bandwidth_saved_percent),
        )
        .ok();
        js_sys::Reflect::set(
            &obj,
            &"errors".into(),
            &JsValue::from_f64(stats.errors as f64),
        )
        .ok();
        js_sys::Reflect::set(
            &obj,
            &"channelCount".into(),
            &JsValue::from_f64(stats.channel_count as f64),
        )
        .ok();

        obj.into()
    }

    /// Reset delta compression stats
    #[wasm_bindgen]
    pub fn reset_delta_stats(&self) {
        let mut inner = self.inner.write();
        inner.delta_stats.reset();
    }

    /// Update delta stats when a message is received (internal helper)
    fn update_delta_stats(&self, is_delta: bool, compressed_size: usize, decompressed_size: usize) {
        let mut inner = self.inner.write();
        inner.delta_stats.total_messages += 1;

        if is_delta {
            inner.delta_stats.delta_messages += 1;
            inner.delta_stats.total_bytes_with_compression += compressed_size as u64;
            inner.delta_stats.total_bytes_without_compression += decompressed_size as u64;
        } else {
            inner.delta_stats.full_messages += 1;
            let size = compressed_size as u64;
            inner.delta_stats.total_bytes_with_compression += size;
            inner.delta_stats.total_bytes_without_compression += size;
        }

        inner.delta_stats.calculate_savings();
    }

    /// Decode a delta message
    fn decode_delta_message(
        inner: &Arc<RwLock<WasmSockudoInner>>,
        channel: &str,
        delta_data: serde_json::Value,
    ) -> Result<String, String> {
        // Extract delta fields
        let algorithm = delta_data
            .get("algorithm")
            .and_then(|v| v.as_str())
            .unwrap_or("fossil");
        let delta_base64 = delta_data
            .get("delta")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing delta field".to_string())?;

        // Get the decoder
        let decoder = decoders::get_decoder(algorithm)
            .ok_or_else(|| format!("Unknown algorithm: {}", algorithm))?;

        // Get base message for this channel
        let inner_lock = inner.read();
        let base_message = inner_lock
            .delta_base_messages
            .get(channel)
            .ok_or_else(|| format!("No base message for channel: {}", channel))?
            .clone();
        drop(inner_lock);

        web_sys::console::log_1(
            &format!(
                "[WASM Delta] Decoding with {}, base length: {}, delta: {}",
                algorithm,
                base_message.len(),
                delta_base64
            )
            .into(),
        );

        // Decode base64 delta
        let delta_bytes = decoders::decode_base64(delta_base64)
            .map_err(|e| format!("Base64 decode failed: {}", e))?;

        // Apply delta
        let base_bytes = base_message.as_bytes();
        let reconstructed_bytes = decoder
            .decode(base_bytes, &delta_bytes)
            .map_err(|e| format!("Delta decode failed: {}", e))?;

        // Convert to string
        let reconstructed = String::from_utf8(reconstructed_bytes)
            .map_err(|e| format!("UTF-8 decode failed: {}", e))?;

        web_sys::console::log_1(
            &format!(
                "[WASM Delta] Decoded successfully, result length: {}",
                reconstructed.len()
            )
            .into(),
        );

        Ok(reconstructed)
    }
}

/// WebAssembly-friendly channel wrapper
#[wasm_bindgen]
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

    /// Bind a callback to all events on this channel (global)
    #[wasm_bindgen(js_name = bind_global)]
    pub fn bind_global(&self, callback: Function) -> WasmChannel {
        let mut callbacks = self.callbacks.write();
        callbacks
            .entry("__all__".to_string())
            .or_default()
            .push(callback);
        self.clone()
    }

    /// Unbind callbacks from a specific event
    #[wasm_bindgen]
    pub fn unbind(&self, event_name: Option<String>) -> WasmChannel {
        let mut callbacks = self.callbacks.write();
        if let Some(name) = event_name {
            callbacks.remove(&name);
        }
        self.clone()
    }

    /// Unbind global callbacks
    #[wasm_bindgen(js_name = unbind_global)]
    pub fn unbind_global(&self) -> WasmChannel {
        let mut callbacks = self.callbacks.write();
        callbacks.remove("__all__");
        self.clone()
    }

    /// Unbind all callbacks (specific and global)
    #[wasm_bindgen(js_name = unbind_all)]
    pub fn unbind_all(&self) -> WasmChannel {
        let mut callbacks = self.callbacks.write();
        callbacks.clear();
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
#[wasm_bindgen]
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
#[wasm_bindgen]
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
