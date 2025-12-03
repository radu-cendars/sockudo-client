//! Integration tests for delta compression with real sockudo server
//!
//! These tests require a running sockudo server instance.
//! To run these tests, ensure you have a sockudo server running locally or specify connection details.

use sockudo_client::{DeltaAlgorithm, DeltaOptions, SockudoClient, SockudoOptions};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::sleep;

// Configuration for test server
const TEST_APP_KEY: &str = "app-key";
const TEST_HOST: &str = "localhost";
const TEST_PORT: u16 = 6001;
const TEST_USE_TLS: bool = false;

// Helper to create test client options
fn create_test_options() -> SockudoOptions {
    SockudoOptions::new(TEST_APP_KEY)
        .ws_host(TEST_HOST)
        .ws_port(TEST_PORT)
        .use_tls(TEST_USE_TLS)
}

// ============================================================================
// Basic Connection Tests with Delta Compression
// ============================================================================

#[tokio::test]
#[ignore] // Run with: cargo test --test integration_delta_compression -- --ignored
async fn test_connect_with_delta_compression_enabled() {
    let options = create_test_options().delta_compression(DeltaOptions {
        enabled: true,
        algorithms: vec![DeltaAlgorithm::Fossil, DeltaAlgorithm::Xdelta3],
        debug: true,
        max_messages_per_key: 10,
        on_stats: None,
        on_error: None,
    });

    let client = SockudoClient::new(options.into()).expect("Failed to create client");

    // Connect and wait for connection to be established
    client.connect().await.expect("Failed to connect");
    client
        .wait_for_connection(5)
        .await
        .expect("Failed to establish connection");

    assert!(client.is_connected(), "Client should be connected");
    assert!(client.socket_id().is_some(), "Should have socket ID");

    // Check delta compression is enabled
    if let Some(stats) = client.get_delta_stats() {
        println!("Delta compression stats: {:?}", stats);
        assert_eq!(stats.errors, 0, "Should have no errors");
    }

    client.disconnect().await;
}

#[tokio::test]
#[ignore]
async fn test_connect_with_delta_compression_disabled() {
    let options = create_test_options().delta_compression(DeltaOptions {
        enabled: false,
        algorithms: vec![DeltaAlgorithm::Fossil],
        debug: false,
        max_messages_per_key: 10,
        on_stats: None,
        on_error: None,
    });

    let client = SockudoClient::new(options.into()).expect("Failed to create client");

    // Connect and wait for connection to be established
    client.connect().await.expect("Failed to connect");
    client
        .wait_for_connection(5)
        .await
        .expect("Failed to establish connection");

    assert!(client.is_connected(), "Client should be connected");

    // Delta stats should be None or indicate disabled
    let stats = client.get_delta_stats();
    if let Some(stats) = stats {
        assert_eq!(stats.total_messages, 0, "No messages should be processed");
    }

    client.disconnect().await;
}

#[tokio::test]
#[ignore]
async fn test_connect_with_fossil_only() {
    let options = create_test_options().delta_compression(DeltaOptions {
        enabled: true,
        algorithms: vec![DeltaAlgorithm::Fossil],
        debug: true,
        max_messages_per_key: 10,
        on_stats: None,
        on_error: None,
    });

    let client = SockudoClient::new(options.into()).expect("Failed to create client");

    client.connect().await.expect("Failed to connect");
    client
        .wait_for_connection(5)
        .await
        .expect("Failed to establish connection");

    assert!(client.is_connected(), "Client should be connected");

    client.disconnect().await;
}

#[tokio::test]
#[ignore]
async fn test_connect_with_xdelta3_only() {
    let options = create_test_options().delta_compression(DeltaOptions {
        enabled: true,
        algorithms: vec![DeltaAlgorithm::Xdelta3],
        debug: true,
        max_messages_per_key: 10,
        on_stats: None,
        on_error: None,
    });

    let client = SockudoClient::new(options.into()).expect("Failed to create client");

    client.connect().await.expect("Failed to connect");
    client
        .wait_for_connection(5)
        .await
        .expect("Failed to establish connection");

    assert!(client.is_connected(), "Client should be connected");

    client.disconnect().await;
}

