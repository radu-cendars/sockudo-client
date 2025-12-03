//! Example of using presence channels for tracking online users.

use serde_json::json;
use sockudo_client::{utils::wait_for_signal, PusherOptions, SockudoClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    let options = PusherOptions::new("app-key")
        .ws_host("localhost")
        .ws_port(6001)
        .use_tls(false)
        .auth_endpoint("http://localhost:3000/pusher/auth");

    let client = SockudoClient::new(options.into())?;
    println!("✓ Client created");

    // Subscribe to a presence channel BEFORE connecting
    // The subscription will happen automatically when connection is established
    let channel = client.subscribe("presence-chat-room")?;
    println!("✓ Subscribed to presence-chat-room");

    // Bind event handlers BEFORE connecting so they're ready when events arrive

    // Debug: bind to internal subscription event to see if it arrives
    channel.bind("pusher_internal:subscription_succeeded", |event| {
        println!("DEBUG: Received pusher_internal:subscription_succeeded!");
        if let Some(ref data) = event.data {
            println!("  Internal data: {}", data);
        }
    });

    // Handle subscription succeeded - get initial member list
    channel.bind("pusher:subscription_succeeded", |event| {
        println!("Joined room!");
        if let Some(data_ref) = &event.data {
            #[cfg(feature = "wasm")]
            let data = Some(data_ref.clone());
            #[cfg(not(feature = "wasm"))]
            let data = serde_json::from_str::<serde_json::Value>(data_ref).ok();

            if let Some(data) = data {
                if let Some(members) = data.get("members") {
                    println!("Current members: {:?}", members);
                }
                if let Some(count) = data.get("count") {
                    println!("Member count: {}", count);
                }
            }
        }
    });

    // Handle new members joining
    channel.bind("pusher:member_added", |event| {
        if let Some(data_ref) = &event.data {
            #[cfg(feature = "wasm")]
            let data = Some(data_ref.clone());
            #[cfg(not(feature = "wasm"))]
            let data = serde_json::from_str::<serde_json::Value>(data_ref).ok();

            if let Some(data) = data {
                let user_id = data
                    .get("user_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                let user_info = data.get("user_info");
                println!("Member joined: {} ({:?})", user_id, user_info);
            }
        }
    });

    // Handle members leaving
    channel.bind("pusher:member_removed", |event| {
        if let Some(data_ref) = &event.data {
            #[cfg(feature = "wasm")]
            let data = Some(data_ref.clone());
            #[cfg(not(feature = "wasm"))]
            let data = serde_json::from_str::<serde_json::Value>(data_ref).ok();

            if let Some(data) = data {
                let user_id = data
                    .get("user_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                println!("Member left: {}", user_id);
            }
        }
    });

    // Handle chat messages
    channel.bind("message", |event| {
        if let Some(data_ref) = &event.data {
            #[cfg(feature = "wasm")]
            let data = Some(data_ref.clone());
            #[cfg(not(feature = "wasm"))]
            let data = serde_json::from_str::<serde_json::Value>(data_ref).ok();

            if let Some(data) = data {
                let text = data.get("text").and_then(|v| v.as_str()).unwrap_or("");
                let from = event.user_id.as_deref().unwrap_or("unknown");
                println!("[{}]: {}", from, text);
            }
        }
    });

    // Now connect - event handlers are ready
    client.connect().await?;
    println!("✓ Connection started");

    // Wait a bit for connection and subscription to complete
    println!("Waiting for subscription to complete...");
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    // Send a message via client event
    println!("Sending client-typing event...");
    #[cfg(feature = "wasm")]
    channel.trigger(
        "client-typing",
        json!({
            "typing": true
        }),
    )?;
    #[cfg(not(feature = "wasm"))]
    channel.trigger(
        "client-typing",
        json!({
            "typing": true
        })
        .to_string(),
    )?;

    println!("\nConnected to presence channel. Press Ctrl+C to exit.");
    wait_for_signal().await;

    client.disconnect().await;
    Ok(())
}
