/// Layar uji koneksi (Fase 0) — connect WebSocket, kirim echo, lihat balasan.
library;

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../core/network/ws_client.dart';
import '../../../core/protocol/messages.dart';
import '../providers/connection_providers.dart';

class ConnectionTestScreen extends ConsumerStatefulWidget {
  const ConnectionTestScreen({super.key});

  @override
  ConsumerState<ConnectionTestScreen> createState() =>
      _ConnectionTestScreenState();
}

class _ConnectionTestScreenState extends ConsumerState<ConnectionTestScreen> {
  final _hostController = TextEditingController(text: '192.168.1.10');
  final _portController = TextEditingController(text: '48484');
  final _echoController = TextEditingController(text: 'halo dashkey');
  final List<String> _log = [];

  @override
  void dispose() {
    _hostController.dispose();
    _portController.dispose();
    _echoController.dispose();
    super.dispose();
  }

  Future<void> _connect() async {
    final conn = ref.read(wsConnectionProvider);
    final host = _hostController.text.trim();
    final port = int.tryParse(_portController.text.trim()) ?? 48484;

    try {
      await conn.connect(host, port);
    } catch (e) {
      if (mounted) _addLog('Gagal connect: $e');
    }
  }

  void _disconnect() {
    ref.read(wsConnectionProvider).disconnect();
  }

  void _sendEcho() {
    final conn = ref.read(wsConnectionProvider);
    final text = _echoController.text.trim();
    if (text.isEmpty) return;
    conn.send(Outbound.echo(text));
    _addLog('>> echo: $text');
  }

  void _sendPing() {
    ref.read(wsConnectionProvider).send(Outbound.ping());
    _addLog('>> ping');
  }

  void _addLog(String line) {
    setState(() {
      _log.add('[${DateTime.now().toIso8601String().substring(11, 19)}] $line');
      if (_log.length > 100) _log.removeAt(0);
    });
  }

  @override
  Widget build(BuildContext context) {
    final statusAsync = ref.watch(wsStatusProvider);
    ref.listen<AsyncValue<ProtocolMessage>>(wsMessagesProvider, (prev, next) {
      next.whenData((msg) {
        switch (msg.type) {
          case Inbound.echoReply:
            _addLog('<< echo_reply: ${msg.payload?['text']}');
          case Inbound.pong:
            _addLog('<< pong');
          case Inbound.error:
            _addLog('<< error: ${msg.payload?['message']}');
          default:
            _addLog('<< ${msg.type}: ${msg.payload}');
        }
      });
    });

    final status = statusAsync.value ?? WsStatus.disconnected;
    final isConnected = status == WsStatus.connected;

    return Scaffold(
      appBar: AppBar(
        title: const Text('DashKey — Uji Koneksi'),
        backgroundColor: Theme.of(context).colorScheme.inversePrimary,
      ),
      body: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          children: [
            Row(
              children: [
                Expanded(
                  flex: 3,
                  child: TextField(
                    controller: _hostController,
                    decoration: const InputDecoration(
                      labelText: 'IP Host',
                      border: OutlineInputBorder(),
                    ),
                    keyboardType: TextInputType.url,
                  ),
                ),
                const SizedBox(width: 8),
                Expanded(
                  child: TextField(
                    controller: _portController,
                    decoration: const InputDecoration(
                      labelText: 'Port',
                      border: OutlineInputBorder(),
                    ),
                    keyboardType: TextInputType.number,
                  ),
                ),
              ],
            ),
            const SizedBox(height: 12),
            Row(
              children: [
                FilledButton(
                  onPressed: isConnected ? null : _connect,
                  child: const Text('Connect'),
                ),
                const SizedBox(width: 8),
                OutlinedButton(
                  onPressed: isConnected ? _disconnect : null,
                  child: const Text('Disconnect'),
                ),
                const SizedBox(width: 12),
                _StatusBadge(status: status),
              ],
            ),
            const Divider(height: 32),
            Row(
              children: [
                Expanded(
                  child: TextField(
                    controller: _echoController,
                    decoration: const InputDecoration(
                      labelText: 'Teks echo',
                      border: OutlineInputBorder(),
                    ),
                  ),
                ),
                const SizedBox(width: 8),
                FilledButton(
                  onPressed: isConnected ? _sendEcho : null,
                  child: const Text('Kirim Echo'),
                ),
                const SizedBox(width: 8),
                OutlinedButton(
                  onPressed: isConnected ? _sendPing : null,
                  child: const Text('Ping'),
                ),
              ],
            ),
            const SizedBox(height: 16),
            Expanded(
              child: Container(
                width: double.infinity,
                padding: const EdgeInsets.all(8),
                decoration: BoxDecoration(
                  color: Colors.black87,
                  borderRadius: BorderRadius.circular(8),
                ),
                child: ListView.builder(
                  itemCount: _log.length,
                  itemBuilder: (context, i) => Text(
                    _log[i],
                    style: const TextStyle(
                      color: Colors.greenAccent,
                      fontFamily: 'monospace',
                      fontSize: 12,
                    ),
                  ),
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }
}

class _StatusBadge extends StatelessWidget {
  const _StatusBadge({required this.status});

  final WsStatus status;

  @override
  Widget build(BuildContext context) {
    final (color, label) = switch (status) {
      WsStatus.connected => (Colors.green, 'Connected'),
      WsStatus.connecting => (Colors.orange, 'Connecting...'),
      WsStatus.error => (Colors.red, 'Error'),
      WsStatus.disconnected => (Colors.grey, 'Disconnected'),
    };
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
      decoration: BoxDecoration(
        color: color,
        borderRadius: BorderRadius.circular(16),
      ),
      child: Text(
        label,
        style: const TextStyle(color: Colors.white, fontSize: 12),
      ),
    );
  }
}
