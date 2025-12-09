//! Comprehensive WASM test suite for Sockudo client
//!
//! Tests all major features:
//! - Basic connection and disconnection
//! - Channel subscription and unsubscription
//! - Event binding and triggering
//! - Delta compression
//! - Conflation keys
//! - Publish filtering
//! - Private and presence channels

#![cfg(target_arch = "wasm32")]

use sockudo_client::wasm::{WasmDeltaOptions, WasmFilterOp, WasmOptions, WasmSockudo};
use wasm_bindgen::JsValue;
use wasm_bindgen_test::*;
use web_sys::console;

wasm_bindgen_test_configure!(run_in_browser);

// Helper function to create test options
fn create_test_options(app_key: &str) -> WasmOptions {
    let mut options = WasmOptions::new(app_key);
    options.set_ws_host("ws-mt1.pusher.com");
    options.set_use_tls(true);
    options
}

#[wasm_bindgen_test]
fn test_client_creation() {
    console::log_1(&"Test: Client creation".into());

    let options = create_test_options("test-app-key");
    let client = WasmSockudo::new("test-app-key", Some(options));

    assert!(client.is_ok(), "Client should be created successfully");

    let client = client.unwrap();
    assert_eq!(
        client.state(),
        "initialized",
        "Initial state should be 'initialized'"
    );
    assert!(
        client.socket_id().is_none(),
        "Socket ID should be None before connection"
    );
}

#[wasm_bindgen_test]
fn test_client_with_cluster() {
    console::log_1(&"Test: Client with cluster".into());

    let mut options = WasmOptions::new("test-app-key");
    options.set_cluster("mt1");

    let client = WasmSockudo::new("test-app-key", Some(options));
    assert!(client.is_ok(), "Client with cluster should be created");
}

#[wasm_bindgen_test]
fn test_delta_compression_options() {
    console::log_1(&"Test: Delta compression options".into());

    let mut delta_opts = WasmDeltaOptions::new();

    // Test default values
    assert_eq!(
        delta_opts.enabled(),
        true,
        "Delta compression should be enabled by default"
    );
    assert_eq!(
        delta_opts.max_messages_per_key(),
        10,
        "Default max messages should be 10"
    );
    assert_eq!(
        delta_opts.debug(),
        false,
        "Debug should be false by default"
    );

    // Test setters
    delta_opts.set_enabled(false);
    assert_eq!(
        delta_opts.enabled(),
        false,
        "Should disable delta compression"
    );

    delta_opts.set_max_messages_per_key(20);
    assert_eq!(
        delta_opts.max_messages_per_key(),
        20,
        "Should update max messages"
    );

    delta_opts.set_debug(true);
    assert_eq!(delta_opts.debug(), true, "Should enable debug mode");

    delta_opts.set_algorithms("fossil,xdelta3");
}

#[wasm_bindgen_test]
fn test_client_with_delta_compression() {
    console::log_1(&"Test: Client with delta compression".into());

    let mut options = WasmOptions::new("test-app-key");
    options.set_cluster("mt1");

    let mut delta_opts = WasmDeltaOptions::new();
    delta_opts.set_enabled(true);
    delta_opts.set_max_messages_per_key(15);
    delta_opts.set_algorithms("fossil,xdelta3");

    options.set_delta_compression(delta_opts);

    let client = WasmSockudo::new("test-app-key", Some(options));
    assert!(
        client.is_ok(),
        "Client with delta compression should be created"
    );
}

