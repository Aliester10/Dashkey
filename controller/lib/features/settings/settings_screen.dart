/// Layar Settings (HP) — pengaturan perangkat, termasuk sensitivitas
/// kursor trackpad (PRD2 FR-T8).
library;

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../trackpad/trackpad_area.dart';
import 'settings_controller.dart';

class SettingsScreen extends ConsumerWidget {
  const SettingsScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final sensitivity = ref.watch(sensitivityProvider);
    final notifier = ref.read(sensitivityProvider.notifier);

    return Scaffold(
      appBar: AppBar(title: const Text('Settings')),
      body: ListView(
        padding: const EdgeInsets.all(16),
        children: [
          Card(
            child: Padding(
              padding: const EdgeInsets.all(16),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  const ListTile(
                    leading: Icon(Icons.touch_app),
                    title: Text('Sensitivitas Kursor'),
                    subtitle: Text(
                      'Semakin besar nilainya, kursor di PC semakin cepat '
                      'bergerak mengikuti jari. Berlaku untuk page Trackpad.',
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
                  Center(
                    child: Text(
                      '${sensitivity.round()} / 10',
                      style: Theme.of(context).textTheme.titleMedium,
                    ),
                  ),
                ],
              ),
            ),
          ),
          const SizedBox(height: 8),
          Card(
            child: Padding(
              padding: const EdgeInsets.all(16),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text('Gesture Trackpad',
                      style: Theme.of(context).textTheme.titleMedium),
                  const SizedBox(height: 8),
                  const _GestureRow(
                    icon: Icons.pan_tool,
                    label: 'Geser 1 jari',
                    detail: 'Menggerakkan kursor',
                  ),
                  const _GestureRow(
                    icon: Icons.tap_and_play,
                    label: 'Tap 1 jari',
                    detail: 'Klik kiri',
                  ),
                  const _GestureRow(
                    icon: Icons.pan_tool_alt,
                    label: 'Tekan-tahan lalu geser',
                    detail: 'Drag (klik kiri ditahan)',
                  ),
                  const _GestureRow(
                    icon: Icons.swipe_vertical,
                    label: 'Geser 2 jari vertikal',
                    detail: 'Scroll',
                  ),
                  const _GestureRow(
                    icon: Icons.touch_app,
                    label: 'Tap 2 jari',
                    detail: 'Klik kanan',
                  ),
                ],
              ),
            ),
          ),
        ],
      ),
    );
  }
}

class _GestureRow extends StatelessWidget {
  const _GestureRow({
    required this.icon,
    required this.label,
    required this.detail,
  });

  final IconData icon;
  final String label;
  final String detail;

  @override
  Widget build(BuildContext context) {
    return ListTile(
      leading: Icon(icon, color: Theme.of(context).colorScheme.primary),
      title: Text(label),
      subtitle: Text(detail),
    );
  }
}

/// Preview kecil — dipakai tes unit.
class TrackpadSensitivityTest {
  static double factor(double slider) =>
      TrackpadSensitivity.fromSlider(slider.round());
}
