#!/usr/bin/env node
/**
 * Sockudo Client - Node.js WASM Example
 *
 * This example uses WebAssembly bindings which work in Node.js, Browser, and Deno.
 * This is the recommended approach over NAPI for cross-platform support.
 */

const { Pusher, JsOptions } = require("../pkg/sockudo_client.js");

console.log("=== Sockudo Client WASM for Node.js ===\n");

async function main() {
  try {
    // Create client options
    const options = new JsOptions("app-key");
    options.cluster = "";
    options.ws_host = "localhost";
    options.ws_port = 6001;
    options.use_tls = false;
    options.enable_delta_compression = true;

    // Create client instance
    console.log("✅ Creating Pusher client...");
    const pusher = new Pusher("app-key", options);

    console.log("Initial state:", pusher.state);
    console.log("Socket ID:", pusher.socket_id);

    // Bind global event handler
    console.log("\n✅ Setting up global event handler...");
    pusher.bind("test-event", (event) => {
      console.log("Global event received:", event);
    });

    // Subscribe to a channel
    console.log("\n✅ Subscribing to channel...");
    const channel = pusher.subscribe("my-channel");

    console.log("Channel name:", channel.name);
    console.log("Is subscribed:", channel.subscribed);

    // Bind event handler to channel
    console.log("\n✅ Binding event handler to channel...");
    channel.bind("my-event", (event) => {
      console.log("Channel event received:", {
        event: event.event,
        channel: event.channel,
        data: event.data,
      });
    });

    // Bind to all events on channel
    channel.bind("another-event", (event) => {
      console.log("Another event:", event.event);
    });

    // Connect to server (will fail without actual server)
    console.log("\n✅ Attempting to connect...");
    try {
      await pusher.connect();
      console.log("Connected! State:", pusher.state);
      console.log("Socket ID:", pusher.socket_id);

      // Wait a bit for potential events
      console.log("\nWaiting for events (5 seconds)...");
      await new Promise((resolve) => setTimeout(resolve, 5000));

      // Try to trigger a client event (only works on private/presence channels)
      if (
        channel.name.startsWith("private-") ||
        channel.name.startsWith("presence-")
      ) {
        console.log("\nTriggering client event...");
        channel.trigger("client-my-event", { message: "Hello from WASM!" });
      }
    } catch (error) {
      console.log("Connection error (expected without server):", error.message);
    }

    // Unsubscribe from channel
    console.log("\n✅ Unsubscribing from channel...");
    pusher.unsubscribe("my-channel");

    // Disconnect
    console.log("\n✅ Disconnecting...");
    pusher.disconnect();
    console.log("Final state:", pusher.state);

    // Get delta stats
    const stats = pusher.get_delta_stats();
    if (stats) {
      console.log("\n✅ Delta compression stats:", stats);
    }
  } catch (error) {
    console.error("\n❌ Error:", error.message);
    console.error(error.stack);
    process.exit(1);
  }
}

// Example: Using private channels
async function privateChannelExample() {
  const pusher = new Pusher("your-app-key");

  const options = new JsOptions("your-app-key");
  options.auth_endpoint = "/pusher/auth";

  await pusher.connect();

  const privateChannel = pusher.subscribe("private-user-123");

  privateChannel.bind("notification", (event) => {
    console.log("Private notification:", event);
  });

  // Trigger client event
  privateChannel.trigger("client-typing", { userId: 123 });
}

// Example: Using presence channels
async function presenceChannelExample() {
  const options = new JsOptions("your-app-key");
  options.cluster = "mt1";

  const pusher = new Pusher("your-app-key", options);
  await pusher.connect();

  const presenceChannel = pusher.subscribe("presence-room-123");

  // Note: Members functionality would need to be exposed from Rust
  presenceChannel.bind("pusher:member_added", (event) => {
    console.log("Member joined:", event);
  });

  presenceChannel.bind("pusher:member_removed", (event) => {
    console.log("Member left:", event);
  });
}

console.log("=== WASM Benefits ===");
console.log("✅ Single build works on all platforms (Node.js, Browser, Deno)");
console.log("✅ Full event callback support");
console.log("✅ No Send/Sync issues");
console.log("✅ Smaller binary size");
console.log("✅ Same code runs everywhere\n");

// Run the example
if (require.main === module) {
  main().catch(console.error);
}

module.exports = {
  main,
  privateChannelExample,
  presenceChannelExample,
};
