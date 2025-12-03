import 'package:flutter/material.dart';
import 'package:sockudo_client/sockudo_client.dart';

void main() {
  runApp(const MyApp());
}

class MyApp extends StatelessWidget {
  const MyApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Sockudo Client Demo',
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(seedColor: Colors.deepPurple),
        useMaterial3: true,
      ),
      home: const MyHomePage(title: 'Sockudo Client Demo'),
    );
  }
}

class MyHomePage extends StatefulWidget {
  const MyHomePage({super.key, required this.title});

  final String title;

  @override
  State<MyHomePage> createState() => _MyHomePageState();
}

class _MyHomePageState extends State<MyHomePage> {
  FlutterSockudoClient? _client;
  String _status = 'Disconnected';
  String _socketId = 'None';
  final List<String> _events = [];

  @override
  void initState() {
    super.initState();
    _initClient();
  }

  Future<void> _initClient() async {
    try {
      // Initialize logging
      initLogging(level: 'debug');

      // Create client with your configuration
      final options = PusherOptionsBuilder.build(
        appKey: 'your-app-key',
        cluster: 'mt1',
        // Or use custom host:
        // wsHost: 'your-server.com',
        // wsPort: 6001,
        useTls: true,
        enableDeltaCompression: true,
        enableStats: true,
      );

      _client = FlutterSockudoClient.new$(options: options);

      setState(() {
        _status = 'Initialized';
      });
    } catch (e) {
      setState(() {
        _status = 'Error: $e';
      });
    }
  }

  Future<void> _connect() async {
    if (_client == null) return;

    try {
      setState(() {
        _status = 'Connecting...';
      });

      await _client!.connect();

      final socketId = _client!.getSocketId();

      setState(() {
        _status = 'Connected';
        _socketId = socketId ?? 'Unknown';
      });
    } catch (e) {
      setState(() {
        _status = 'Connection Error: $e';
      });
    }
  }

  void _disconnect() {
    if (_client == null) return;

    try {
      _client!.disconnect();

      setState(() {
        _status = 'Disconnected';
        _socketId = 'None';
      });
    } catch (e) {
      setState(() {
        _status = 'Error: $e';
      });
    }
  }

  Future<void> _subscribeToChannel() async {
    if (_client == null || !_client!.isConnected()) {
      setState(() {
        _events.add('Error: Not connected');
      });
      return;
    }

    try {
      final channel = _client!.subscribe(channelName: 'my-channel');

      setState(() {
        _events.add('Subscribed to: ${channel.getName()}');
      });

      // In a real app, you'd set up event listeners here
      // using the stream API
    } catch (e) {
      setState(() {
        _events.add('Subscribe Error: $e');
      });
    }
  }

  void _showStats() {
    if (_client == null) return;

    final stats = _client!.getDeltaStats();

    if (stats == null) {
      setState(() {
        _events.add('No delta stats available');
      });
      return;
    }

    setState(() {
      _events.add('Delta Stats:');
      _events.add('  Total Messages: ${stats.totalMessages}');
      _events.add('  Delta Messages: ${stats.deltaMessages}');
      _events.add('  Bandwidth Saved: ${stats.bandwidthSavedPercent.toStringAsFixed(2)}%');
    });
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        backgroundColor: Theme.of(context).colorScheme.inversePrimary,
        title: Text(widget.title),
      ),
      body: Padding(
        padding: const EdgeInsets.all(16.0),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            Card(
              child: Padding(
                padding: const EdgeInsets.all(16.0),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      'Status: $_status',
                      style: Theme.of(context).textTheme.titleMedium,
                    ),
                    const SizedBox(height: 8),
                    Text('Socket ID: $_socketId'),
                    Text('Version: ${getVersion()}'),
                  ],
                ),
              ),
            ),
            const SizedBox(height: 16),
            Wrap(
              spacing: 8,
              runSpacing: 8,
              children: [
                ElevatedButton(
                  onPressed: _connect,
                  child: const Text('Connect'),
                ),
                ElevatedButton(
                  onPressed: _disconnect,
                  child: const Text('Disconnect'),
                ),
                ElevatedButton(
                  onPressed: _subscribeToChannel,
                  child: const Text('Subscribe'),
                ),
                ElevatedButton(
                  onPressed: _showStats,
                  child: const Text('Show Stats'),
                ),
              ],
            ),
            const SizedBox(height: 16),
            Expanded(
              child: Card(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.stretch,
                  children: [
                    Padding(
                      padding: const EdgeInsets.all(16.0),
                      child: Row(
                        mainAxisAlignment: MainAxisAlignment.spaceBetween,
                        children: [
                          Text(
                            'Events',
                            style: Theme.of(context).textTheme.titleMedium,
                          ),
                          IconButton(
                            icon: const Icon(Icons.clear),
                            onPressed: () {
                              setState(() {
                                _events.clear();
                              });
                            },
                          ),
                        ],
                      ),
                    ),
                    const Divider(height: 1),
                    Expanded(
                      child: ListView.builder(
                        itemCount: _events.length,
                        itemBuilder: (context, index) {
                          return ListTile(
                            dense: true,
                            title: Text(
                              _events[index],
                              style: const TextStyle(fontFamily: 'monospace'),
                            ),
                          );
                        },
                      ),
                    ),
                  ],
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }

  @override
  void dispose() {
    _client?.disconnect();
    super.dispose();
  }
}