// ============================================================================
// Subscription and Message Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_subscribe_and_receive_with_delta_compression() {
    let options = create_test_options().delta_compression(DeltaOptions {
        enabled: true,
        algorithms: vec![DeltaAlgorithm::Fossil, DeltaAlgorithm::Xdelta3],
        debug: true,
        max_messages_per_key: 10,
        on_stats: None,
        on_error: None,
    });

    let client = SockudoClient::new(options.into()).expect("Failed to create client");

    sleep(Duration::from_secs(2)).await;

    // Subscribe to a test channel
    let channel = client
        .subscribe("test-delta-channel")
        .expect("Failed to subscribe");

    let received_messages = Arc::new(Mutex::new(Vec::new()));
    let received_clone = received_messages.clone();

    // Bind event handler
    channel.bind("test-event", move |event| {
        println!("Received event: {:?}", event);
        received_clone.lock().unwrap().push(event.data.clone());
    });

    // Wait for potential messages
    sleep(Duration::from_secs(5)).await;

    // Check delta stats
    if let Some(stats) = client.get_delta_stats() {
        println!("After subscription - Delta stats: {:?}", stats);
        assert_eq!(stats.errors, 0, "Should have no delta errors");
    }

    client.disconnect().await;
}

#[tokio::test]
#[ignore]
async fn test_multiple_subscriptions_with_delta_compression() {
    let options = create_test_options().delta_compression(DeltaOptions {
        enabled: true,
        algorithms: vec![DeltaAlgorithm::Fossil, DeltaAlgorithm::Xdelta3],
        debug: true,
        max_messages_per_key: 10,
        on_stats: None,
        on_error: None,
    });

    let client = SockudoClient::new(options.into()).expect("Failed to create client");

    sleep(Duration::from_secs(2)).await;

    // Subscribe to multiple channels
    let channel1 = client
        .subscribe("delta-channel-1")
        .expect("Failed to subscribe to channel 1");
    let channel2 = client
        .subscribe("delta-channel-2")
        .expect("Failed to subscribe to channel 2");
    let channel3 = client
        .subscribe("delta-channel-3")
        .expect("Failed to subscribe to channel 3");

    let msg_count = Arc::new(Mutex::new(0u32));

    // Bind handlers to all channels
    for channel in [&channel1, &channel2, &channel3] {
        let count = msg_count.clone();
        channel.bind("test-event", move |_event| {
            *count.lock().unwrap() += 1;
        });
    }

    // Wait for potential messages
    sleep(Duration::from_secs(5)).await;

    // Check delta stats
    if let Some(stats) = client.get_delta_stats() {
        println!("Multi-channel delta stats: {:?}", stats);
        assert_eq!(stats.errors, 0, "Should have no delta errors");
        println!("Channel count in stats: {}", stats.channel_count);
    }

    client.disconnect().await;
}

// ============================================================================
// Stats Callback Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_stats_callback_invocation() {
    let stats_called = Arc::new(Mutex::new(false));
    let stats_called_clone = stats_called.clone();

    let stats_data = Arc::new(Mutex::new(None));
    let stats_data_clone = stats_data.clone();

    let options = create_test_options().delta_compression(DeltaOptions {
        enabled: true,
        algorithms: vec![DeltaAlgorithm::Fossil, DeltaAlgorithm::Xdelta3],
        debug: true,
        max_messages_per_key: 10,
        on_stats: Some(Arc::new(move |stats| {
            *stats_called_clone.lock().unwrap() = true;
            *stats_data_clone.lock().unwrap() = Some(stats.clone());
            println!("Stats callback invoked: {:?}", stats);
        })),
        on_error: None,
    });

    let client = SockudoClient::new(options.into()).expect("Failed to create client");

    sleep(Duration::from_secs(2)).await;

    // Subscribe to trigger potential delta messages
    let _channel = client
        .subscribe("stats-test-channel")
        .expect("Failed to subscribe");

    // Wait for messages
    sleep(Duration::from_secs(5)).await;

    // Check if stats callback was called
    let was_called = *stats_called.lock().unwrap();
    println!("Stats callback was called: {}", was_called);

    if let Some(ref stats) = *stats_data.lock().unwrap() {
        println!(
            "Captured stats: total_messages={}, delta_messages={}, full_messages={}, errors={}",
            stats.total_messages, stats.delta_messages, stats.full_messages, stats.errors
        );
    }

    client.disconnect().await;
}