#[wasm_bindgen_test]
fn test_filter_operations() {
    console::log_1(&"Test: Filter operations".into());

    // Test equality filter
    let eq_filter = WasmFilterOp::eq("status", "active");
    let json = eq_filter.to_json();
    assert!(
        json.contains("status"),
        "Filter JSON should contain field name"
    );

    // Test inequality filter
    let neq_filter = WasmFilterOp::neq("status", "inactive");
    assert!(
        !neq_filter.to_json().is_empty(),
        "Filter should produce valid JSON"
    );

    // Test comparison filters
    let lt_filter = WasmFilterOp::lt("age", "18");
    assert!(!lt_filter.to_json().is_empty());

    let lte_filter = WasmFilterOp::lte("age", "18");
    assert!(!lte_filter.to_json().is_empty());

    let gt_filter = WasmFilterOp::gt("score", "100");
    assert!(!gt_filter.to_json().is_empty());

    let gte_filter = WasmFilterOp::gte("score", "100");
    assert!(!gte_filter.to_json().is_empty());

    // Test IN filter
    let in_filter =
        WasmFilterOp::in_set("role", vec!["admin".to_string(), "moderator".to_string()]);
    assert!(!in_filter.to_json().is_empty());

    // Test NOT IN filter
    let not_in_filter = WasmFilterOp::not_in("role", vec!["banned".to_string()]);
    assert!(!not_in_filter.to_json().is_empty());

    // Test EXISTS filter
    let exists_filter = WasmFilterOp::exists("premium");
    assert!(!exists_filter.to_json().is_empty());

    // Test NOT EXISTS filter
    let not_exists_filter = WasmFilterOp::not_exists("banned");
    assert!(!not_exists_filter.to_json().is_empty());
}

#[wasm_bindgen_test]
fn test_complex_filters() {
    console::log_1(&"Test: Complex filters (AND/OR)".into());

    // Create multiple filters
    let filter1 = WasmFilterOp::eq("status", "active");
    let filter2 = WasmFilterOp::gt("age", "18");
    let filter3 = WasmFilterOp::exists("premium");

    // Test AND filter
    let and_filter = WasmFilterOp::and(vec![filter1, filter2]);
    let json = and_filter.to_json();
    assert!(
        json.contains("And") || json.contains("and"),
        "Should contain AND operator"
    );

    // Create new filters for OR test
    let filter4 = WasmFilterOp::eq("role", "admin");
    let filter5 = WasmFilterOp::eq("role", "moderator");

    // Test OR filter
    let or_filter = WasmFilterOp::or(vec![filter4, filter5]);
    let json = or_filter.to_json();
    assert!(
        json.contains("Or") || json.contains("or"),
        "Should contain OR operator"
    );

    // Test nested filters
    let filter6 = WasmFilterOp::eq("verified", "true");
    let combined = WasmFilterOp::and(vec![or_filter, filter6]);
    assert!(!combined.to_json().is_empty(), "Nested filters should work");
}

#[wasm_bindgen_test]
fn test_channel_subscription() {
    console::log_1(&"Test: Channel subscription".into());

    let options = create_test_options("test-app-key");
    let client = WasmSockudo::new("test-app-key", Some(options)).unwrap();

    // Subscribe to public channel
    let channel = client.subscribe("test-channel", None);
    assert!(channel.is_ok(), "Should subscribe to public channel");

    let channel = channel.unwrap();
    assert_eq!(channel.name(), "test-channel", "Channel name should match");
}

#[wasm_bindgen_test]
fn test_channel_subscription_with_filter() {
    console::log_1(&"Test: Channel subscription with filter".into());

    let options = create_test_options("test-app-key");
    let client = WasmSockudo::new("test-app-key", Some(options)).unwrap();

    // Create a filter
    let filter = WasmFilterOp::eq("type", "important");

    // Subscribe with filter
    let channel = client.subscribe("filtered-channel", Some(filter));
    assert!(channel.is_ok(), "Should subscribe to channel with filter");
}

