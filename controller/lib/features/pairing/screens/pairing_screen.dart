/// Layar pairing — dua mode:
/// 1. Deteksi otomatis: UDP broadcast menemukan host aktif di jaringan,
///    cukup tap host untuk pairing (tanpa scan QR).
/// 2. Scan QR: fallback bila host tidak terdeteksi.
library;

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:mobile_scanner/mobile_scanner.dart';

import '../../connection/controllers/connection_controller.dart';
import '../controllers/host_discovery_controller.dart';
import '../services/host_discovery.dart';

class PairingScreen extends ConsumerStatefulWidget {
  const PairingScreen({super.key});

  @override
  ConsumerState<PairingScreen> createState() => _PairingScreenState();
}

enum _PairingMode { autoDetect, scanQr }

class _PairingScreenState extends ConsumerState<PairingScreen> {
  _PairingMode _mode = _PairingMode.autoDetect;
  bool _qrHandled = false;
  Timer? _autoScanTimer;

  @override
  void initState() {
    super.initState();
    // Mulai pencarian otomatis + rescan berkala selama layar terbuka.
    Future.microtask(() => ref.read(hostDiscoveryProvider.notifier).discover());
    _autoScanTimer = Timer.periodic(
      const Duration(seconds: 4),
      (_) => ref.read(hostDiscoveryProvider.notifier).discover(),
    );
  }

  @override
  void dispose() {
    _autoScanTimer?.cancel();
    super.dispose();
  }

  void _onQrDetect(BarcodeCapture capture) {
    if (_qrHandled) return;
    final raw = capture.barcodes.firstOrNull?.rawValue;
    if (raw == null || raw.isEmpty) return;

    // QR pairing berisi JSON {host, port, token}.
    if (!raw.contains('"host"') || !raw.contains('"token"')) {
      _showHint('QR bukan QR pairing DashKey');
      return;
    }

    setState(() => _qrHandled = true);
    ref.read(connectionControllerProvider.notifier).pairFromQr(raw);
  }

  void _showHint(String message) {
    ScaffoldMessenger.of(context)
      ..hideCurrentSnackBar()
      ..showSnackBar(SnackBar(content: Text(message)));
  }

  void _pair(DiscoveredHost host) {
    ref.read(connectionControllerProvider.notifier).pairFromDiscovery(host);
  }

  @override
  Widget build(BuildContext context) {
    final conn = ref.watch(connectionControllerProvider);
    final discovery = ref.watch(hostDiscoveryProvider);

    // Reaksi terhadap perubahan fase koneksi.
    ref.listen(connectionControllerProvider, (prev, next) {
      if (next.phase == ConnectionPhase.error && next.lastError != null) {
        _showHint(next.lastError!);
        setState(() => _qrHandled = false);
      }
    });

    return Scaffold(
      appBar: AppBar(title: const Text('Pairing dengan Host')),
      body: Column(
        children: [
          Padding(
            padding: const EdgeInsets.fromLTRB(16, 12, 16, 4),
            child: SegmentedButton<_PairingMode>(
              segments: const [
                ButtonSegment(
                  value: _PairingMode.autoDetect,
                  icon: Icon(Icons.wifi_tethering),
                  label: Text('Deteksi otomatis'),
                ),
                ButtonSegment(
                  value: _PairingMode.scanQr,
                  icon: Icon(Icons.qr_code_scanner),
                  label: Text('Scan QR'),
                ),
              ],
              selected: {_mode},
              onSelectionChanged: (selection) =>
                  setState(() => _mode = selection.first),
            ),
          ),
          Expanded(
            child: _mode == _PairingMode.autoDetect
                ? _buildAutoDetect(discovery)
                : _buildQrScanner(conn),
          ),
        ],
      ),
    );
  }