#[tokio::test]
#[ignore]
async fn test_error_callback_invocation() {
    let error_called = Arc::new(Mutex::new(false));
    let error_called_clone = error_called.clone();

    let error_messages = Arc::new(Mutex::new(Vec::new()));
    let error_messages_clone = error_messages.clone();

    let options = create_test_options().delta_compression(DeltaOptions {
        enabled: true,
        algorithms: vec![DeltaAlgorithm::Fossil, DeltaAlgorithm::Xdelta3],
        debug: true,
        max_messages_per_key: 10,
        on_stats: None,
        on_error: Some(Arc::new(move |error| {
            *error_called_clone.lock().unwrap() = true;
            error_messages_clone.lock().unwrap().push(error.to_string());
            println!("Error callback invoked: {}", error);
        })),
    });

    let client = SockudoClient::new(options.into()).expect("Failed to create client");

    sleep(Duration::from_secs(2)).await;

    let _channel = client
        .subscribe("error-test-channel")
        .expect("Failed to subscribe");

    // Wait for potential errors
    sleep(Duration::from_secs(5)).await;

    let was_called = *error_called.lock().unwrap();
    let errors = error_messages.lock().unwrap();

    println!("Error callback was called: {}", was_called);
    println!("Error count: {}", errors.len());
    for error in errors.iter() {
        println!("Error: {}", error);
    }

    client.disconnect().await;
}

// ============================================================================
// Delta Message Processing Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_delta_message_decoding() {
    let options = create_test_options().delta_compression(DeltaOptions {
        enabled: true,
        algorithms: vec![DeltaAlgorithm::Fossil, DeltaAlgorithm::Xdelta3],
        debug: true,
        max_messages_per_key: 10,
        on_stats: None,
        on_error: None,
    });

    let client = SockudoClient::new(options.into()).expect("Failed to create client");

    sleep(Duration::from_secs(2)).await;

    let channel = client
        .subscribe("delta-decode-test")
        .expect("Failed to subscribe");

    let decoded_messages = Arc::new(Mutex::new(Vec::new()));
    let decoded_clone = decoded_messages.clone();

    channel.bind("delta-event", move |event| {
        println!("Decoded delta event: {:?}", event);
        decoded_clone.lock().unwrap().push(event.data.clone());
    });

    // Wait for messages
    sleep(Duration::from_secs(10)).await;

    let messages = decoded_messages.lock().unwrap();
    println!("Total decoded messages: {}", messages.len());

    // Check delta stats
    if let Some(stats) = client.get_delta_stats() {
        println!("Delta decoding stats:");
        println!("  Total messages: {}", stats.total_messages);
        println!("  Delta messages: {}", stats.delta_messages);
        println!("  Full messages: {}", stats.full_messages);
        println!("  Bandwidth saved: {:.2}%", stats.bandwidth_saved_percent);
        println!("  Errors: {}", stats.errors);

        assert_eq!(stats.errors, 0, "Should have no decoding errors");

        if stats.delta_messages > 0 {
            println!("✅ Successfully decoded delta messages!");
        }
    }

    client.disconnect().await;
}

