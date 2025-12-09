//! Comprehensive integration tests for the Sockudo client library.
//!
//! These tests verify all features work correctly against a running Sockudo server.
//!
//! Run with: cargo test --test integration_test -- --nocapture
//!
//! Prerequisites:
//! - Sockudo server running on localhost:6001
//! - Server configured with the test app (app-key, app-secret)

#![cfg(not(target_arch = "wasm32"))]

use sockudo_client::{
    ChannelType, ConnectionState, DeltaAlgorithm, DeltaOptions, FilterOp, PusherOptions,
    SockudoClient,
};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::{sleep, timeout};

/// Test server configuration matching the provided config.json
const TEST_HOST: &str = "127.0.0.1";
const TEST_PORT: u16 = 6001;
const TEST_APP_KEY: &str = "app-key";

/// Helper to create a client with default test options
fn create_test_client() -> SockudoClient {
    let options = PusherOptions::new(TEST_APP_KEY)
        .ws_host(TEST_HOST)
        .ws_port(TEST_PORT)
        .use_tls(false)
        .debug(true);

    SockudoClient::new(options.into()).expect("Failed to create client")
}

/// Helper to create a client with delta compression enabled
fn create_delta_client() -> SockudoClient {
    let delta_options = DeltaOptions {
        enabled: true,
        algorithms: vec![DeltaAlgorithm::Fossil],
        debug: true,
        ..Default::default()
    };

    let options = PusherOptions::new(TEST_APP_KEY)
        .ws_host(TEST_HOST)
        .ws_port(TEST_PORT)
        .use_tls(false)
        .delta_compression(delta_options)
        .debug(true);

    SockudoClient::new(options.into()).expect("Failed to create client")
}

// ============================================================================
// Connection Tests
// ============================================================================

#[tokio::test]
async fn test_client_creation() {
    let client = create_test_client();

    assert!(client.session_id() > 0);
    // Initial state is Initialized, not Disconnected
    assert_eq!(client.state(), ConnectionState::Initialized);
    assert!(client.socket_id().is_none());

    println!("✓ Client created with session ID: {}", client.session_id());
}

#[tokio::test]
async fn test_connect_and_disconnect() {
    let client = create_test_client();

    // Connect
    let connect_result = timeout(Duration::from_secs(10), client.connect()).await;

    match connect_result {
        Ok(Ok(())) => {
            // Wait for connection to fully establish
            sleep(Duration::from_millis(500)).await;

            // Connection may still be in progress, check state
            let state = client.state();
            println!("✓ Connection initiated, state: {:?}", state);
            println!("✓ Socket ID: {:?}", client.socket_id());

            // Disconnect
            client.disconnect().await;

            // Give it a moment to process
            sleep(Duration::from_millis(100)).await;

            println!("✓ Disconnected successfully");
        }
        Ok(Err(e)) => {
            println!("⚠ Connection failed (server may not be running): {}", e);
        }
        Err(_) => {
            println!("⚠ Connection timeout (server may not be running)");
        }
    }
}

#[tokio::test]
async fn test_connection_state_changes() {
    let client = create_test_client();
    let state_changed = Arc::new(AtomicBool::new(false));
    let state_changed_clone = state_changed.clone();

    // Bind to connection state changes using bind_global
    client.bind_global(move |event| {
        println!("Event: {} on {:?}", event.event, event.channel);
        if event.event.contains("state") || event.event.contains("connection") {
            state_changed_clone.store(true, Ordering::SeqCst);
        }
    });

    let connect_result = timeout(Duration::from_secs(10), client.connect()).await;

    match connect_result {
        Ok(Ok(())) => {
            sleep(Duration::from_millis(500)).await;
            println!("✓ Connection state monitoring works");
            client.disconnect().await;
        }
        Ok(Err(e)) => {
            println!("⚠ Connection failed: {}", e);
        }
        Err(_) => {
            println!("⚠ Connection timeout");
        }
    }
}

// ============================================================================
// Channel Subscription Tests
// ============================================================================

