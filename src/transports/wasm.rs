//! WASM WebSocket transport implementation using web-sys.

use async_trait::async_trait;
use parking_lot::RwLock;
use std::sync::Arc;
use tracing::{debug, error, info};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{CloseEvent, ErrorEvent, MessageEvent, WebSocket};

use super::transport::{MessageCallback, Transport};
use crate::error::{Result, SockudoError};

/// WASM WebSocket transport
pub struct WasmTransport {
    /// WebSocket instance
    ws: Arc<RwLock<Option<WebSocket>>>,
    /// Connected flag
    connected: Arc<RwLock<bool>>,
    /// Message callback
    on_message: Arc<RwLock<Option<MessageCallback>>>,
    /// Close callback
    on_close: Arc<RwLock<Option<Box<dyn Fn(Option<u16>, Option<String>)>>>>,
    /// Error callback
    on_error: Arc<RwLock<Option<Box<dyn Fn(String)>>>>,
    /// Closures for event handlers (need to keep them alive)
    _closures: Arc<RwLock<Vec<Closure<dyn FnMut(JsValue)>>>>,
}

impl WasmTransport {
    /// Create a new WASM transport
    pub fn new() -> Self {
        Self {
            ws: Arc::new(RwLock::new(None)),
            connected: Arc::new(RwLock::new(false)),
            on_message: Arc::new(RwLock::new(None)),
            on_close: Arc::new(RwLock::new(None)),
            on_error: Arc::new(RwLock::new(None)),
            _closures: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Setup event listeners on the WebSocket
    fn setup_listeners(&self, ws: &WebSocket) {
        let ws_clone = ws.clone();
        let connected = self.connected.clone();
        let on_message_cb = self.on_message.clone();
        let on_close_cb = self.on_close.clone();
        let on_error_cb = self.on_error.clone();
        let mut closures = Vec::new();

        // onopen handler
        {
            let connected = connected.clone();
            let onopen = Closure::wrap(Box::new(move |_event: JsValue| {
                info!("WebSocket connection opened");
                *connected.write() = true;
            }) as Box<dyn FnMut(JsValue)>);

            ws.set_onopen(Some(onopen.as_ref().unchecked_ref()));
            closures.push(onopen);
        }

        // onmessage handler
        {
            let on_message = on_message_cb.clone();
            let onmessage = Closure::wrap(Box::new(move |event: JsValue| {
                if let Ok(message_event) = event.dyn_into::<MessageEvent>() {
                    if let Ok(text) = message_event.data().dyn_into::<js_sys::JsString>() {
                        let text_str = text.as_string().unwrap_or_default();
                        debug!("Received message: {}", text_str);

                        if let Some(ref callback) = *on_message.read() {
                            callback(&text_str);
                        }
                    }
                }
            }) as Box<dyn FnMut(JsValue)>);

            ws.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
            closures.push(onmessage);
        }

        // onclose handler
        {
            let connected = connected.clone();
            let on_close = on_close_cb.clone();
            let onclose = Closure::wrap(Box::new(move |event: JsValue| {
                info!("WebSocket connection closed");
                *connected.write() = false;

                if let Ok(close_event) = event.dyn_into::<CloseEvent>() {
                    let code = close_event.code();
                    let reason = close_event.reason();

                    if let Some(ref callback) = *on_close.read() {
                        callback(
                            Some(code),
                            if reason.is_empty() {
                                None
                            } else {
                                Some(reason)
                            },
                        );
                    }
                }
            }) as Box<dyn FnMut(JsValue)>);

            ws.set_onclose(Some(onclose.as_ref().unchecked_ref()));
            closures.push(onclose);
        }

        // onerror handler
        {
            let connected = connected.clone();
            let on_error = on_error_cb.clone();
            let onerror = Closure::wrap(Box::new(move |event: JsValue| {
                error!("WebSocket error occurred");
                *connected.write() = false;

                let error_msg = if let Ok(error_event) = event.dyn_into::<ErrorEvent>() {
                    format!("WebSocket error: {}", error_event.message())
                } else {
                    "Unknown WebSocket error".to_string()
                };

                if let Some(ref callback) = *on_error.read() {
                    callback(error_msg);
                }
            }) as Box<dyn FnMut(JsValue)>);

            ws.set_onerror(Some(onerror.as_ref().unchecked_ref()));
            closures.push(onerror);
        }

        // Store closures to keep them alive
        *self._closures.write() = closures;
    }
}

impl Default for WasmTransport {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait(?Send)]
impl Transport for WasmTransport {
    async fn connect(&mut self, url: &str) -> Result<()> {
        if self.is_connected() {
            return Err(SockudoError::invalid_state("Already connected"));
        }

        info!("Connecting to WebSocket: {}", url);

        // Create WebSocket
        let ws = WebSocket::new(url).map_err(|e| {
            SockudoError::connection(format!("Failed to create WebSocket: {:?}", e))
        })?;

        // Set binary type to arraybuffer (optional, but recommended)
        ws.set_binary_type(web_sys::BinaryType::Arraybuffer);

        // Setup event listeners
        self.setup_listeners(&ws);

        // Store WebSocket
        *self.ws.write() = Some(ws);

        // Wait for connection to establish
        // In WASM, the connection is async but we don't have a direct await mechanism
        // The onopen handler will set connected to true
        let connected = self.connected.clone();
        let mut attempts = 0;
        const MAX_ATTEMPTS: u32 = 50; // 5 seconds with 100ms intervals

        while !*connected.read() && attempts < MAX_ATTEMPTS {
            gloo_timers::future::TimeoutFuture::new(100).await;
            attempts += 1;
        }

        if !*connected.read() {
            return Err(SockudoError::connection("Connection timeout"));
        }

        info!("WebSocket connected successfully");
        Ok(())
    }

    async fn disconnect(&mut self) {
        if !self.is_connected() {
            return;
        }

        info!("Disconnecting WebSocket");

        if let Some(ws) = self.ws.write().take() {
            let _ = ws.close();
        }

        *self.connected.write() = false;

        // Clear closures
        self._closures.write().clear();

        info!("WebSocket disconnected");
    }

    async fn send(&self, message: &str) -> Result<()> {
        if !self.is_connected() {
            return Err(SockudoError::invalid_state("Not connected"));
        }

        debug!("Sending message: {}", message);

        let ws_lock = self.ws.read();
        if let Some(ws) = ws_lock.as_ref() {
            ws.send_with_str(message)
                .map_err(|e| SockudoError::websocket(format!("Send failed: {:?}", e)))?;
            Ok(())
        } else {
            Err(SockudoError::invalid_state("WebSocket not available"))
        }
    }

    async fn ping(&self) -> Result<()> {
        // Note: In browser WebSocket API, ping/pong is handled automatically
        // by the browser and not exposed to JavaScript
        debug!("Ping (automatic in browser WebSocket)");
        Ok(())
    }

    fn is_connected(&self) -> bool {
        *self.connected.read()
    }

    fn on_message(&mut self, callback: MessageCallback) {
        *self.on_message.write() = Some(callback);
    }

    fn on_close(&mut self, callback: Box<dyn Fn(Option<u16>, Option<String>)>) {
        *self.on_close.write() = Some(callback);
    }

    fn on_error(&mut self, callback: Box<dyn Fn(String)>) {
        *self.on_error.write() = Some(callback);
    }
}
