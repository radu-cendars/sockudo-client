# Browser Testing for Sockudo WebSocket Client

This directory contains a comprehensive web-based test interface for the Sockudo WebSocket client.

## Quick Start

### 1. Build the WASM Package

First, make sure you've built the WASM package:

```bash
# On Windows (PowerShell)
.\build-wasm.ps1

# On Linux/Mac
./build-wasm.sh
```

This will create the `pkg/` directory with the compiled WASM files.

### 2. Start a Local Server

**Due to browser CORS restrictions, you cannot open `test.html` directly from the filesystem.** You need to serve it through HTTP. Choose one of the following options:

#### Option A: Node.js Server (Recommended)

```bash
cd examples/browser
node serve.js
```

Then open: http://localhost:8080/test.html

#### Option B: Python Server

```bash
cd examples/browser
python serve.py
```

Then open: http://localhost:8080/test.html

#### Option C: PowerShell Server (Windows)

```powershell
cd examples\browser
.\serve.ps1
```

Then open: http://localhost:8080/test.html

**Note:** On Windows, you may need to run PowerShell as Administrator, or run this command once (as Admin):
```powershell
netsh http add urlacl url=http://+:8080/ user=Everyone
```

#### Option D: Use Any Other HTTP Server

You can use any static file server. Just make sure:
- It serves files from the `examples/browser/` directory
- It can also serve files from `../../pkg/` at the `/pkg/` URL path
- It sets proper MIME types (especially `application/wasm` for `.wasm` files)

Examples with other servers:

```bash
# Using Python's built-in server (Python 3)
python -m http.server 8080

# Using PHP's built-in server
php -S localhost:8080

# Using Node.js 'http-server' package
npx http-server -p 8080 --cors

# Using 'serve' package
npx serve -p 8080 --cors
```

## Test Page Features

The test page (`test.html`) provides a comprehensive interface to test all Sockudo features:

### 🔌 Connection Management
- Configure connection settings (host, port, app key, cluster)
- Enable/disable TLS, debug mode, delta compression
- Set authentication endpoint
- Real-time connection status
- View socket ID

### 📡 Channel Management
- Subscribe to public, private, presence, and encrypted channels
- Unsubscribe from channels
- Visual list of active channels with type badges
- Support for all channel types

### ⚡ Event System
- Bind to custom events on any channel
- Trigger client events (on private/presence channels)
- JSON event data editor with validation
- List of all bound events
- Real-time event monitoring

### 📊 Statistics Dashboard
- Total messages received
- Delta compression statistics
- Bandwidth savings percentage
- Average compression ratio
- Auto-refresh every 5 seconds

### 👥 Presence Channels
- View all members in presence channels
- Real-time member join/leave notifications
- Display member avatars and info
- Track your own user ID
- Member count display

### 📝 Event Log
- Real-time event stream
- Color-coded by type (info, success, error, warning)
- Timestamps for all events
- Auto-scroll option
- Clear log button
- Export log to text file

## Usage Example

1. **Start the server** using one of the methods above
2. **Open the test page** in your browser (http://localhost:8080/test.html)
3. **Configure connection**:
   - Default settings work with a local Pusher server on `localhost:6001`
   - Modify as needed for your setup
4. **Click Connect** to establish WebSocket connection
5. **Subscribe to channels**:
   - Public: `my-channel`
   - Private: `private-my-channel`
   - Presence: `presence-chat-room`
   - Encrypted: `private-encrypted-secrets`
6. **Bind to events** to receive notifications
7. **Trigger client events** (only on private/presence channels)
8. **Monitor the event log** for all activity
9. **Check statistics** to see delta compression in action

## Testing with a Real Server

To test with a Pusher-compatible server:

1. Update the connection settings:
   - **App Key**: Your Pusher app key
   - **Cluster**: Your cluster (e.g., `us2`, `eu`, `ap1`)
   - **Host**: Leave as provided by cluster, or use custom
   - **Port**: Usually `443` for TLS, `80` for non-TLS
   - **Use TLS**: Enable for production servers

2. For private/presence channels, set the **Auth Endpoint** to your server's authorization URL

3. Click **Connect** and start testing!

## Troubleshooting

### CORS Errors
- **Problem**: `Access to script blocked by CORS policy`
- **Solution**: Make sure you're using one of the provided server scripts, not opening the HTML file directly

### WASM Not Loading
- **Problem**: `Failed to load WASM module`
- **Solution**: Ensure you've run `build-wasm.sh` or `build-wasm.ps1` first

### Port Already in Use
- **Problem**: `Port 8080 is already in use`
- **Solution**: Modify the `PORT` variable in the server script, or stop the other application

### Connection Failed
- **Problem**: Cannot connect to WebSocket server
- **Solution**: 
  - Check that your WebSocket server is running
  - Verify host/port settings
  - Check TLS settings match your server
  - Look in browser console for detailed error messages

### Authorization Failed
- **Problem**: Private/presence channels fail to subscribe
- **Solution**:
  - Ensure your auth endpoint is correct
  - Check that your auth endpoint returns proper signatures
  - Look in the event log for detailed error messages

## Browser Compatibility

The test page works in all modern browsers:
- ✅ Chrome/Edge 90+
- ✅ Firefox 88+
- ✅ Safari 14+
- ✅ Opera 76+

WebAssembly and ES6 modules support is required.

## File Structure

```
examples/browser/
├── README.md          # This file
├── test.html          # Comprehensive test interface
├── serve.js           # Node.js HTTP server
├── serve.py           # Python HTTP server
└── serve.ps1          # PowerShell HTTP server
```

## Tips

- **Use the Event Log**: It shows detailed information about all WebSocket activity
- **Export Logs**: Click "Export Log" to save troubleshooting information
- **Enable Debug Mode**: See detailed protocol messages in the event log
- **Auto-scroll**: Keep enabled to always see the latest events
- **Keyboard Shortcut**: Press `Ctrl+Enter` to quickly connect

## Advanced Usage

### Testing Delta Compression

1. Subscribe to a channel
2. Have another client send messages with similar data
3. Watch the statistics panel update
4. See bandwidth savings in real-time

### Testing Presence Channels

1. Subscribe to a `presence-` channel
2. Open the page in multiple tabs/browsers
3. Watch members appear in the member list
4. Close tabs and see members disappear

### Testing Client Events

1. Subscribe to a private or presence channel
2. Select the channel in the Events panel
3. Enter an event name starting with `client-`
4. Add JSON data
5. Click "Trigger Event"
6. See the event in other connected clients

## Contributing

Found a bug or want to add a feature? Please open an issue or submit a pull request!

## License

MIT License - see LICENSE file for details