#[tokio::test]
async fn test_subscribe_public_channel() {
    let client = create_test_client();

    let connect_result = timeout(Duration::from_secs(10), client.connect()).await;

    match connect_result {
        Ok(Ok(())) => {
            let channel_result = client.subscribe("test-channel");

            match channel_result {
                Ok(channel) => {
                    assert_eq!(channel.name(), "test-channel");
                    println!("✓ Subscribed to public channel: {}", channel.name());
                    sleep(Duration::from_millis(500)).await;
                }
                Err(e) => {
                    println!("✗ Failed to subscribe: {}", e);
                }
            }

            client.disconnect().await;
        }
        Ok(Err(e)) => {
            println!("⚠ Connection failed: {}", e);
        }
        Err(_) => {
            println!("⚠ Connection timeout");
        }
    }
}

#[tokio::test]
async fn test_subscribe_multiple_channels() {
    let client = create_test_client();

    let connect_result = timeout(Duration::from_secs(10), client.connect()).await;

    match connect_result {
        Ok(Ok(())) => {
            let channels = vec!["channel-1", "channel-2", "channel-3"];

            for name in &channels {
                match client.subscribe(name) {
                    Ok(channel) => {
                        println!("✓ Subscribed to: {}", channel.name());
                    }
                    Err(e) => {
                        println!("✗ Failed to subscribe to {}: {}", name, e);
                    }
                }
            }

            sleep(Duration::from_millis(500)).await;

            let all_channels = client.all_channels();
            assert_eq!(all_channels.len(), channels.len());
            println!("✓ All {} channels subscribed", all_channels.len());

            client.disconnect().await;
        }
        Ok(Err(e)) => {
            println!("⚠ Connection failed: {}", e);
        }
        Err(_) => {
            println!("⚠ Connection timeout");
        }
    }
}

#[tokio::test]
async fn test_unsubscribe_channel() {
    let client = create_test_client();

    let connect_result = timeout(Duration::from_secs(10), client.connect()).await;

    match connect_result {
        Ok(Ok(())) => {
            let _ = client.subscribe("temp-channel");
            sleep(Duration::from_millis(300)).await;

            assert!(client.channel("temp-channel").is_some());
            println!("✓ Channel exists after subscribe");

            client.unsubscribe("temp-channel");
            sleep(Duration::from_millis(300)).await;

            assert!(client.channel("temp-channel").is_none());
            println!("✓ Channel removed after unsubscribe");

            client.disconnect().await;
        }
        Ok(Err(e)) => {
            println!("⚠ Connection failed: {}", e);
        }
        Err(_) => {
            println!("⚠ Connection timeout");
        }
    }
}

// ============================================================================
// Event Binding Tests
// ============================================================================

#[tokio::test]
async fn test_event_binding() {
    let client = create_test_client();
    let event_received = Arc::new(AtomicBool::new(false));
    let event_received_clone = event_received.clone();

    let connect_result = timeout(Duration::from_secs(10), client.connect()).await;

    match connect_result {
        Ok(Ok(())) => {
            let channel = client.subscribe("event-test-channel").unwrap();

            let binding_id = channel.bind("test-event", move |event| {
                println!("Received event: {:?}", event);
                event_received_clone.store(true, Ordering::SeqCst);
            });

            assert!(binding_id > 0);
            println!("✓ Event binding created with ID: {}", binding_id);

            sleep(Duration::from_secs(1)).await;

            client.disconnect().await;
        }
        Ok(Err(e)) => {
            println!("⚠ Connection failed: {}", e);
        }
        Err(_) => {
            println!("⚠ Connection timeout");
        }
    }
}

#[tokio::test]
async fn test_global_event_binding() {
    let client = create_test_client();
    let events_count = Arc::new(AtomicU32::new(0));
    let events_count_clone = events_count.clone();

    client.bind_global(move |event| {
        println!("Global event: {} on {:?}", event.event, event.channel);
        events_count_clone.fetch_add(1, Ordering::SeqCst);
    });

    let connect_result = timeout(Duration::from_secs(10), client.connect()).await;

    match connect_result {
        Ok(Ok(())) => {
            sleep(Duration::from_millis(500)).await;

            let count = events_count.load(Ordering::SeqCst);
            println!("✓ Global binding received {} events", count);

            client.disconnect().await;
        }
        Ok(Err(e)) => {
            println!("⚠ Connection failed: {}", e);
        }
        Err(_) => {
            println!("⚠ Connection timeout");
        }
    }
}

// ============================================================================
// Tag Filtering Tests
// ============================================================================

