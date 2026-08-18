/// Halaman Trackpad (PRD2) — halaman khusus, terpisah dari grid tombol.
///
/// Dipakai dengan dua cara:
/// - `embedded: false` (default): halaman penuh dengan AppBar sendiri,
///   dibuka via tombol Trackpad di halaman grid.
/// - `embedded: true`: ditampilkan sebagai body di dalam Scaffold grid
///   (saat active page adalah trackpad).
library;

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../core/network/ws_client.dart';
import '../../core/protocol/messages.dart';
import '../../features/connection/providers/connection_providers.dart';
import '../settings/settings_controller.dart';
import 'trackpad_area.dart';

class TrackpadScreen extends ConsumerWidget {
  const TrackpadScreen({
    super.key,
    this.pageName = 'Trackpad',
    this.embedded = false,
  });

  final String pageName;

  /// Jika true, tampil tanpa Scaffold/AppBar (menjadi body halaman grid).
  final bool embedded;

  void _send(WidgetRef ref, ProtocolMessage message) {
    final conn = ref.read(wsConnectionProvider);
    if (conn.status == WsStatus.connected) {
      conn.send(message);
    }
  }

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final sensitivity = TrackpadSensitivity.fromSlider(
      ref.watch(sensitivityProvider).round(),
    );

    final content = Padding(
      padding: const EdgeInsets.all(12),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Expanded(
            child: TrackpadArea(
              sensitivity: sensitivity,
              onMessage: (message) => _send(ref, message),
            ),
          ),
          const SizedBox(height: 12),
          Row(
            children: [
              Expanded(
                child: FilledButton.icon(
                  onPressed: () => _send(ref, Outbound.mouseClick('left')),
                  icon: const Icon(Icons.touch_app),
                  label: const Text('Klik Kiri'),
                  style: FilledButton.styleFrom(
                    padding: const EdgeInsets.symmetric(vertical: 14),
                  ),
                ),
              ),
              const SizedBox(width: 10),
              Expanded(
                child: FilledButton.tonalIcon(
                  onPressed: () => _send(ref, Outbound.mouseClick('right')),
                  icon: const Icon(Icons.more_horiz),
                  label: const Text('Klik Kanan'),
                  style: FilledButton.styleFrom(
                    padding: const EdgeInsets.symmetric(vertical: 14),
                  ),
                ),
              ),
            ],
          ),
        ],
      ),
    );

    if (embedded) return content;

    // Halaman penuh tersendiri dengan AppBar.
    return Scaffold(
      appBar: AppBar(
        title: Text(pageName),
        actions: [
          IconButton(
            tooltip: 'Settings',
            icon: const Icon(Icons.settings_outlined),
            onPressed: () => Navigator.of(context).push(
              MaterialPageRoute(builder: (_) => const _TrackpadSettings()),
            ),
          ),
        ],
      ),
      body: content,
    );
  }
}

/// Pengaturan sensitivitas dalam konteks halaman trackpad.
class _TrackpadSettings extends ConsumerWidget {
  const _TrackpadSettings();

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final sensitivity = ref.watch(sensitivityProvider);
    final notifier = ref.read(sensitivityProvider.notifier);

    return Scaffold(
      appBar: AppBar(title: const Text('Sensitivitas Kursor')),
      body: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          children: [
            const ListTile(
              leading: Icon(Icons.touch_app),
              title: Text('Kecepatan kursor'),
              subtitle: Text(
                'Semakin besar nilainya, kursor di PC semakin cepat '
                'mengikuti gerakan jari.',
              ),
            ),
            Slider(
              value: sensitivity,
              min: 1,
              max: 10,
              divisions: 9,
              label: sensitivity.round().toString(),
              onChanged: (v) => notifier.set(v),
            ),
            Text(
              '${sensitivity.round()} / 10',
              style: Theme.of(context).textTheme.titleMedium,
            ),
          ],
        ),
      ),
    );
  }
}