  // ── Mode deteksi otomatis ────────────────────────────────────────────
  Widget _buildAutoDetect(HostDiscoveryState discovery) {
    return Column(
      children: [
        Expanded(
          child: discovery.scanning && discovery.hosts.isEmpty
              ? const Center(
                  child: Column(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      CircularProgressIndicator(),
                      SizedBox(height: 12),
                      Text('Mencari host DashKey di jaringan…'),
                    ],
                  ),
                )
              : discovery.hosts.isEmpty
                  ? _EmptyDiscovery(
                      error: discovery.error,
                      onScanQr: () => setState(() => _mode = _PairingMode.scanQr),
                    )
                  : ListView(
                      padding: const EdgeInsets.all(16),
                      children: [
                        Text(
                          'Host ditemukan — ketuk untuk pairing:',
                          style: Theme.of(context).textTheme.bodyMedium,
                        ),
                        const SizedBox(height: 8),
                        for (final host in discovery.hosts)
                          Card(
                            child: ListTile(
                              leading: CircleAvatar(
                                backgroundColor:
                                    Theme.of(context).colorScheme.primaryContainer,
                                child: Icon(
                                  Icons.computer,
                                  color: Theme.of(context)
                                      .colorScheme
                                      .onPrimaryContainer,
                                ),
                              ),
                              title: Text(host.hostName),
                              subtitle: Text(
                                '${host.host}:${host.port}'
                                '${host.version.isNotEmpty ? ' · v${host.version}' : ''}',
                              ),
                              trailing: const Icon(Icons.chevron_right),
                              onTap: () => _pair(host),
                            ),
                          ),
                      ],
                    ),
        ),
        Padding(
          padding: const EdgeInsets.all(16),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              Text(
                'Pastikan DashKey Host berjalan di PC dan '
                'HP terhubung ke jaringan yang sama.',
                textAlign: TextAlign.center,
                style: Theme.of(context).textTheme.bodySmall,
              ),
              const SizedBox(height: 12),
              Row(
                children: [
                  Expanded(
                    child: OutlinedButton.icon(
                      onPressed: () => ref
                          .read(hostDiscoveryProvider.notifier)
                          .discover(),
                      icon: discovery.scanning
                          ? const SizedBox(
                              width: 16,
                              height: 16,
                              child: CircularProgressIndicator(strokeWidth: 2),
                            )
                          : const Icon(Icons.refresh),
                      label: const Text('Pindai ulang'),
                    ),
                  ),
                  const SizedBox(width: 12),
                  Expanded(
                    child: OutlinedButton.icon(
                      onPressed: () =>
                          ref.read(connectionControllerProvider.notifier).tryAutoConnect(),
                      icon: const Icon(Icons.link),
                      label: const Text('Reconnect'),
                    ),
                  ),
                ],
              ),
            ],
          ),
        ),
      ],
    );
  }

  // ── Mode scan QR (fallback) ──────────────────────────────────────────
  Widget _buildQrScanner(ConnectionState conn) {
    return Stack(
      children: [
        MobileScanner(onDetect: _onQrDetect),
        if (conn.phase == ConnectionPhase.connecting ||
            conn.phase == ConnectionPhase.authenticating)
          Container(
            color: Colors.black54,
            alignment: Alignment.center,
            child: const Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                CircularProgressIndicator(),
                SizedBox(height: 12),
                Text(
                  'Menghubungkan...',
                  style: TextStyle(color: Colors.white),
                ),
              ],
            ),
          ),
        Positioned(
          left: 0,
          right: 0,
          bottom: 24,
          child: Center(
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                Container(
                  padding: const EdgeInsets.symmetric(
                    horizontal: 16,
                    vertical: 10,
                  ),
                  decoration: BoxDecoration(
                    color: Colors.black87,
                    borderRadius: BorderRadius.circular(12),
                  ),
                  child: const Text(
                    'Jalankan `dashkey-host pair` di PC lalu arahkan kamera '
                    'ke QR code yang tampil di layar PC.',
                    textAlign: TextAlign.center,
                    style: TextStyle(color: Colors.white, fontSize: 13),
                  ),
                ),
              ],
            ),
          ),
        ),
      ],
    );
  }
}

/// State kosong saat tidak ada host terdeteksi.
class _EmptyDiscovery extends StatelessWidget {
  const _EmptyDiscovery({this.error, required this.onScanQr});

  final String? error;
  final VoidCallback onScanQr;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    return Center(
      child: Padding(
        padding: const EdgeInsets.all(24),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(Icons.wifi_tethering_off, size: 56, color: theme.colorScheme.outline),
            const SizedBox(height: 16),
            Text(
              'Tidak ada host ditemukan',
              style: theme.textTheme.titleMedium,
            ),
            const SizedBox(height: 8),
            Text(
              error ??
                  'Pastikan DashKey Host aktif di PC dan HP di jaringan '
                      'yang sama. Jika tetap tidak muncul, gunakan scan QR.',
              textAlign: TextAlign.center,
              style: theme.textTheme.bodyMedium?.copyWith(
                color: theme.colorScheme.onSurfaceVariant,
              ),
            ),
            const SizedBox(height: 20),
            FilledButton.tonalIcon(
              onPressed: onScanQr,
              icon: const Icon(Icons.qr_code_scanner),
              label: const Text('Coba Scan QR'),
            ),
          ],
        ),
      ),
    );
  }
}