#!/usr/bin/env node
/**
 * Basic test to verify NAPI bindings work
 */

const sockudo = require("./index.node");

console.log("=== Sockudo Client NAPI Test ===\n");

// Test 1: Get version
console.log("✓ Test 1: Get version");
const version = sockudo.getVersion();
console.log("  Version:", version);
console.log();

// Test 2: Initialize logging
console.log("✓ Test 2: Initialize logging");
sockudo.initLogging();
console.log("  Logging initialized");
console.log();

// Test 3: Create client
console.log("✓ Test 3: Create client");
const options = {
  appKey: "app-key",
  cluster: "mt1",
  useTls: false,
};

try {
  const client = new sockudo.SockudoClient(options);
  console.log("  Client created successfully");
  console.log("  Initial state:", client.getState());
  console.log("  Socket ID:", client.getSocketId());
  console.log("  Is connected:", client.isConnected());
  console.log();

  // Test 4: Subscribe to a channel (without connecting)
  console.log("✓ Test 4: Subscribe to channel");
  const channel = client.subscribe("test-channel");
  console.log("  Channel name:", channel.getName());
  console.log("  Is subscribed:", channel.isSubscribed());
  console.log();

  // Test 5: Unsubscribe
  console.log("✓ Test 5: Unsubscribe from channel");
  client.unsubscribe("test-channel");
  console.log("  Channel unsubscribed");
  console.log();

  // Test 6: Test async connect/disconnect (will fail without server, but should not crash)
  console.log("✓ Test 6: Test async connect (will fail without server)");
  (async () => {
    try {
      await client.connect();
      console.log("  Connected!");
      console.log("  State after connect:", client.getState());

      await client.disconnect();
      console.log("  Disconnected!");
      console.log("  State after disconnect:", client.getState());
    } catch (error) {
      console.log("  Expected error (no server):", error.message);
    }
    console.log();

    console.log("=== All tests completed! ===");
  })();
} catch (error) {
  console.error("✗ Error:", error.message);
  console.error(error.stack);
  process.exit(1);
}