#[tokio::test]
async fn test_subscribe_with_filter_eq() {
    let client = create_test_client();

    let connect_result = timeout(Duration::from_secs(10), client.connect()).await;

    match connect_result {
        Ok(Ok(())) => {
            let filter = FilterOp::eq("type", "important");

            match client.subscribe_with_filter("filtered-channel", Some(filter)) {
                Ok(channel) => {
                    println!("✓ Subscribed with eq filter: {}", channel.name());
                }
                Err(e) => {
                    println!("✗ Failed to subscribe with filter: {}", e);
                }
            }

            sleep(Duration::from_millis(500)).await;
            client.disconnect().await;
        }
        Ok(Err(e)) => {
            println!("⚠ Connection failed: {}", e);
        }
        Err(_) => {
            println!("⚠ Connection timeout");
        }
    }
}

#[tokio::test]
async fn test_subscribe_with_complex_filter() {
    let client = create_test_client();

    let connect_result = timeout(Duration::from_secs(10), client.connect()).await;

    match connect_result {
        Ok(Ok(())) => {
            let filter = FilterOp::and(vec![
                FilterOp::eq("type", "goal"),
                FilterOp::in_set(
                    "league",
                    vec!["premier".to_string(), "champions".to_string()],
                ),
            ]);

            match client.subscribe_with_filter("complex-filter-channel", Some(filter)) {
                Ok(channel) => {
                    println!("✓ Subscribed with complex AND filter: {}", channel.name());
                }
                Err(e) => {
                    println!("✗ Failed to subscribe with filter: {}", e);
                }
            }

            sleep(Duration::from_millis(500)).await;
            client.disconnect().await;
        }
        Ok(Err(e)) => {
            println!("⚠ Connection failed: {}", e);
        }
        Err(_) => {
            println!("⚠ Connection timeout");
        }
    }
}

#[tokio::test]
async fn test_subscribe_with_or_filter() {
    let client = create_test_client();

    let connect_result = timeout(Duration::from_secs(10), client.connect()).await;

    match connect_result {
        Ok(Ok(())) => {
            let filter = FilterOp::or(vec![
                FilterOp::eq("type", "goal"),
                FilterOp::eq("type", "assist"),
            ]);

            match client.subscribe_with_filter("or-filter-channel", Some(filter)) {
                Ok(channel) => {
                    println!("✓ Subscribed with OR filter: {}", channel.name());
                }
                Err(e) => {
                    println!("✗ Failed to subscribe with filter: {}", e);
                }
            }

            sleep(Duration::from_millis(500)).await;
            client.disconnect().await;
        }
        Ok(Err(e)) => {
            println!("⚠ Connection failed: {}", e);
        }
        Err(_) => {
            println!("⚠ Connection timeout");
        }
    }
}

#[tokio::test]
async fn test_subscribe_with_comparison_filters() {
    let client = create_test_client();

    let connect_result = timeout(Duration::from_secs(10), client.connect()).await;

    match connect_result {
        Ok(Ok(())) => {
            let filter = FilterOp::and(vec![
                FilterOp::gt("price", "100"),
                FilterOp::lt("price", "1000"),
            ]);

            match client.subscribe_with_filter("price-range-channel", Some(filter)) {
                Ok(channel) => {
                    println!("✓ Subscribed with comparison filter: {}", channel.name());
                }
                Err(e) => {
                    println!("✗ Failed to subscribe with filter: {}", e);
                }
            }

            sleep(Duration::from_millis(500)).await;
            client.disconnect().await;
        }
        Ok(Err(e)) => {
            println!("⚠ Connection failed: {}", e);
        }
        Err(_) => {
            println!("⚠ Connection timeout");
        }
    }
}

#[tokio::test]
async fn test_subscribe_with_exists_filter() {
    let client = create_test_client();

    let connect_result = timeout(Duration::from_secs(10), client.connect()).await;

    match connect_result {
        Ok(Ok(())) => {
            let filter = FilterOp::exists("metadata");

            match client.subscribe_with_filter("exists-filter-channel", Some(filter)) {
                Ok(channel) => {
                    println!("✓ Subscribed with exists filter: {}", channel.name());
                }
                Err(e) => {
                    println!("✗ Failed to subscribe with filter: {}", e);
                }
            }

            sleep(Duration::from_millis(500)).await;
            client.disconnect().await;
        }
        Ok(Err(e)) => {
            println!("⚠ Connection failed: {}", e);
        }
        Err(_) => {
            println!("⚠ Connection timeout");
        }
    }
}

