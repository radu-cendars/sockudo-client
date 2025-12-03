//! Example demonstrating cross-platform signal handling.
//!
//! This example shows how to use the signal handling utilities to gracefully
//! shutdown a Sockudo client when receiving termination signals.
//!
//! Supported signals:
//! - Windows: Ctrl+C, Ctrl+Break
//! - Unix/Linux/macOS: SIGINT (Ctrl+C), SIGTERM, SIGHUP

use serde_json::json;
use sockudo_client::{utils::SignalHandler, PusherOptions, SockudoClient};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging to see signal messages
    tracing_subscriber::fmt::init();

    println!("=== Sockudo Client - Signal Handling Demo ===\n");

    // Configure the client
    let options = PusherOptions::new("app-key")
        .ws_host("localhost")
        .ws_port(6001)
        .use_tls(false)
        .auth_endpoint("http://localhost:3000/pusher/auth")
        .debug(true);

    // Create the client
    let client = SockudoClient::new(options.into())?;
    println!("✓ Created client with session ID: {}", client.session_id());

    // Connect to the server
    client.connect().await?;
    println!("✓ Connected! Socket ID: {:?}\n", client.socket_id());

    // Subscribe to a private channel (required for client events)
    let channel = client.subscribe("private-demo-channel")?;

    // Track received messages
    let message_count = Arc::new(AtomicBool::new(false));
    let msg_count_clone = message_count.clone();

    channel.bind("demo-event", move |event| {
        println!("📨 Received event: {:?}", event.data);
        msg_count_clone.store(true, Ordering::SeqCst);
    });

    println!("✓ Subscribed to 'private-demo-channel'\n");

    // Create a signal handler that listens for:
    // - Windows: Ctrl+C and Ctrl+Break
    // - Unix: SIGINT, SIGTERM, and SIGHUP
    let mut signal_handler = SignalHandler::new()?;
    println!("✓ Signal handler initialized");
    println!("  Listening for termination signals:");
    #[cfg(windows)]
    println!("  - Ctrl+C (SIGINT)");
    #[cfg(windows)]
    println!("  - Ctrl+Break");
    #[cfg(unix)]
    println!("  - SIGINT (Ctrl+C)");
    #[cfg(unix)]
    println!("  - SIGTERM");
    #[cfg(unix)]
    println!("  - SIGHUP");
    println!();

    // Spawn a task to send periodic heartbeats
    let heartbeat_channel = channel.clone();
    let heartbeat_task = tokio::spawn(async move {
        let mut counter = 0;
        loop {
            sleep(Duration::from_secs(5)).await;
            counter += 1;

            #[cfg(feature = "wasm")]
            let result = heartbeat_channel.trigger(
                "client-heartbeat",
                json!({
                    "counter": counter,
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                }),
            );
            #[cfg(not(feature = "wasm"))]
            let result = heartbeat_channel.trigger(
                "client-heartbeat",
                json!({
                    "counter": counter,
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                })
                .to_string(),
            );

            if let Err(e) = result {
                eprintln!("Failed to send heartbeat: {}", e);
                break;
            }

            println!("💓 Sent heartbeat #{}", counter);
        }
    });

    println!("🚀 Client is running!");
    println!("   Press Ctrl+C to gracefully shutdown...\n");

    // Wait for signal (this will block until a signal is received)
    signal_handler.wait().await;

    println!("\n🛑 Shutdown signal received!");
    println!("   Cleaning up gracefully...");

    // Cancel background task
    heartbeat_task.abort();

    // Print delta compression stats before shutdown
    if let Some(stats) = client.get_delta_stats() {
        println!(
            "   📊 Final delta compression stats: {} messages, {:.1}% bandwidth saved",
            stats.total_messages, stats.bandwidth_saved_percent
        );
    }

    // Unsubscribe from channels
    println!("   → Unsubscribing from channels...");
    client.unsubscribe("private-demo-channel");

    // Disconnect from the server
    println!("   → Disconnecting from server...");
    client.disconnect().await;

    // Print final statistics
    if message_count.load(Ordering::SeqCst) {
        println!("   → Messages were received during this session");
    }

    println!("\n✅ Shutdown complete. Goodbye!");

    Ok(())
}
