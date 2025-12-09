// Bundle entry point for Bun
// This file imports all dependencies and makes them available globally

import init, {
  WasmSockudo,
  WasmOptions,
  WasmChannel,
} from "./dist/sockudo_client.js";

// For compatibility, alias the Wasm classes
const SockudoClient = WasmSockudo;
const SockudoOptions = WasmOptions;
const Channel = WasmChannel;
import * as fossilDelta from "fossil-delta";
import { decode as vcdiffDecode } from "@ably/vcdiff-decoder";

// Initialize WASM module
await init();

// TODO: Filter and FilterExamples are not yet exported from WASM client
// Placeholder until they're implemented
const Filter = {
  eq: (key, value) => ({ op: "eq", key, value }),
  ne: (key, value) => ({ op: "ne", key, value }),
  gt: (key, value) => ({ op: "gt", key, value }),
  gte: (key, value) => ({ op: "gte", key, value }),
  lt: (key, value) => ({ op: "lt", key, value }),
  lte: (key, value) => ({ op: "lte", key, value }),
  in: (key, values) => ({ op: "in", key, values }),
  and: (...filters) => ({ op: "and", filters }),
  or: (...filters) => ({ op: "or", filters }),
};

const FilterExamples = {
  simpleEq: () => Filter.eq("event_type", "goal"),
  complexAnd: () =>
    Filter.and(Filter.eq("event_type", "goal"), Filter.gte("priority", "high")),
};

// Create a compatibility wrapper for Pusher JS API
class PusherCompatWrapper {
  constructor(appKey, config) {
    console.log("[PusherCompat] Creating client with config:", config);

    const options = new SockudoOptions(appKey);

    if (config.cluster) options.cluster = config.cluster;
    if (config.wsHost) options.ws_host = config.wsHost;
    if (config.wsPort) options.ws_port = parseInt(config.wsPort);
    if (config.forceTLS !== undefined) options.use_tls = config.forceTLS;
    if (config.authEndpoint) options.auth_endpoint = config.authEndpoint;

    // Enable delta compression if configured
    if (config.enableDeltaCompression) {
      options.enableDeltaCompression();
    }

    console.log("[PusherCompat] Options created:", options);

    try {
      // Call WasmSockudo constructor which will return a SockudoClient instance with all methods
      this._client = new WasmSockudo(appKey, options);
      console.log("[PusherCompat] Client created:", this._client);
      console.log(
        "[PusherCompat] Client.bind exists?",
        typeof this._client.bind,
      );
      console.log(
        "[PusherCompat] Client.connect exists?",
        typeof this._client.connect,
      );
      console.log("[PusherCompat] WasmSockudo?", this._client.constructor.name);
    } catch (e) {
      console.error("[PusherCompat] Failed to create client:", e);
      throw e;
    }

    this._isConnecting = false;
    this._autoConnect = config.autoConnect !== false; // Default to true for Pusher compatibility

    // Create connection object for compatibility
    this.connection = {
      state: "initialized",
      socket_id: null,
      _callbacks: {},
      connection: null, // Will be set when WebSocket is available

      bind: (event, callback) => {
        if (!this.connection._callbacks[event]) {
          this.connection._callbacks[event] = [];
        }
        this.connection._callbacks[event].push(callback);

        // Don't forward connection events to client - we handle them in _monitorPusherEvents
        // Only forward if it's not a connection state event
        const connectionEvents = [
          "connected",
          "connecting",
          "disconnected",
          "failed",
          "error",
          "unavailable",
        ];
        if (!connectionEvents.includes(event)) {
          this._client.bind(event, callback);
        }
      },

      send_event: (event, data) => {
        return this._client.send_event(event, data);
      },
    };

    // Set up state monitoring
    this._monitorState();

    // Monitor Pusher protocol events to update state
    this._monitorPusherEvents();

    // Auto-connect for Pusher compatibility (unless explicitly disabled)
    if (this._autoConnect) {
      // Use setTimeout to allow event bindings to be set up first
      setTimeout(() => this.connect(), 0);
    }
  }

  _triggerConnectionCallbacks(eventName, data) {
    // Trigger callbacks registered via connection.bind()
    if (this.connection._callbacks[eventName]) {
      this.connection._callbacks[eventName].forEach((callback) => {
        try {
          callback(data);
        } catch (e) {
          console.error(`[PusherCompat] Error in ${eventName} callback:`, e);
        }
      });
    }
  }