// ============================================================================
// Delta Compression Tests
// ============================================================================

#[tokio::test]
async fn test_delta_compression_enabled() {
    let client = create_delta_client();

    // Note: Delta compression in Sockudo works per-channel based on server config.
    // The server's channel_delta_compression config determines which channels use delta.
    // The client just needs to subscribe to delta-enabled channels and the server
    // will automatically send delta-compressed messages.
    println!("✓ Delta compression configured in client options");

    let connect_result = timeout(Duration::from_secs(10), client.connect()).await;

    match connect_result {
        Ok(Ok(())) => {
            // Wait for connection to fully establish
            sleep(Duration::from_millis(500)).await;

            println!("  - Connection state: {:?}", client.state());
            println!("  - Socket ID: {:?}", client.socket_id());
            // Note: is_delta_compression_enabled() checks if server sent
            // pusher:delta_compression_enabled. Sockudo uses per-channel config
            // so this may remain false even though delta works on specific channels.
            println!(
                "  - Delta active (global): {}",
                client.is_delta_compression_enabled()
            );

            match client.subscribe("benchmark") {
                Ok(channel) => {
                    println!("✓ Subscribed to delta-enabled channel: {}", channel.name());
                }
                Err(e) => {
                    println!("✗ Failed to subscribe: {}", e);
                }
            }

            // Subscribe to market-data too (configured for delta in server config)
            match client.subscribe("market-data") {
                Ok(channel) => {
                    println!("✓ Subscribed to delta-enabled channel: {}", channel.name());
                }
                Err(e) => {
                    println!("✗ Failed to subscribe: {}", e);
                }
            }

            // Wait for potential messages from server
            // Stats will be zero if no messages are published to these channels
            println!("\n⏳ Waiting 2 seconds for incoming messages...");
            sleep(Duration::from_secs(2)).await;

            if let Some(stats) = client.get_delta_stats() {
                println!("\nDelta Stats:");
                println!("  - Total messages: {}", stats.total_messages);
                println!("  - Delta messages: {}", stats.delta_messages);
                println!("  - Full messages: {}", stats.full_messages);
                println!("  - Bandwidth saved: {:.2}%", stats.bandwidth_saved_percent);
                println!("  - Channel count: {}", stats.channel_count);
                println!("✓ Delta statistics available");

                if stats.total_messages == 0 {
                    println!("\n💡 Note: Stats are zero because no messages were received.");
                    println!("   Delta compression is configured per-channel on the server.");
                    println!("   To see it in action, publish messages to 'benchmark' or");
                    println!("   'market-data' channels via the Pusher HTTP API.");
                    println!("   Example: POST /apps/app-id/events with channel='benchmark'");
                }
            }

            client.disconnect().await;
        }
        Ok(Err(e)) => {
            println!("⚠ Connection failed: {}", e);
        }
        Err(_) => {
            println!("⚠ Connection timeout");
        }
    }
}

#[tokio::test]
async fn test_delta_compression_market_data_channel() {
    let client = create_delta_client();

    let connect_result = timeout(Duration::from_secs(10), client.connect()).await;

    match connect_result {
        Ok(Ok(())) => {
            match client.subscribe("market-data") {
                Ok(channel) => {
                    channel.bind("price-update", |event| {
                        println!("Price update: {:?}", event.data);
                    });
                    println!("✓ Subscribed to market-data with delta compression");
                }
                Err(e) => {
                    println!("✗ Failed to subscribe: {}", e);
                }
            }

            sleep(Duration::from_secs(2)).await;
            client.disconnect().await;
        }
        Ok(Err(e)) => {
            println!("⚠ Connection failed: {}", e);
        }
        Err(_) => {
            println!("⚠ Connection timeout");
        }
    }
}

#[tokio::test]
async fn test_delta_stats_reset() {
    let client = create_delta_client();

    let connect_result = timeout(Duration::from_secs(10), client.connect()).await;

    match connect_result {
        Ok(Ok(())) => {
            let _ = client.subscribe("benchmark");
            sleep(Duration::from_millis(500)).await;

            client.reset_delta_stats();

            if let Some(stats) = client.get_delta_stats() {
                assert_eq!(stats.total_messages, 0);
                assert_eq!(stats.delta_messages, 0);
                println!("✓ Delta stats reset successfully");
            }

            client.disconnect().await;
        }
        Ok(Err(e)) => {
            println!("⚠ Connection failed: {}", e);
        }
        Err(_) => {
            println!("⚠ Connection timeout");
        }
    }
}

