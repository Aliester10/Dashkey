/// Haptic manager (PRD §5) — pemetaan jenis aksi → getaran Flutter.
library;

import 'package:flutter/services.dart';

import 'feedback_config.dart';

/// Peta haptic sesuai rekomendasi PRD §5:
/// normal = light, penting = medium, destruktif = heavy, toggle = selection.
Future<void> playHaptic(DashHaptic haptic) async {
  switch (haptic) {
    case DashHaptic.light:
      await HapticFeedback.lightImpact();
    case DashHaptic.medium:
      await HapticFeedback.mediumImpact();
    case DashHaptic.heavy:
      await HapticFeedback.heavyImpact();
    case DashHaptic.selection:
      await HapticFeedback.selectionClick();
    case DashHaptic.none:
      break;
  }
}

/// Pilih haptic berdasarkan jenis aksi tombol (PRD §5 mapping).
DashHaptic hapticForAction(String? actionType) {
  if (actionType == null) return DashHaptic.light;
  final t = actionType.toLowerCase();
  if (t.contains('toggle') || t.contains('mute') || t == 'media_control') {
    return DashHaptic.selection;
  }
  if (t.contains('stream') || t.contains('recording') || t.contains('obs')) {
    return DashHaptic.medium;
  }
  return DashHaptic.light;
}