  _monitorPusherEvents() {
    // Listen for Pusher connection established event
    this._client.bind("pusher:connection_established", (eventData) => {
      console.log(
        "[PusherCompat] pusher:connection_established received:",
        eventData,
      );
      this.connection.state = "connected";

      // The event data can come in different formats:
      // 1. { event: "...", data: "{...}" } - wrapped with stringified data
      // 2. { socket_id: "..." } - direct object
      // 3. "{socket_id: ...}" - stringified object

      let socketId = null;

      // If data is wrapped in event object
      if (eventData && eventData.data) {
        if (typeof eventData.data === "string") {
          try {
            const parsed = JSON.parse(eventData.data);
            socketId = parsed.socket_id;
          } catch (e) {
            console.error("[PusherCompat] Failed to parse data:", e);
          }
        } else if (eventData.data.socket_id) {
          socketId = eventData.data.socket_id;
        }
      }
      // If data is direct object
      else if (eventData && eventData.socket_id) {
        socketId = eventData.socket_id;
      }
      // If data is stringified
      else if (typeof eventData === "string") {
        try {
          const parsed = JSON.parse(eventData);
          socketId = parsed.socket_id;
        } catch (e) {
          console.error("[PusherCompat] Failed to parse string data:", e);
        }
      }

      // Fallback to client's socket_id
      this.connection.socket_id = socketId || this._client.socket_id;

      console.log(
        `[PusherCompat] Connected with socket_id: ${this.connection.socket_id}`,
      );

      // Trigger the connected callback
      this._triggerConnectionCallbacks("connected");
    });

    // Listen for Pusher errors
    this._client.bind("pusher:error", (data) => {
      console.log("[PusherCompat] pusher:error received:", data);
      this.connection.state = "error";
    });
  }

  _monitorState() {
    // Monitor state changes via state_change event
    this._client.bind("state_change", (data) => {
      console.log("[PusherCompat] State change:", data);
      if (data && data.current) {
        const oldState = this.connection.state;
        this.connection.state = data.current;

        // Update socket_id when connected
        if (data.current === "connected") {
          this.connection.socket_id = this._client.socket_id;
        } else if (
          data.current === "disconnected" ||
          data.current === "failed"
        ) {
          this.connection.socket_id = null;
        }

        console.log(
          `[PusherCompat] State updated: ${oldState} -> ${data.current}`,
        );
      }
    });

    // Also monitor individual state events for backwards compatibility
    this._client.bind("connecting", () => {
      console.log("[PusherCompat] connecting event");
      this.connection.state = "connecting";
    });

    this._client.bind("connected", () => {
      console.log("[PusherCompat] connected event");
      this.connection.state = "connected";
      this.connection.socket_id = this._client.socket_id;
    });

    this._client.bind("disconnected", () => {
      console.log("[PusherCompat] disconnected event");
      this.connection.state = "disconnected";
      this.connection.socket_id = null;
    });

    this._client.bind("failed", () => {
      console.log("[PusherCompat] failed event");
      this.connection.state = "failed";
    });

    this._client.bind("error", (err) => {
      console.log("[PusherCompat] error event:", err);
      this.connection.state = "error";
    });
  }

  async connect() {
    if (this._isConnecting || this.connection.state === "connected") {
      return;
    }
    this._isConnecting = true;
    this.connection.state = "connecting";

    // Trigger connecting callback
    this._triggerConnectionCallbacks("connecting");

    try {
      await this._client.connect();
      this._isConnecting = false;
    } catch (error) {
      this._isConnecting = false;
      this.connection.state = "failed";
      this._triggerConnectionCallbacks("failed");
      throw error;
    }
  }

  disconnect() {
    this._client.disconnect();
  }

  subscribe(channelName, filter) {
    // TODO: Pass filter when WASM client supports it
    return this._client.subscribe(channelName);
  }

  unsubscribe(channelName) {
    this._client.unsubscribe(channelName);
  }

  bind(event, callback) {
    this._client.bind(event, callback);
  }

  bind_global(callback) {
    this._client.bind_global(callback);
  }

  unbind(event) {
    this._client.unbind(event);
  }

  unbind_all() {
    this._client.unbind_all();
  }

  unbind_global() {
    this._client.unbind_global();
  }

  channel(name) {
    return this._client.channel(name);
  }

  get_delta_stats() {
    return this._client.get_delta_stats();
  }

  reset_delta_stats() {
    this._client.reset_delta_stats();
  }
}

// Make libraries available globally for app.js
window.Pusher = PusherCompatWrapper;
window.SockudoClient = SockudoClient;
window.SockudoOptions = SockudoOptions;
window.Filter = Filter;
window.FilterExamples = FilterExamples;
window.fossilDelta = fossilDelta;
window.vcdiff = { decode: vcdiffDecode };

// Import the main app
import "./app.js";
