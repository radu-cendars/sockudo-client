import 'bridge_generated.dart';

/// Re-export main classes with primary Sockudo names
export 'bridge_generated.dart' show
  SockudoClient,
  SockudoOptions,
  Channel,
  PusherEvent,
  DeltaStats,
  MemberInfo;

// Pusher-compatible aliases for backward compatibility
/// Pusher-compatible alias for SockudoClient
typedef Pusher = SockudoClient;

/// Pusher-compatible alias for SockudoOptions
typedef PusherOptions = SockudoOptions;

/// Helper extension methods for Sockudo Client
extension SockudoClientExtensions on SockudoClient {
  /// Convenience method to create and connect a client
  static Future<SockudoClient> connectWith(PusherOptions options) async {
    final client = SockudoClient.new$(options: options);
    await client.connect();
    return client;
  }
}

/// Helper methods for options
extension PusherOptionsBuilder on PusherOptions {
  /// Create options with sensible defaults
  static PusherOptions build({
    required String appKey,
    String? cluster,
    String? wsHost,
    int? wsPort,
    bool? useTls,
    String? authEndpoint,
    int? activityTimeoutMs,
    int? pongTimeoutMs,
    bool? enableDeltaCompression,
    bool? enableStats,
  }) {
    return PusherOptions(
      appKey: appKey,
      cluster: cluster,
      wsHost: wsHost,
      wsPort: wsPort,
      useTls: useTls,
      authEndpoint: authEndpoint,
      activityTimeoutMs: activityTimeoutMs,
      pongTimeoutMs: pongTimeoutMs,
      enableDeltaCompression: enableDeltaCompression,
      enableStats: enableStats,
    );
  }
}
