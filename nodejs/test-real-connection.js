#!/usr/bin/env node
/**
 * Test real connection to Sockudo instance
 */

const { Pusher, JsOptions } = require('../pkg/sockudo_client.js');

console.log('=== Testing Real Sockudo Connection ===\n');

async function testConnection() {
    try {
        // Create options for your Sockudo instance
        const options = new JsOptions('app-key');

        // Configure for local Sockudo instance (adjust as needed)
        console.log('Configuration:');
        console.log('  App Key: app-key');
        console.log('  Host: localhost (default)');
        console.log('  Port: 6001 (default)');
        console.log('  TLS: false (default)\n');

        // You can customize these:
        // options.ws_host = 'your-server.com';
        // options.ws_port = 6001;
        // options.use_tls = false;

        // Create client
        const pusher = new Pusher('app-key', options);

        console.log('Client created');
        console.log('Initial state:', pusher.state);

        // Bind to connection events to see what's happening
        console.log('\nAttempting to connect...');

        try {
            await pusher.connect();
            console.log('✅ Connected successfully!');
            console.log('State:', pusher.state);
            console.log('Socket ID:', pusher.socket_id);

            // Subscribe to a test channel
            console.log('\nSubscribing to test-channel...');
            const channel = pusher.subscribe('test-channel');

            console.log('Channel name:', channel.name);
            console.log('Subscribed:', channel.subscribed);

            // Bind event handler
            channel.bind('test-event', (event) => {
                console.log('✅ Event received:', event);
            });

            console.log('\nListening for events (10 seconds)...');
            console.log('Try sending an event from your server now!');

            await new Promise(resolve => setTimeout(resolve, 10000));

            console.log('\nDisconnecting...');
            pusher.disconnect();
            console.log('Disconnected. Final state:', pusher.state);

        } catch (error) {
            console.error('❌ Connection failed:', error.message);
            console.error('Full error:', error);

            console.log('\n=== Troubleshooting ===');
            console.log('1. Is your Sockudo server running?');
            console.log('2. Check the host and port:');
            console.log('   - Default: ws://localhost:6001');
            console.log('3. Verify app-key is correct');
            console.log('4. Check server logs for connection attempts');
        }

    } catch (error) {
        console.error('❌ Error:', error.message);
        console.error(error.stack);
    }
}

testConnection();
