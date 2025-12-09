/**
 * Delta Compression Client Library for Sockudo
 *
 * This library provides client-side delta compression support for WebSocket messages.
 * It supports both Fossil Delta and Xdelta3 (VCDIFF) algorithms.
 *
 * ## Features
 * - Automatic delta decoding and message reconstruction
 * - Support for multiple algorithms (fossil, xdelta3)
 * - Per-channel base message tracking
 * - Sequence number validation and error recovery
 * - Bandwidth savings statistics
 *
 * ## Usage
 *
 * ```javascript
 * const pusher = new Pusher(appKey, options);
 * const deltaManager = new DeltaCompressionManager(pusher, {
 *   algorithms: ['fossil', 'xdelta3'],
 *   onStats: (stats) => console.log('Bandwidth saved:', stats.bandwidthSavedPercent)
 * });
 *
 * // Delta compression is automatically enabled
 * const channel = pusher.subscribe('my-channel');
 * channel.bind('my-event', (data) => {
 *   // Data is automatically reconstructed from deltas
 *   console.log(data);
 * });
 * ```
 */

(function (global) {
  "use strict";

  // ============================================================================
  // Utility Functions
  // ============================================================================

  /**
   * Base64 decode a string to Uint8Array
   */
  function base64ToBytes(base64) {
    const binaryString = atob(base64);
    const bytes = new Uint8Array(binaryString.length);
    for (let i = 0; i < binaryString.length; i++) {
      bytes[i] = binaryString.charCodeAt(i);
    }
    return bytes;
  }

  /**
   * Convert Uint8Array to string
   */
  function bytesToString(bytes) {
    return new TextDecoder().decode(bytes);
  }

  /**
   * Convert string to Uint8Array
   */
  function stringToBytes(str) {
    return new TextEncoder().encode(str);
  }

  // ============================================================================
  // Delta Algorithms
  // ============================================================================

  /**
   * Fossil Delta decoder
   */
  class FossilDeltaDecoder {
    static isAvailable() {
      return typeof fossilDelta !== "undefined" && fossilDelta.apply;
    }

    static apply(base, delta) {
      if (!this.isAvailable()) {
        throw new Error("Fossil Delta library not loaded");
      }

      try {
        // fossilDelta.apply expects Uint8Array inputs
        const baseBytes = typeof base === "string" ? stringToBytes(base) : base;
        const deltaBytes =
          typeof delta === "string" ? stringToBytes(delta) : delta;

        const result = fossilDelta.apply(baseBytes, deltaBytes);
        return bytesToString(result);
      } catch (error) {
        throw new Error(`Fossil delta decode failed: ${error.message}`);
      }
    }
  }

  /**
   * Xdelta3 (VCDIFF) decoder
   */
  class Xdelta3Decoder {
    static isAvailable() {
      // Check for vcdiff-decoder library
      return typeof VCDiffDecoder !== "undefined";
    }

    static apply(base, delta) {
      if (!this.isAvailable()) {
        throw new Error("Xdelta3/VCDIFF library not loaded");
      }

      try {
        const baseBytes = typeof base === "string" ? stringToBytes(base) : base;
        const deltaBytes =
          typeof delta === "string" ? stringToBytes(delta) : delta;

        const decoder = new VCDiffDecoder();
        const result = decoder.decode(baseBytes, deltaBytes);
        return bytesToString(result);
      } catch (error) {
        throw new Error(`Xdelta3 decode failed: ${error.message}`);
      }
    }
  }

  // ============================================================================
  // Channel State
  // ============================================================================

  /**
   * Per-channel delta state tracking with conflation key support
   */
  class ChannelState {
    constructor(channelName) {
      this.channelName = channelName;
      this.conflationKey = null; // e.g., "asset", "device_id"
      this.maxMessagesPerKey = 10;

      // Conflation key caches: Map<conflationKeyValue, Array<CachedMessage>>
      // e.g., { "BTC": [{content: {...}, seq: 1}, ...], "ETH": [...] }
      this.conflationCaches = new Map();

      // Legacy single-base tracking (for non-conflation channels)
      this.baseMessage = null;
      this.baseSequence = null;
      this.lastSequence = null;

      // Statistics
      this.deltaCount = 0;
      this.fullMessageCount = 0;
    }

    /**
     * Initialize cache from server sync
     */
    initializeFromCacheSync(data) {
      this.conflationKey = data.conflation_key || null;
      this.maxMessagesPerKey = data.max_messages_per_key || 10;
      this.conflationCaches.clear();

      // Load all conflation group caches
      if (data.states) {
        for (const [key, messages] of Object.entries(data.states)) {
          const cache = messages.map((msg) => ({
            content: msg.content,
            sequence: msg.seq,
          }));
          this.conflationCaches.set(key, cache);
        }
      }
    }

    /**
     * Set new base message (legacy - for non-conflation channels)
     */
    setBase(message, sequence) {
      this.baseMessage = message;
      this.baseSequence = sequence;
      this.lastSequence = sequence;
    }

    /**
     * Get base message for a conflation key at specific index
     */
    getBaseMessage(conflationKeyValue, baseIndex) {
      if (!this.conflationKey) {
        // Legacy mode: return single base
        return this.baseMessage;
      }

      const key = conflationKeyValue || "";
      const cache = this.conflationCaches.get(key);

      if (!cache || baseIndex >= cache.length) {
        return null;
      }

      return cache[baseIndex].content;
    }

    /**
     * Add or update message in conflation cache
     */
    updateConflationCache(conflationKeyValue, message, sequence) {
      const key = conflationKeyValue || "";
      let cache = this.conflationCaches.get(key);

      if (!cache) {
        cache = [];
        this.conflationCaches.set(key, cache);
      }

      // Add message to cache
      cache.push({ content: message, sequence });

      // Enforce max size (FIFO eviction)
      while (cache.length > this.maxMessagesPerKey) {
        cache.shift();
      }
    }

    /**
     * Check if we have a valid base
     */
    hasBase() {
      if (this.conflationKey) {
        return this.conflationCaches.size > 0;
      }
      return this.baseMessage !== null && this.baseSequence !== null;
    }

    /**
     * Validate sequence number
     */
    isValidSequence(sequence) {
      if (this.lastSequence === null) {
        return true; // First message
      }
      return sequence > this.lastSequence;
    }

    /**
     * Update sequence after processing a message
     */
    updateSequence(sequence) {
      this.lastSequence = sequence;
    }

    /**
     * Record delta received
     */
    recordDelta() {
      this.deltaCount++;
    }

    /**
     * Record full message received
     */
    recordFullMessage() {
      this.fullMessageCount++;
    }

    /**
     * Get statistics
     */
    getStats() {
      return {
        channelName: this.channelName,
        conflationKey: this.conflationKey,
        conflationGroupCount: this.conflationCaches.size,
        deltaCount: this.deltaCount,
        fullMessageCount: this.fullMessageCount,
        totalMessages: this.deltaCount + this.fullMessageCount,
      };
    }
  }

  // ============================================================================
  // Delta Compression Manager
  // ============================================================================

  /**
   * Main delta compression manager
   */
  class DeltaCompressionManager {
    constructor(pusher, options = {}) {
      this.pusher = pusher;
      this.options = {
        algorithms: options.algorithms || ["fossil", "xdelta3"],
        autoEnable: options.autoEnable !== false,
        onStats: options.onStats || null,
        onError: options.onError || null,
        debug: options.debug || false,
      };

      // State
      this.enabled = false;
      this.channelStates = new Map();
      this.stats = {
        totalMessages: 0,
        deltaMessages: 0,
        fullMessages: 0,
        totalBytesWithoutCompression: 0,
        totalBytesWithCompression: 0,
        errors: 0,
      };

      // Detect available algorithms
      this.availableAlgorithms = this._detectAvailableAlgorithms();

      if (this.availableAlgorithms.length === 0) {
        console.warn(
          "[DeltaCompression] No delta algorithms available. Please include fossil-delta or vcdiff-decoder libraries.",
        );
        return;
      }

      // Initialize
      if (this.options.autoEnable) {
        this._initialize();
      }
    }

    /**
     * Detect which algorithm libraries are loaded
     */
    _detectAvailableAlgorithms() {
      const available = [];

      if (FossilDeltaDecoder.isAvailable()) {
        available.push("fossil");
        this._log("Fossil Delta decoder available");
      }

      if (Xdelta3Decoder.isAvailable()) {
        available.push("xdelta3");
        this._log("Xdelta3 decoder available");
      }

      return available;
    }

    /**
     * Initialize delta compression
     */
    _initialize() {
      // Wait for connection
      if (this.pusher.connection.state === "connected") {
        this._enableDeltaCompression();
      } else {
        this.pusher.connection.bind("connected", () => {
          this._enableDeltaCompression();
        });
      }

      // Bind to global events for delta messages
      this._bindDeltaEvents();
    }

    /**
     * Send enable request to server
     */
    _enableDeltaCompression() {
      if (this.enabled) {
        return;
      }

      // Filter to only algorithms we support AND server supports
      const supportedAlgorithms = this.availableAlgorithms.filter((algo) =>
        this.options.algorithms.includes(algo),
      );

      if (supportedAlgorithms.length === 0) {
        this._log("No mutually supported algorithms");
        return;
      }

      // Send enable request
      const enableMessage = {
        event: "pusher:enable_delta_compression",
        data: JSON.stringify({
          algorithms: supportedAlgorithms,
        }),
      };

      this._log("Sending enable request", supportedAlgorithms);

      // Access internal pusher connection to send message
      if (this.pusher.connection.socket) {
        this.pusher.connection.socket.send(JSON.stringify(enableMessage));
      }
    }

    /**
     * Bind to delta-related events
     */
    _bindDeltaEvents() {
      // Listen for enable confirmation
      this.pusher.connection.bind(
        "pusher:delta_compression_enabled",
        (data) => {
          this.enabled = data.enabled;
          this._log("Delta compression enabled", data);
        },
      );

      // Listen for cache sync (conflation keys)
      this.pusher.connection.bind("pusher:delta_cache_sync", (data) => {
        this._handleCacheSync(data);
      });

      // Listen for delta messages on all channels
      // We need to intercept the raw WebSocket messages to get channel context
      let retryCount = 0;
      const setupInterceptor = () => {
        // Debug: log what's available on first try
        if (retryCount === 0 && this.pusher.connection) {
          this._log(
            "Connection object keys:",
            Object.keys(this.pusher.connection),
          );
        }

        // Check for socket - it might be under connection.socket or connection.socket_
        const socket =
          this.pusher.connection?.socket || this.pusher.connection?.socket_;

        if (!socket) {
          retryCount++;
          if (retryCount <= 3) {
            this._log(
              `setupInterceptor: No socket available yet, retry ${retryCount}/3`,
            );
            setTimeout(() => setupInterceptor(), 100);
          } else {
            this._log(
              "setupInterceptor: Giving up on WebSocket interceptor, using bind_global fallback",
            );
            this._setupFallbackBinding();
          }
          return;
        }

        const originalOnMessage = socket.onmessage;

        this._log("WebSocket interceptor installed successfully!");

        socket.onmessage = (event) => {
          // IMPORTANT: Call original handler FIRST so Pusher processes the message
          if (originalOnMessage) {
            originalOnMessage.call(socket, event);
          }

          // Then track for our bandwidth stats
          try {
            const message = JSON.parse(event.data);

            // Track delta messages for bandwidth stats
            if (
              message.event === "pusher:delta" &&
              message.channel &&
              message.data
            ) {
              const parsedData =
                typeof message.data === "string"
                  ? JSON.parse(message.data)
                  : message.data;
              this._log("Intercepted pusher:delta message", {
                channel: message.channel,
                seq: parsedData.seq,
              });
              this._trackDeltaMessage(message.channel, parsedData);

              // Also handle delta decompression
              this._handleDeltaMessage(message);
            }

            // Track regular messages with delta sequence for bandwidth stats
            if (
              message.channel &&
              message.data &&
              message.event !== "pusher:delta" &&
              !message.event.startsWith("pusher:")
            ) {
              const parsedData =
                typeof message.data === "string"
                  ? JSON.parse(message.data)
                  : message.data;
              if (
                parsedData &&
                typeof parsedData === "object" &&
                "__delta_seq" in parsedData
              ) {
                this._log("Intercepted full message with __delta_seq", {
                  channel: message.channel,
                  seq: parsedData.__delta_seq,
                });
                this._trackFullMessage(message.channel, parsedData);
                this._handleRegularMessage(
                  message.channel,
                  message.event,
                  parsedData,
                );
              }
            }
          } catch (e) {
            this._log("Error in interceptor", e);
          }
        };
      };

      // Setup interceptor now or wait for connection
      if (this.pusher.connection.state === "connected") {
        this._log("Connection already established, setting up interceptor");
        setupInterceptor();
      } else {
        this._log("Waiting for connection before setting up interceptor");
        this.pusher.connection.bind("connected", () => {
          this._log("Connection established, setting up interceptor");
          setupInterceptor();
        });
      }
    }

    /**
     * Setup fallback binding using bind_global (when WebSocket interception fails)
     */
    _setupFallbackBinding() {
      this._log("Setting up fallback event binding");

      // Store reference to all subscribed channels to track full messages
      const subscribedChannels = new Set();

      // First, bind to any already-subscribed channels
      const allChannels = this.pusher.allChannels();
      this._log(
        `Found ${allChannels.length} existing channels, binding to them`,
      );
      allChannels.forEach((existingChannel) => {
        subscribedChannels.add(existingChannel.name);
        existingChannel.bind_global((eventName, data) => {
          try {
            if (
              eventName.startsWith("pusher:") ||
              eventName.startsWith("pusher_internal:")
            ) {
              return;
            }

            if (data && typeof data === "object" && "__delta_seq" in data) {
              const messageJson = JSON.stringify(data);
              this.stats.totalMessages++;
              this.stats.fullMessages++;
              this.stats.totalBytesWithoutCompression += messageJson.length;
              this.stats.totalBytesWithCompression += messageJson.length;

              if (data.__conflation_key !== undefined) {
                this.stats.uniqueKeys.add(data.__conflation_key);
              }

              this._log("Tracked full message via fallback (existing)", {
                channel: existingChannel.name,
                size: messageJson.length,
                seq: data.__delta_seq,
              });
              this._updateStats();

              // IMPORTANT: Also initialize channel state for delta decompression
              this._handleRegularMessage(existingChannel.name, eventName, data);
            }
          } catch (e) {
            this._log("Error tracking full message on existing channel", e);
          }
        });
      });

      // Wrap subscribe to track new channels
      const originalSubscribe = this.pusher.subscribe.bind(this.pusher);
      this.pusher.subscribe = (channelName) => {
        subscribedChannels.add(channelName);
        const channel = originalSubscribe(channelName);

        // Bind to all events on this channel to track full messages
        channel.bind_global((eventName, data) => {
          try {
            // Skip Pusher protocol events
            if (
              eventName.startsWith("pusher:") ||
              eventName.startsWith("pusher_internal:")
            ) {
              return;
            }

            // Track full messages with delta sequence
            if (data && typeof data === "object" && "__delta_seq" in data) {
              const messageJson = JSON.stringify(data);
              this.stats.totalMessages++;
              this.stats.fullMessages++;
              this.stats.totalBytesWithoutCompression += messageJson.length;
              this.stats.totalBytesWithCompression += messageJson.length;

              if (data.__conflation_key !== undefined) {
                this.stats.uniqueKeys.add(data.__conflation_key);
              }

              this._log("Tracked full message via fallback", {
                channel: channelName,
                size: messageJson.length,
                seq: data.__delta_seq,
              });
              this._updateStats();

              // IMPORTANT: Also initialize channel state for delta decompression
              this._handleRegularMessage(channelName, eventName, data);
            }
          } catch (e) {
            this._log("Error tracking full message", e);
          }
        });

        return channel;
      };

      // Bind to global pusher:delta events for delta tracking AND decompression
      this.pusher.bind_global((eventName, data) => {
        try {
          if (eventName === "pusher:delta") {
            // Track delta message stats
            if (data && typeof data === "object") {
              const parsedData =
                typeof data === "string" ? JSON.parse(data) : data;
              if (parsedData.delta) {
                const deltaBytes = base64ToBytes(parsedData.delta);

                this.stats.totalMessages++;
                this.stats.deltaMessages++;
                this.stats.totalBytesWithCompression += deltaBytes.length;

                // Estimate original size - typical price update is ~200 bytes
                // We can't decompress without base messages (server isn't sending __delta_seq in full messages)
                const estimatedOriginalSize = 200;
                this.stats.totalBytesWithoutCompression +=
                  estimatedOriginalSize;

                this._log("Tracked delta via fallback", {
                  deltaSize: deltaBytes.length,
                  estimatedOriginal: estimatedOriginalSize,
                  seq: parsedData.seq,
                });

                this._updateStats();
              }
            }
          }
        } catch (e) {
          this._log("Error in fallback binding", e);
        }

        return true; // Allow event to propagate
      });
    }

    /**
     * Handle cache sync message (conflation keys)
     */
    _handleCacheSync(rawData) {
      try {
        // Parse if string
        const data =
          typeof rawData === "string" ? JSON.parse(rawData) : rawData;
        const parsedData =
          typeof data.data === "string" ? JSON.parse(data.data) : data.data;
        const channel = data.channel;

        this._log("Received cache sync", {
          channel,
          conflationKey: parsedData.conflation_key,
          groupCount: Object.keys(parsedData.states || {}).length,
        });

        // Get or create channel state
        let channelState = this.channelStates.get(channel);
        if (!channelState) {
          channelState = new ChannelState(channel);
          this.channelStates.set(channel, channelState);
        }

        // Initialize from cache sync
        channelState.initializeFromCacheSync(parsedData);

        this._log("Cache initialized", channelState.getStats());
      } catch (error) {
        this._error("Failed to handle cache sync", error);
      }
    }

    /**
     * Handle delta-compressed message
     */
    _handleDeltaMessage(message) {
      try {
        // message is the full WebSocket message envelope: {event, channel, data}
        const channel = message.channel;
        const parsedData =
          typeof message.data === "string"
            ? JSON.parse(message.data)
            : message.data;

        const event = parsedData.event;
        const delta = parsedData.delta;
        const sequence = parsedData.seq;
        const algorithm = parsedData.algorithm || "fossil";
        const conflationKey = parsedData.conflation_key;
        const baseIndex = parsedData.base_index;

        this._log("Received delta message", {
          channel,
          event,
          sequence,
          algorithm,
          conflationKey,
          baseIndex,
          deltaSize: delta.length,
        });

        // Get channel state
        let channelState = this.channelStates.get(channel);
        if (!channelState) {
          this._error(`No channel state for ${channel}`);
          this._requestResync(channel);
          return false;
        }

        // Get base message
        let baseMessage;
        if (channelState.conflationKey) {
          // Conflation mode: get specific base by key and index
          baseMessage = channelState.getBaseMessage(conflationKey, baseIndex);
          if (!baseMessage) {
            this._error(
              `No base message for channel ${channel}, key ${conflationKey}, index ${baseIndex}`,
            );
            this._requestResync(channel);
            return false;
          }
        } else {
          // Legacy mode: single base message
          baseMessage = channelState.baseMessage;
          if (!baseMessage) {
            this._error(`No base message for channel ${channel}`);
            this._requestResync(channel);
            return false;
          }
        }

        // Decode base64 delta
        const deltaBytes = base64ToBytes(delta);

        // Apply delta based on algorithm
        let reconstructedMessage;
        if (algorithm === "fossil") {
          reconstructedMessage = FossilDeltaDecoder.apply(
            baseMessage,
            deltaBytes,
          );
        } else if (algorithm === "xdelta3") {
          reconstructedMessage = Xdelta3Decoder.apply(baseMessage, deltaBytes);
        } else {
          throw new Error(`Unknown algorithm: ${algorithm}`);
        }

        // Update conflation cache with reconstructed message
        if (channelState.conflationKey) {
          channelState.updateConflationCache(
            conflationKey,
            reconstructedMessage,
            sequence,
          );
        }

        // Update state
        channelState.updateSequence(sequence);
        channelState.recordDelta();

        // Note: Bandwidth stats are tracked in _trackDeltaMessage, but we need to add
        // the reconstructed message size here since we didn't know it before decompression
        this.stats.totalBytesWithoutCompression += reconstructedMessage.length;
        this._updateStats();

        // Emit the reconstructed event
        const pusherChannel = this.pusher.channel(channel);
        if (pusherChannel) {
          const parsedMessageData = JSON.parse(reconstructedMessage);
          pusherChannel.emit(event, parsedMessageData);
        }

        this._log("Delta applied successfully", {
          channel,
          event,
          conflationKey,
          originalSize: reconstructedMessage.length,
          deltaSize: deltaBytes.length,
          compressionRatio:
            ((deltaBytes.length / reconstructedMessage.length) * 100).toFixed(
              1,
            ) + "%",
        });

        return false; // Prevent original event from propagating
      } catch (error) {
        this._error("Delta decode failed", error);
        this.stats.errors++;
        return false;
      }
    }

    /**
     * Track delta message for bandwidth stats
     */
    _trackDeltaMessage(channelName, deltaData) {
      try {
        const deltaBytes = base64ToBytes(deltaData.delta || "");

        // Update stats - delta message received
        this.stats.totalMessages++;
        this.stats.deltaMessages++;
        this.stats.totalBytesWithCompression += deltaBytes.length;

        this._log("Tracked delta message", {
          channel: channelName,
          deltaSize: deltaBytes.length,
          seq: deltaData.seq,
        });

        this._updateStats();
      } catch (e) {
        this._log("Error tracking delta message", e);
      }
    }

    /**
     * Track full message for bandwidth stats
     */
    _trackFullMessage(channelName, data) {
      try {
        const messageJson = JSON.stringify(data);
        const messageSize = messageJson.length;

        // Update stats - full message received
        this.stats.totalMessages++;
        this.stats.fullMessages++;
        this.stats.totalBytesWithoutCompression += messageSize;
        this.stats.totalBytesWithCompression += messageSize;

        // Track conflation key
        if (data.__conflation_key !== undefined) {
          this.stats.uniqueKeys.add(data.__conflation_key);
        }

        this._log("Tracked full message", {
          channel: channelName,
          size: messageSize,
          seq: data.__delta_seq,
          conflationKey: data.__conflation_key,
        });

        this._updateStats();
      } catch (e) {
        this._log("Error tracking full message", e);
      }
    }

    /**
     * Handle regular (full) message
     */
    _handleRegularMessage(channelName, eventName, data) {
      // Check if this message contains delta sequence markers
      if (data && typeof data === "object" && "__delta_seq" in data) {
        const sequence = data.__delta_seq;
        const conflationKey = data.__conflation_key;

        // Find which channel this message belongs to by checking all subscribed channels
        let targetChannelName = null;
        for (const channelName of this.channelStates.keys()) {
          const channel = this.pusher.channel(channelName);
          if (channel && channel.subscribed) {
            targetChannelName = channelName;
            break;
          }
        }

        // If no existing channel state, assume it's for the first subscribed channel
        if (!targetChannelName) {
          const allChannels = this.pusher.allChannels();
          if (allChannels.length > 0) {
            targetChannelName = allChannels[0].name;
          }
        }

        if (!targetChannelName) {
          this._log("Cannot determine channel for message", {
            eventName,
            sequence,
          });
          return true;
        }

        // This is a full message with delta tracking
        const messageJson = JSON.stringify(data);

        let channelState = this.channelStates.get(targetChannelName);
        if (!channelState) {
          channelState = new ChannelState(targetChannelName);
          this.channelStates.set(targetChannelName, channelState);
        }

        // Update cache
        if (channelState.conflationKey && conflationKey !== undefined) {
          // Conflation mode: update specific conflation group cache
          channelState.updateConflationCache(
            conflationKey,
            messageJson,
            sequence,
          );

          this._log("Stored full message (conflation)", {
            channel: targetChannelName,
            conflationKey,
            sequence,
            size: messageJson.length,
          });
        } else {
          // Legacy mode: update single base
          channelState.setBase(messageJson, sequence);

          this._log("Stored full message", {
            channel: targetChannelName,
            sequence,
            size: messageJson.length,
          });
        }

        channelState.recordFullMessage();

        // Update stats
        this.stats.totalMessages++;
        this.stats.fullMessages++;
        this.stats.totalBytesWithoutCompression += messageJson.length;
        this.stats.totalBytesWithCompression += messageJson.length;
        this._updateStats();
      }

      return true; // Allow event to propagate normally
    }

    /**
     * Request resync for a channel
     */
    _requestResync(channel) {
      const resyncMessage = {
        event: "pusher:delta_sync_error",
        data: JSON.stringify({
          channel,
        }),
      };

      this._log("Requesting resync for channel", channel);

      if (this.pusher.connection.socket) {
        this.pusher.connection.socket.send(JSON.stringify(resyncMessage));
      }

      // Clear channel state
      this.channelStates.delete(channel);
    }

    /**
     * Update and emit stats
     */
    _updateStats() {
      if (this.options.onStats) {
        const bandwidthSaved =
          this.stats.totalBytesWithoutCompression -
          this.stats.totalBytesWithCompression;
        const bandwidthSavedPercent =
          this.stats.totalBytesWithoutCompression > 0
            ? (
                (bandwidthSaved / this.stats.totalBytesWithoutCompression) *
                100
              ).toFixed(1)
            : 0;

        this.options.onStats({
          ...this.stats,
          bandwidthSaved,
          bandwidthSavedPercent: parseFloat(bandwidthSavedPercent),
        });
      }
    }

    /**
     * Get current statistics
     */
    getStats() {
      const bandwidthSaved =
        this.stats.totalBytesWithoutCompression -
        this.stats.totalBytesWithCompression;
      const bandwidthSavedPercent =
        this.stats.totalBytesWithoutCompression > 0
          ? (
              (bandwidthSaved / this.stats.totalBytesWithoutCompression) *
              100
            ).toFixed(1)
          : 0;

      // Get per-channel statistics
      const channelStats = [];
      for (const [channelName, channelState] of this.channelStates) {
        channelStats.push(channelState.getStats());
      }

      return {
        ...this.stats,
        bandwidthSaved,
        bandwidthSavedPercent: parseFloat(bandwidthSavedPercent),
        enabled: this.enabled,
        availableAlgorithms: this.availableAlgorithms,
        channelCount: this.channelStates.size,
        channels: channelStats,
      };
    }

    /**
     * Reset statistics
     */
    resetStats() {
      this.stats = {
        totalMessages: 0,
        deltaMessages: 0,
        fullMessages: 0,
        totalBytesWithoutCompression: 0,
        totalBytesWithCompression: 0,
        errors: 0,
      };
      this._updateStats();
    }

    /**
     * Clear channel state (useful for testing)
     */
    clearChannelState(channel) {
      if (channel) {
        this.channelStates.delete(channel);
      } else {
        this.channelStates.clear();
      }
    }

    /**
     * Log message (if debug enabled)
     */
    _log(...args) {
      if (this.options.debug) {
        console.log("[DeltaCompression]", ...args);
      }
    }

    /**
     * Log error
     */
    _error(...args) {
      console.error("[DeltaCompression]", ...args);
      if (this.options.onError) {
        this.options.onError(args);
      }
    }
  }

  // ============================================================================
  // Export
  // ============================================================================

  // Export to global scope
  global.DeltaCompressionManager = DeltaCompressionManager;

  // Also export utility classes for advanced usage
  global.DeltaCompression = {
    Manager: DeltaCompressionManager,
    FossilDecoder: FossilDeltaDecoder,
    Xdelta3Decoder: Xdelta3Decoder,
  };
})(window);