#[tokio::test]
#[ignore]
async fn test_bandwidth_savings_calculation() {
    let options = create_test_options().delta_compression(DeltaOptions {
        enabled: true,
        algorithms: vec![DeltaAlgorithm::Fossil, DeltaAlgorithm::Xdelta3],
        debug: true,
        max_messages_per_key: 10,
        on_stats: None,
        on_error: None,
    });

    let client = SockudoClient::new(options.into()).expect("Failed to create client");

    sleep(Duration::from_secs(2)).await;

    let _channel = client
        .subscribe("bandwidth-test")
        .expect("Failed to subscribe");

    // Wait for messages to accumulate
    sleep(Duration::from_secs(10)).await;

    if let Some(stats) = client.get_delta_stats() {
        println!("\n=== Bandwidth Savings Analysis ===");
        println!("Total messages: {}", stats.total_messages);
        println!("Delta messages: {}", stats.delta_messages);
        println!("Full messages: {}", stats.full_messages);
        println!(
            "Total bytes without compression: {}",
            stats.total_bytes_without_compression
        );
        println!(
            "Total bytes with compression: {}",
            stats.total_bytes_with_compression
        );
        println!("Bandwidth saved: {} bytes", stats.bandwidth_saved);
        println!("Bandwidth saved: {:.2}%", stats.bandwidth_saved_percent);

        if stats.total_messages > 0 {
            let delta_ratio = (stats.delta_messages as f64 / stats.total_messages as f64) * 100.0;
            println!("Delta message ratio: {:.2}%", delta_ratio);
        }

        if stats.delta_messages > 0 && stats.bandwidth_saved > 0 {
            println!("✅ Delta compression is working and saving bandwidth!");
        }
    }

    client.disconnect().await;
}

// ============================================================================
// Conflation Key Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_conflation_key_handling() {
    let options = create_test_options().delta_compression(DeltaOptions {
        enabled: true,
        algorithms: vec![DeltaAlgorithm::Fossil, DeltaAlgorithm::Xdelta3],
        debug: true,
        max_messages_per_key: 10,
        on_stats: None,
        on_error: None,
    });

    let client = SockudoClient::new(options.into()).expect("Failed to create client");

    sleep(Duration::from_secs(2)).await;

    // Subscribe to a channel that uses conflation keys (e.g., market data)
    let _channel = client
        .subscribe("market-data")
        .expect("Failed to subscribe");

    // Wait for messages
    sleep(Duration::from_secs(10)).await;

    if let Some(stats) = client.get_delta_stats() {
        println!("\n=== Conflation Key Analysis ===");
        println!("Total channels: {}", stats.channel_count);
        println!("Total delta messages: {}", stats.delta_messages);
        println!("Total full messages: {}", stats.full_messages);
        println!("Bandwidth saved: {:.2}%", stats.bandwidth_saved_percent);

        // Note: Per-channel stats not available through UniFFI interface
        // Only aggregate stats are available
    }

    client.disconnect().await;
}

// ============================================================================
// Cache Size Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_small_cache_size() {
    let options = create_test_options().delta_compression(DeltaOptions {
        enabled: true,
        algorithms: vec![DeltaAlgorithm::Fossil, DeltaAlgorithm::Xdelta3],
        debug: true,
        max_messages_per_key: 2, // Small cache
        on_stats: None,
        on_error: None,
    });

    let client = SockudoClient::new(options.into()).expect("Failed to create client");

    sleep(Duration::from_secs(2)).await;

    let _channel = client
        .subscribe("small-cache-test")
        .expect("Failed to subscribe");

    sleep(Duration::from_secs(10)).await;

    if let Some(stats) = client.get_delta_stats() {
        println!("Small cache test - Stats: {:?}", stats);
        assert_eq!(stats.errors, 0, "Small cache should work without errors");
    }

    client.disconnect().await;
}

#[tokio::test]
#[ignore]
async fn test_large_cache_size() {
    let options = create_test_options().delta_compression(DeltaOptions {
        enabled: true,
        algorithms: vec![DeltaAlgorithm::Fossil, DeltaAlgorithm::Xdelta3],
        debug: true,
        max_messages_per_key: 100, // Large cache
        on_stats: None,
        on_error: None,
    });

    let client = SockudoClient::new(options.into()).expect("Failed to create client");

    sleep(Duration::from_secs(2)).await;

    let _channel = client
        .subscribe("large-cache-test")
        .expect("Failed to subscribe");

    sleep(Duration::from_secs(10)).await;

    if let Some(stats) = client.get_delta_stats() {
        println!("Large cache test - Stats: {:?}", stats);
        assert_eq!(stats.errors, 0, "Large cache should work without errors");
    }

    client.disconnect().await;
}

