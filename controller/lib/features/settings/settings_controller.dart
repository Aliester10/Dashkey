/// Pengaturan perangkat (PRD2 FR-T8): sensitivitas kursor disimpan lokal
/// via shared_preferences agar konsisten setelah reconnect.
library;

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:shared_preferences/shared_preferences.dart';

/// Nilai slider 1..10 (semakin besar, kursor makin cepat).
final sensitivityProvider =
    NotifierProvider<SensitivityNotifier, double>(SensitivityNotifier.new);

class SensitivityNotifier extends Notifier<double> {
  static const _key = 'mouse_sensitivity';

  @override
  double build() {
    Future.microtask(_load);
    return 5;
  }

  Future<void> _load() async {
    final prefs = await SharedPreferences.getInstance();
    state = prefs.getDouble(_key) ?? 5;
  }

  Future<void> set(double value) async {
    state = value.clamp(1, 10);
    final prefs = await SharedPreferences.getInstance();
    await prefs.setDouble(_key, state);
  }
}