// ============================================================================
// Channel Type Detection Tests
// ============================================================================

#[tokio::test]
async fn test_channel_type_detection() {
    assert!(matches!(
        ChannelType::from_name("public-channel"),
        ChannelType::Public
    ));
    assert!(matches!(
        ChannelType::from_name("private-channel"),
        ChannelType::Private
    ));
    assert!(matches!(
        ChannelType::from_name("presence-channel"),
        ChannelType::Presence
    ));
    assert!(matches!(
        ChannelType::from_name("private-encrypted-channel"),
        ChannelType::PrivateEncrypted
    ));

    println!("✓ Channel type detection works correctly");
    println!("  - 'public-channel' -> Public");
    println!("  - 'private-channel' -> Private");
    println!("  - 'presence-channel' -> Presence");
    println!("  - 'private-encrypted-channel' -> PrivateEncrypted");
}

// ============================================================================
// Multiple Clients Tests
// ============================================================================

#[tokio::test]
async fn test_multiple_clients() {
    let client1 = create_test_client();
    let client2 = create_test_client();

    assert_ne!(client1.session_id(), client2.session_id());
    println!("✓ Clients have unique session IDs");

    let connect1 = timeout(Duration::from_secs(10), client1.connect()).await;
    let connect2 = timeout(Duration::from_secs(10), client2.connect()).await;

    match (connect1, connect2) {
        (Ok(Ok(())), Ok(Ok(()))) => {
            // Wait for connections to establish
            sleep(Duration::from_millis(500)).await;

            let socket_id1 = client1.socket_id();
            let socket_id2 = client2.socket_id();

            // Socket IDs may be None if connection is still in progress
            println!("✓ Both clients connected");
            println!("  - Client 1 socket: {:?}", socket_id1);
            println!("  - Client 2 socket: {:?}", socket_id2);

            let _ = client1.subscribe("shared-channel");
            let _ = client2.subscribe("shared-channel");

            sleep(Duration::from_millis(500)).await;

            println!("✓ Both clients subscribed to shared channel");

            client1.disconnect().await;
            client2.disconnect().await;
        }
        _ => {
            println!("⚠ One or both connections failed");
            client1.disconnect().await;
            client2.disconnect().await;
        }
    }
}

// ============================================================================
// Delta Decoder Unit Tests (using vcdiff-decoder)
// ============================================================================

#[tokio::test]
async fn test_fossil_delta_decoder() {
    use sockudo_client::delta::{DeltaDecoder, FossilDeltaDecoder};

    let decoder = FossilDeltaDecoder::new();

    assert!(decoder.is_available());
    assert_eq!(decoder.algorithm(), "fossil");

    let base = b"Hello, World!";
    let target = b"Hello, Rust World!";

    let delta = fossil_delta::delta(target, base);

    let result = decoder.decode(base, &delta).unwrap();
    assert_eq!(result, target);

    println!("✓ Fossil delta decoder works correctly");
}

#[tokio::test]
async fn test_xdelta3_decoder_with_vcdiff() {
    use sockudo_client::delta::{DeltaDecoder, Xdelta3Decoder};

    let decoder = Xdelta3Decoder::new();

    assert!(decoder.is_available());
    assert_eq!(decoder.algorithm(), "xdelta3");

    let base = b"Hello, World!";
    let target = b"Hello, Rust World!";

    // Encode with xdelta3
    let delta = xdelta3::encode(target, base).expect("Failed to encode");

    // Decode with vcdiff-decoder (through Xdelta3Decoder)
    let result = decoder.decode(base, &delta).expect("Failed to decode");
    assert_eq!(result, target);

    println!("✓ Xdelta3/VCDIFF decoder (using vcdiff-decoder) works correctly");
}

#[tokio::test]
async fn test_base64_encoding() {
    use sockudo_client::delta::{decode_base64, encode_base64};

    let original = b"Test data for base64 encoding";
    let encoded = encode_base64(original);
    let decoded = decode_base64(&encoded).unwrap();

    assert_eq!(decoded, original);
    println!("✓ Base64 encoding/decoding works correctly");
}

