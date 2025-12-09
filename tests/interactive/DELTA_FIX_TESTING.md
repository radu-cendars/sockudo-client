# Delta Compression Fix - Testing Guide

## Overview

This document describes the fix for the delta compression sync error issue and how to test it.

## The Problem

When testing delta compression, clients were receiving `pusher:delta_sync_error` messages because the server and client were using **different base messages** for delta computation:

**Server base (used for computing deltas):**
```json
{"event":"price","channel":"market-data","data":{"asset":"BTC"}}
```

**Client base (attempted to use for applying deltas):**
```json
{"event":"price","channel":"market-data","data":{"asset":"BTC"},"sequence":0,"conflation_key":"BTC"}
```

This mismatch caused the fossil delta algorithm to fail when trying to reconstruct messages.

## Root Cause

The Pusher client library's protocol decoder was not extracting the `sequence` and `conflation_key` fields from incoming WebSocket messages. These fields were present in the raw message but weren't being copied to the `PusherEvent` object, so the delta compression manager couldn't properly strip them before storing base messages.

## The Fix

Two files were modified in the Pusher client library (`test/interactive/sockudo-js/src/`):

1. **`core/connection/protocol/protocol.ts`**
   - Added extraction of `sequence` and `conflation_key` fields from incoming messages
   - These fields are now copied to the `PusherEvent` object

2. **`core/connection/protocol/message-types.ts`**
   - Added `sequence?: number` and `conflation_key?: string` to the `PusherEvent` interface
   - This ensures TypeScript compatibility

The fix ensures that when full messages arrive with sequence/conflation metadata, the client can properly strip these fields before storing the base message for delta computation, matching what the server stored.

## Testing Instructions

### 1. Rebuild the Client Library

```bash
cd test/interactive
./rebuild-client.sh
```

This script will:
- Build the Pusher client with the fix
- Copy it to `public/pusher-local.js`

### 2. Start the Sockudo Server

In the main project directory:

```bash
# Make sure delta compression is enabled in your config
cargo run --release
```

### 3. Start the Test Server

In `test/interactive`:

```bash
npm install  # If you haven't already
npm start
```

The test server should start on `http://localhost:3000`

### 4. Open the Test Page

Navigate to: `http://localhost:3000/delta-test-local.html`

### 5. Run the Test

1. **Click "Connect"**
   - Should see: "✅ Connected to Sockudo"
   - Status should show: "Connected" (green)

2. **Click "Enable Delta"**
   - Should see: "✅ Delta compression enabled: {"enabled":true}"

3. **Click "Subscribe to Channel"**
   - Should see: "✅ Subscribed to market-data"
   - "Send Test Message" button should become enabled

4. **Click "Send Test Message" multiple times (5-10 times)**
   - First message: Should arrive as a full message with `sequence: 0`
   - Subsequent messages: Should arrive as delta messages
   - Watch the console for detailed logging

## Expected Results

### ✅ Success Indicators

1. **No sync errors**: You should NOT see any `pusher:delta_sync_error` messages
2. **Delta messages working**: After the first message, subsequent messages should show:
   - Delta Messages counter increasing
   - Bandwidth Saved percentage showing (typically 60-90% for similar messages)
3. **Console shows proper field extraction**:
   ```
   📨 Raw message: {"event":"price","channel":"market-data",...}
      🔢 Sequence: 0
      🔑 Conflation Key: BTC (if configured)
      📝 Raw: {full message...}
   ```

### ❌ Failure Indicators (if fix didn't work)

1. `pusher:delta_sync_error` events appearing
2. Delta Messages counter stays at 0
3. Bandwidth Saved stays at 0%
4. Console shows: "Delta decode failed" or "No base message"

## Debugging

### Enable Detailed Logging

The test page already has verbose logging enabled. Watch the browser console for:

- `📨 Raw message`: Shows every incoming WebSocket message
- `🔢 Sequence`: Confirms sequence numbers are being extracted
- `🔑 Conflation Key`: Shows conflation keys (if configured)
- `📝 Raw`: Shows the raw message before parsing

### Check Server Logs

Enable debug logging on the Sockudo server:

```bash
DEBUG=true cargo run
```

Look for messages about:
- Delta compression state
- Base message storage
- Delta computation

### Common Issues

**Issue**: "Pusher is not defined"
- **Solution**: Make sure you ran `./rebuild-client.sh` to copy the built file

**Issue**: Delta messages not appearing
- **Solution**: Verify delta compression is enabled in Sockudo config:
  ```json
  {
    "delta_compression": {
      "enabled": true,
      "algorithm": "fossil"
    }
  }
  ```

**Issue**: Still getting sync errors
- **Solution**: Check that the server is storing messages WITHOUT sequence/conflation_key fields
- Look at server logs for "STORING base message" entries

## Configuration

### Server Configuration (Sockudo)

Ensure delta compression is enabled in `config/config.json`:

```json
{
  "delta_compression": {
    "enabled": true,
    "algorithm": "fossil",
    "full_message_interval": 10,
    "min_message_size": 100,
    "max_state_age_secs": 300,
    "max_channel_states_per_socket": 100
  }
}
```

### Channel-Specific Settings

For conflation key testing, configure in your channel settings:

```json
{
  "channel_settings": {
    "market-data": {
      "delta_compression": {
        "enabled": true,
        "conflation_key": "asset"
      }
    }
  }
}
```

## Performance Metrics

With the fix working correctly, you should see:

- **Bandwidth savings**: 60-90% for similar consecutive messages
- **CPU overhead**: ~5-20μs per message
- **Memory usage**: ~10-50KB per socket
- **Latency impact**: Negligible (<1ms)

## Additional Testing

### Test with Conflation Keys

Modify the test to send messages with different conflation keys:

```javascript
// Send different assets
await sendMessage({ asset: 'BTC' });
await sendMessage({ asset: 'ETH' });
await sendMessage({ asset: 'BTC' });  // Should delta against first BTC message
```

### Test Full Message Intervals

Send 10+ messages in a row to trigger the full message interval (default: 10):

```javascript
for (let i = 0; i < 15; i++) {
  await sendMessage({ asset: 'BTC', price: 50000 + i });
}
```

You should see every 10th message arrive as a full message instead of a delta.

### Load Testing

Use the existing `conflation-test.html` page for high-volume testing with 100+ conflation keys.

## Verification Checklist

- [ ] Client library rebuilt with fix
- [ ] Test page loads without errors
- [ ] Connection establishes successfully
- [ ] Delta compression enables without errors
- [ ] First message arrives with sequence=0
- [ ] Subsequent messages arrive as deltas (not full messages)
- [ ] NO `pusher:delta_sync_error` events
- [ ] Bandwidth Saved shows >0%
- [ ] Delta Messages counter increases
- [ ] Console shows sequence/conflation_key extraction

## Next Steps

Once verified working:

1. **Document the fix** in the main project README
2. **Update the Pusher client** in production
3. **Consider upstreaming** these changes to the official Pusher JS client (if applicable)
4. **Add automated tests** to prevent regression

## Support

If you encounter issues:

1. Check the browser console for detailed logs
2. Check the Sockudo server logs (with DEBUG=true)
3. Verify both server and client configurations
4. Ensure you're using the rebuilt client (pusher-local.js)
5. Try clearing browser cache and reconnecting

## Summary

This fix ensures that the client properly extracts and handles the `sequence` and `conflation_key` fields from incoming messages, allowing the delta compression system to work correctly by matching the base messages between client and server.