// ============================================================================
// Stats Reset Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_stats_reset() {
    let options = create_test_options().delta_compression(DeltaOptions {
        enabled: true,
        algorithms: vec![DeltaAlgorithm::Fossil, DeltaAlgorithm::Xdelta3],
        debug: true,
        max_messages_per_key: 10,
        on_stats: None,
        on_error: None,
    });

    let client = SockudoClient::new(options.into()).expect("Failed to create client");

    sleep(Duration::from_secs(2)).await;

    let _channel = client.subscribe("reset-test").expect("Failed to subscribe");

    // Wait for some messages
    sleep(Duration::from_secs(5)).await;

    // Get initial stats
    let initial_stats = client.get_delta_stats();
    if let Some(ref stats) = initial_stats {
        println!("Initial stats: {:?}", stats);
    }

    // Reset stats
    client.reset_delta_stats();

    // Get stats after reset
    let reset_stats = client.get_delta_stats();
    if let Some(ref stats) = reset_stats {
        println!("After reset stats: {:?}", stats);
        assert_eq!(stats.total_messages, 0, "Stats should be reset to 0");
        assert_eq!(stats.delta_messages, 0, "Delta messages should be 0");
        assert_eq!(stats.full_messages, 0, "Full messages should be 0");
    }

    client.disconnect().await;
}

// ============================================================================
// Reconnection Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_delta_compression_after_reconnection() {
    let options = create_test_options().delta_compression(DeltaOptions {
        enabled: true,
        algorithms: vec![DeltaAlgorithm::Fossil, DeltaAlgorithm::Xdelta3],
        debug: true,
        max_messages_per_key: 10,
        on_stats: None,
        on_error: None,
    });

    let client = SockudoClient::new(options.into()).expect("Failed to create client");

    // Wait for auto-connection to complete
    client
        .wait_for_connection(5)
        .await
        .expect("Failed to establish connection");
    assert!(client.is_connected(), "Should be connected initially");

    // Disconnect
    client.disconnect().await;
    sleep(Duration::from_secs(1)).await;
    assert!(!client.is_connected(), "Should be disconnected");

    // Reconnect
    client.connect().await.expect("Failed to reconnect");
    client
        .wait_for_connection(5)
        .await
        .expect("Failed to reconnect");
    assert!(client.is_connected(), "Should be reconnected");

    // Check delta compression still works
    if let Some(stats) = client.get_delta_stats() {
        println!("After reconnection - Delta stats: {:?}", stats);
        assert_eq!(stats.errors, 0, "Should have no errors after reconnection");
    }

    client.disconnect().await;
}

// ============================================================================
// Performance and Load Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_high_message_volume() {
    let message_count = Arc::new(Mutex::new(0u32));
    let message_count_clone = message_count.clone();

    let options = create_test_options().delta_compression(DeltaOptions {
        enabled: true,
        algorithms: vec![DeltaAlgorithm::Fossil, DeltaAlgorithm::Xdelta3],
        debug: false, // Disable debug for performance
        max_messages_per_key: 20,
        on_stats: Some(Arc::new(move |stats| {
            if stats.total_messages % 100 == 0 {
                println!(
                    "Processed {} messages, {:.2}% bandwidth saved",
                    stats.total_messages, stats.bandwidth_saved_percent
                );
            }
        })),
        on_error: None,
    });

    let client = SockudoClient::new(options.into()).expect("Failed to create client");

    sleep(Duration::from_secs(2)).await;

    let channel = client
        .subscribe("high-volume-test")
        .expect("Failed to subscribe");

    channel.bind("rapid-event", move |_event| {
        *message_count_clone.lock().unwrap() += 1;
    });

    // Wait for high volume of messages
    sleep(Duration::from_secs(30)).await;

    let total_received = *message_count.lock().unwrap();
    println!("Total messages received: {}", total_received);

    if let Some(stats) = client.get_delta_stats() {
        println!("\n=== High Volume Test Results ===");
        println!("Total messages processed: {}", stats.total_messages);
        println!("Delta messages: {}", stats.delta_messages);
        println!("Full messages: {}", stats.full_messages);
        println!("Bandwidth saved: {:.2}%", stats.bandwidth_saved_percent);
        println!("Errors: {}", stats.errors);

        assert_eq!(stats.errors, 0, "Should handle high volume without errors");
    }

    client.disconnect().await;
}