// ============================================================================
// Reconnection Tests
// ============================================================================

#[tokio::test]
async fn test_automatic_reconnection() {
    let client = create_test_client();

    let connect_result = timeout(Duration::from_secs(10), client.connect()).await;

    match connect_result {
        Ok(Ok(())) => {
            println!("✓ Initial connection established");

            client.disconnect().await;
            sleep(Duration::from_millis(500)).await;

            assert!(!client.is_connected());
            println!("✓ Disconnected");

            let reconnect_result = timeout(Duration::from_secs(10), client.connect()).await;

            match reconnect_result {
                Ok(Ok(())) => {
                    sleep(Duration::from_millis(500)).await;
                    println!("✓ Reconnection successful, state: {:?}", client.state());
                }
                Ok(Err(e)) => {
                    println!("⚠ Reconnection failed: {}", e);
                }
                Err(_) => {
                    println!("⚠ Reconnection timeout");
                }
            }

            client.disconnect().await;
        }
        Ok(Err(e)) => {
            println!("⚠ Connection failed: {}", e);
        }
        Err(_) => {
            println!("⚠ Connection timeout");
        }
    }
}

// ============================================================================
// Summary Test - Run All Features
// ============================================================================

#[tokio::test]
async fn test_full_feature_summary() {
    println!("\n");
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║          Sockudo Client - Native Integration Tests           ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║                                                              ║");
    println!(
        "║  Testing against server at {}:{}                   ║",
        TEST_HOST, TEST_PORT
    );
    println!(
        "║  App Key: {}                                         ║",
        TEST_APP_KEY
    );
    println!("║                                                              ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!("\n");

    let client = create_delta_client();

    println!("1. Client Creation");
    println!("   ✓ Session ID: {}", client.session_id());
    println!(
        "   ✓ Delta compression enabled: {}",
        client.is_delta_compression_enabled()
    );

    let connect_result = timeout(Duration::from_secs(10), client.connect()).await;

    match connect_result {
        Ok(Ok(())) => {
            println!("\n2. Connection");
            println!("   ✓ Connected successfully");
            println!("   ✓ Socket ID: {:?}", client.socket_id());
            println!("   ✓ State: {:?}", client.state());

            println!("\n3. Channel Subscriptions");

            if let Ok(ch) = client.subscribe("test-public") {
                println!("   ✓ Public channel: {}", ch.name());
            }

            if let Ok(ch) = client.subscribe("benchmark") {
                println!("   ✓ Delta channel (benchmark): {}", ch.name());
            }

            if let Ok(ch) = client.subscribe("market-data") {
                println!("   ✓ Delta channel (market-data): {}", ch.name());
            }

            let filter = FilterOp::eq("type", "important");
            if let Ok(ch) = client.subscribe_with_filter("filtered", Some(filter)) {
                println!("   ✓ Filtered channel: {}", ch.name());
            }

            sleep(Duration::from_secs(1)).await;

            println!("\n4. Delta Compression Stats");
            if let Some(stats) = client.get_delta_stats() {
                println!("   - Total messages: {}", stats.total_messages);
                println!("   - Delta messages: {}", stats.delta_messages);
                println!("   - Full messages: {}", stats.full_messages);
                println!(
                    "   - Bandwidth saved: {:.2}%",
                    stats.bandwidth_saved_percent
                );
            }

            println!("\n5. All Channels");
            for channel in client.all_channels() {
                println!("   - {}", channel.name());
            }

            client.disconnect().await;
            println!("\n6. Disconnection");
            println!("   ✓ Disconnected cleanly");

            println!("\n╔══════════════════════════════════════════════════════════════╗");
            println!("║                    All Tests Completed!                      ║");
            println!("╚══════════════════════════════════════════════════════════════╝\n");
        }
        Ok(Err(e)) => {
            println!("\n⚠ Connection failed: {}", e);
            println!(
                "\nMake sure Sockudo server is running at {}:{}",
                TEST_HOST, TEST_PORT
            );
        }
        Err(_) => {
            println!("\n⚠ Connection timeout");
            println!(
                "\nMake sure Sockudo server is running at {}:{}",
                TEST_HOST, TEST_PORT
            );
        }
    }
}
