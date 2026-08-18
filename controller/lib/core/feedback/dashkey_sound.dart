/// Sound manager (PRD §8) — feedback suara opsional.
///
/// Default OFF. Menggunakan SystemSound bawaan Flutter (click, 20–40 ms).
/// Volume disediakan sebagai setting cadangan untuk custom asset di masa depan.
library;

import 'package:flutter/services.dart';

class DashKeySound {
  DashKeySound._();

  /// Putar suara "click" singkat (default: OFF diatur oleh FeedbackEngine).
  static Future<void> click() async {
    try {
      await SystemSound.play(SystemSoundType.click);
    } catch (_) {
      // Device/platform tanpa dukungan sound system — abaikan.
    }
  }
}
