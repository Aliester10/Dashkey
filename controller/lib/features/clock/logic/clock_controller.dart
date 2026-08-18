/// Clock Controller — waktu sistem real-time (PRD clock §26).
///
/// Mengikuti arsitektur Riverpod yang sudah dipakai DashKey:
/// - `Timer.periodic` untuk update setiap detik.
/// - State hanya berubah jika nilai detik berbeda (hindari rebuild berlebih).
/// - Timer di-cancel otomatis saat provider tidak dipakai (ref.onDispose).
library;

import 'dart:async';

import 'package:flutter_riverpod/flutter_riverpod.dart';

final clockProvider =
    NotifierProvider<ClockController, DateTime>(ClockController.new);

class ClockController extends Notifier<DateTime> {
  Timer? _timer;

  @override
  DateTime build() {
    _timer = Timer.periodic(const Duration(seconds: 1), (_) {
      final now = DateTime.now();
      // Hanya update bila detik berubah — halaman tidak rebuild tiap tick.
      if (now.second != state.second) {
        state = now;
      }
    });
    ref.onDispose(() => _timer?.cancel());
    return DateTime.now();
  }
}
