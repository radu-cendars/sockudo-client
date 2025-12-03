//! Example usage of the Sockudo client library.

use sockudo_client::{
    utils::wait_for_signal, DeltaOptions, FilterOp, PusherOptions, SockudoClient,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Configure the client
    let options = PusherOptions::new("app-key")
        .ws_host("localhost")
        .ws_port(6001)
        .use_tls(false)
        .delta_compression(DeltaOptions {
            enabled: true,
            ..Default::default()
        })
        .debug(true);

    // Create the client
    let client = SockudoClient::new(options.into())?;

    println!("Created client with session ID: {}", client.session_id());

    // Bind global event handler
    client.bind_global(|event| {
        println!("[Global] Event: {} on {:?}", event.event, event.channel);
    });

    // Connect to the server
    client.connect().await?;
    println!("Connected! Socket ID: {:?}", client.socket_id());

    // Subscribe to a public channel
    let public_channel = client.subscribe("chat-room")?;
    public_channel.bind("message", |event| {
        println!("[Chat] Message: {:?}", event.data);
    });

    // Subscribe to a private channel (requires auth endpoint)
    let private_channel = client.subscribe("private-user-123")?;
    private_channel.bind("notification", |event| {
        println!("[Private] Notification: {:?}", event.data);
    });

    // Subscribe with tag filtering
    let filter = FilterOp::and(vec![
        FilterOp::eq("type", "goal"),
        FilterOp::in_set(
            "league",
            vec!["premier".to_string(), "champions".to_string()],
        ),
    ]);
    let sports_channel = client.subscribe_with_filter("sports-updates", Some(filter))?;
    sports_channel.bind("score-update", |event| {
        println!("[Sports] Score: {:?}", event.data);
    });

    // Check delta compression stats periodically
    if let Some(stats) = client.get_delta_stats() {
        println!(
            "Delta stats: {} messages, {:.1}% bandwidth saved",
            stats.total_messages, stats.bandwidth_saved_percent
        );
    }

    // Keep running (cross-platform signal handling)
    println!("Client running. Press Ctrl+C to exit.");
    wait_for_signal().await;

    // Disconnect
    client.disconnect().await;
    println!("Disconnected.");

    Ok(())
}