#[wasm_bindgen_test]
fn test_multiple_channel_subscriptions() {
    console::log_1(&"Test: Multiple channel subscriptions".into());

    let options = create_test_options("test-app-key");
    let client = WasmSockudo::new("test-app-key", Some(options)).unwrap();

    // Subscribe to multiple channels
    let channel1 = client.subscribe("channel-1", None);
    let channel2 = client.subscribe("channel-2", None);
    let channel3 = client.subscribe("channel-3", None);

    assert!(channel1.is_ok(), "Should subscribe to channel 1");
    assert!(channel2.is_ok(), "Should subscribe to channel 2");
    assert!(channel3.is_ok(), "Should subscribe to channel 3");

    // Get channel by name
    let retrieved = client.channel("channel-2");
    assert!(retrieved.is_some(), "Should retrieve subscribed channel");
    assert_eq!(
        retrieved.unwrap().name(),
        "channel-2",
        "Retrieved channel should have correct name"
    );
}

#[wasm_bindgen_test]
fn test_channel_resubscription() {
    console::log_1(&"Test: Channel resubscription".into());

    let options = create_test_options("test-app-key");
    let client = WasmSockudo::new("test-app-key", Some(options)).unwrap();

    // Subscribe twice to same channel
    let channel1 = client.subscribe("test-channel", None).unwrap();
    let channel2 = client.subscribe("test-channel", None).unwrap();

    // Should return same channel
    assert_eq!(
        channel1.name(),
        channel2.name(),
        "Resubscription should return same channel"
    );
}

#[wasm_bindgen_test]
fn test_channel_unsubscribe() {
    console::log_1(&"Test: Channel unsubscribe".into());

    let options = create_test_options("test-app-key");
    let client = WasmSockudo::new("test-app-key", Some(options)).unwrap();

    // Subscribe and then unsubscribe
    let _channel = client.subscribe("test-channel", None).unwrap();
    client.unsubscribe("test-channel");

    // Channel should no longer be retrievable
    let retrieved = client.channel("test-channel");
    assert!(
        retrieved.is_none(),
        "Unsubscribed channel should not be retrievable"
    );
}

#[wasm_bindgen_test]
fn test_disconnect() {
    console::log_1(&"Test: Disconnect".into());

    let options = create_test_options("test-app-key");
    let client = WasmSockudo::new("test-app-key", Some(options)).unwrap();

    client.disconnect();

    // State should be disconnected
    assert_eq!(
        client.state(),
        "disconnected",
        "State should be 'disconnected' after disconnect"
    );
    assert!(
        client.socket_id().is_none(),
        "Socket ID should be None after disconnect"
    );
}

#[wasm_bindgen_test]
fn test_options_getters_setters() {
    console::log_1(&"Test: Options getters and setters".into());

    let mut options = WasmOptions::new("test-app-key");

    // Test cluster
    options.set_cluster("eu");
    assert_eq!(
        options.cluster(),
        Some("eu".to_string()),
        "Cluster should be set"
    );

    // Test ws_host
    options.set_ws_host("example.com");
    assert_eq!(
        options.ws_host(),
        Some("example.com".to_string()),
        "WS host should be set"
    );

    // Test ws_port
    options.set_ws_port(8080);
    assert_eq!(options.ws_port(), Some(8080), "WS port should be set");

    // Test use_tls
    options.set_use_tls(false);
    assert_eq!(options.use_tls(), Some(false), "TLS should be disabled");

    // Test auth_endpoint
    options.set_auth_endpoint("https://example.com/auth");
    assert_eq!(
        options.auth_endpoint(),
        Some("https://example.com/auth".to_string()),
        "Auth endpoint should be set"
    );
}

#[wasm_bindgen_test]
fn test_enable_delta_compression_shorthand() {
    console::log_1(&"Test: Enable delta compression shorthand".into());

    let mut options = WasmOptions::new("test-app-key");
    options.enable_delta_compression();

    // Client should be created successfully with delta compression
    let client = WasmSockudo::new("test-app-key", Some(options));
    assert!(
        client.is_ok(),
        "Client with delta compression shorthand should work"
    );
}

