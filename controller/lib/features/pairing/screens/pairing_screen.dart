/// Layar pairing — scan QR dari Host, kirim pair_request, simpan kredensial.
library;

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:mobile_scanner/mobile_scanner.dart';

import '../../connection/controllers/connection_controller.dart';

class PairingScreen extends ConsumerStatefulWidget {
  const PairingScreen({super.key});

  @override
  ConsumerState<PairingScreen> createState() => _PairingScreenState();
}

class _PairingScreenState extends ConsumerState<PairingScreen> {
  bool _handled = false;

  void _onDetect(BarcodeCapture capture) {
    if (_handled) return;
    final raw = capture.barcodes.firstOrNull?.rawValue;
    if (raw == null || raw.isEmpty) return;

    // QR pairing berisi JSON {host, port, token}.
    if (!raw.contains('"host"') || !raw.contains('"token"')) {
      _showHint('QR bukan QR pairing DashKey');
      return;
    }

    setState(() => _handled = true);
    ref.read(connectionControllerProvider.notifier).pairFromQr(raw);
  }

  void _showHint(String message) {
    ScaffoldMessenger.of(context)
      ..hideCurrentSnackBar()
      ..showSnackBar(SnackBar(content: Text(message)));
  }

  @override
  Widget build(BuildContext context) {
    final conn = ref.watch(connectionControllerProvider);

    // Reaksi terhadap perubahan fase koneksi.
    ref.listen(connectionControllerProvider, (prev, next) {
      if (next.phase == ConnectionPhase.error && next.lastError != null) {
        _showHint(next.lastError!);
        setState(() => _handled = false);
      }
    });

    return Scaffold(
      appBar: AppBar(title: const Text('Pairing dengan Host')),
      body: Column(
        children: [
          Expanded(
            child: Stack(
              children: [
                MobileScanner(onDetect: _onDetect),
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
              ],
            ),
          ),
          Padding(
            padding: const EdgeInsets.all(16),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                Text(
                  'Jalankan `dashkey-host pair` di PC lalu arahkan kamera '
                  'ke QR code yang tampil di layar PC.',
                  textAlign: TextAlign.center,
                  style: Theme.of(context).textTheme.bodyMedium,
                ),
                const SizedBox(height: 12),
                OutlinedButton.icon(
                  onPressed: () =>
                      ref.read(connectionControllerProvider.notifier).tryAutoConnect(),
                  icon: const Icon(Icons.refresh),
                  label: const Text('Coba Reconnect (kredensial lama)'),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}
