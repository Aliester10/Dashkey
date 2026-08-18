import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'shared/theme/app_theme.dart';

import 'features/connection/controllers/connection_controller.dart';
import 'features/grid/screens/grid_screen.dart';
import 'features/pairing/screens/pairing_screen.dart';

void main() {
  runApp(const ProviderScope(child: DashKeyApp()));
}

class DashKeyApp extends StatelessWidget {
  const DashKeyApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'DashKey',
      debugShowCheckedModeBanner: false,
      theme: AppTheme.darkTheme,
      darkTheme: AppTheme.darkTheme,
      themeMode: ThemeMode.dark,
      home: const BootstrapScreen(),
    );
  }
}

/// Layar bootstrap: memilih layar sesuai fase koneksi.
class BootstrapScreen extends ConsumerWidget {
  const BootstrapScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final conn = ref.watch(connectionControllerProvider);

    return switch (conn.phase) {
      ConnectionPhase.authenticated => const GridScreen(),
      ConnectionPhase.needsPairing ||
      ConnectionPhase.disconnected ||
      ConnectionPhase.error =>
        const PairingScreen(),
      ConnectionPhase.connecting ||
      ConnectionPhase.authenticating =>
        Scaffold(
          body: Center(
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                const CircularProgressIndicator(),
                const SizedBox(height: 16),
                Text('Menghubungkan ke ${conn.host}:${conn.port}...'),
              ],
            ),
          ),
        ),
    };
  }
}