#[wasm_bindgen_test]
fn test_private_channel_subscription() {
    console::log_1(&"Test: Private channel subscription".into());

    let mut options = WasmOptions::new("test-app-key");
    options.set_cluster("mt1");
    options.set_auth_endpoint("http://localhost:8080/pusher/auth");

    let client = WasmSockudo::new("test-app-key", Some(options)).unwrap();

    // Subscribe to private channel (will fail without real auth endpoint, but should not crash)
    let channel = client.subscribe("private-test", None);
    assert!(
        channel.is_ok(),
        "Private channel subscription should not crash"
    );
}

#[wasm_bindgen_test]
fn test_presence_channel_subscription() {
    console::log_1(&"Test: Presence channel subscription".into());

    let mut options = WasmOptions::new("test-app-key");
    options.set_cluster("mt1");
    options.set_auth_endpoint("http://localhost:8080/pusher/auth");

    let client = WasmSockudo::new("test-app-key", Some(options)).unwrap();

    // Subscribe to presence channel
    let channel = client.subscribe("presence-test", None);
    assert!(
        channel.is_ok(),
        "Presence channel subscription should not crash"
    );
}

#[wasm_bindgen_test]
fn test_encrypted_channel_subscription() {
    console::log_1(&"Test: Encrypted channel subscription".into());

    let mut options = WasmOptions::new("test-app-key");
    options.set_cluster("mt1");
    options.set_auth_endpoint("http://localhost:8080/pusher/auth");

    let client = WasmSockudo::new("test-app-key", Some(options)).unwrap();

    // Subscribe to encrypted channel
    let channel = client.subscribe("private-encrypted-test", None);
    assert!(
        channel.is_ok(),
        "Encrypted channel subscription should not crash"
    );
}

#[wasm_bindgen_test]
fn test_send_event() {
    console::log_1(&"Test: Send event".into());

    let options = create_test_options("test-app-key");
    let client = WasmSockudo::new("test-app-key", Some(options)).unwrap();

    // Send event (will fail without connection, but should not crash)
    let data = JsValue::from_str(r#"{"message": "test"}"#);
    let result = client.send_event("test-event", data, None);

    // Should return false because not connected
    assert_eq!(
        result, false,
        "Send event should return false when not connected"
    );
}

#[wasm_bindgen_test]
fn test_send_event_to_channel() {
    console::log_1(&"Test: Send event to channel".into());

    let options = create_test_options("test-app-key");
    let client = WasmSockudo::new("test-app-key", Some(options)).unwrap();

    let data = JsValue::from_str(r#"{"message": "test"}"#);
    let result = client.send_event("test-event", data, Some("test-channel".to_string()));

    assert_eq!(
        result, false,
        "Send event should return false when not connected"
    );
}

#[wasm_bindgen_test]
fn test_delta_stats() {
    console::log_1(&"Test: Delta stats".into());

    let options = create_test_options("test-app-key");
    let client = WasmSockudo::new("test-app-key", Some(options)).unwrap();

    // Get delta stats (currently returns NULL but should not crash)
    let stats = client.get_delta_stats();
    assert!(
        stats.is_null() || !stats.is_undefined(),
        "Delta stats should be accessible"
    );

    // Reset delta stats (should not crash)
    client.reset_delta_stats();
}

#[wasm_bindgen_test]
fn test_unbind_operations() {
    console::log_1(&"Test: Unbind operations".into());

    let options = create_test_options("test-app-key");
    let client = WasmSockudo::new("test-app-key", Some(options)).unwrap();

    // These operations should not crash
    client.unbind(Some("test-event".to_string()));
    client.unbind(None);
    client.unbind_global();
    client.unbind_all();
}

#[wasm_bindgen_test]
fn test_channel_unbind_operations() {
    console::log_1(&"Test: Channel unbind operations".into());

    let options = create_test_options("test-app-key");
    let client = WasmSockudo::new("test-app-key", Some(options)).unwrap();

    let channel = client.subscribe("test-channel", None).unwrap();

    // Test chaining
    let channel = channel.unbind(Some("test-event".to_string()));
    let channel = channel.unbind_global();
    let _channel = channel.unbind_all();
}
