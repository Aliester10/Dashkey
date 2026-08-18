/// Feedback Engine (PRD §13) — abstraction tunggal untuk seluruh feedback.
///
/// - Menyimpan & mem-persist pengaturan feedback (shared_preferences).
/// - `trigger()` menjalankan haptic (+ sound opsional) dalam satu titik,
///   sehingga perilaku bisa diubah global tanpa menyentuh setiap tombol.
library;

import 'dart:convert';

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:shared_preferences/shared_preferences.dart';

import 'dashkey_haptic.dart';
import 'dashkey_sound.dart';
import 'feedback_config.dart';

final feedbackEngineProvider =
    NotifierProvider<FeedbackEngine, FeedbackSettings>(FeedbackEngine.new);

class FeedbackEngine extends Notifier<FeedbackSettings> {
  static const _key = 'feedback_settings';

  @override
  FeedbackSettings build() {
    Future.microtask(_load);
    return const FeedbackSettings();
  }

  Future<void> _load() async {
    final prefs = await SharedPreferences.getInstance();
    final raw = prefs.getString(_key);
    if (raw == null) return;
    try {
      state = FeedbackSettings.fromJson(
        jsonDecode(raw) as Map<String, dynamic>,
      );
    } catch (_) {
      // Format lama/rusak — biarkan default.
    }
  }

  Future<void> _persist() async {
    final prefs = await SharedPreferences.getInstance();
    await prefs.setString(_key, jsonEncode(state.toJson()));
  }

  /// Terapkan seluruh profil (PRD §9) — mengembalikan nilai default profil.
  Future<void> applyProfile(FeedbackProfile profile) async {
    state = FeedbackSettings.defaults[profile]!;
    await _persist();
  }

  Future<void> update(FeedbackSettings settings) async {
    state = settings;
    await _persist();
  }

  Future<void> setHapticEnabled(bool enabled) async {
    state = state.copyWith(hapticEnabled: enabled);
    await _persist();
  }

  Future<void> setHapticIntensity(double intensity) async {
    state = state.copyWith(hapticIntensity: intensity.clamp(0.5, 1.5));
    await _persist();
  }

  Future<void> setAnimationEnabled(bool enabled) async {
    state = state.copyWith(animationEnabled: enabled);
    await _persist();
  }

  Future<void> setAnimationSpeed(double speed) async {
    state = state.copyWith(animationSpeed: speed.clamp(0.5, 1.5));
    await _persist();
  }

  Future<void> setSoundEnabled(bool enabled) async {
    state = state.copyWith(soundEnabled: enabled);
    await _persist();
  }

  Future<void> setSoundVolume(double volume) async {
    state = state.copyWith(soundVolume: volume.clamp(0, 1));
    await _persist();
  }

  /// Titik masuk tunggal: panggil saat tombol ditekan.
  Future<void> trigger(DashHaptic haptic) async {
    final settings = state;
    if (settings.hapticEnabled && haptic != DashHaptic.none) {
      await playHaptic(haptic);
    }
    if (settings.soundEnabled) {
      await DashKeySound.click();
    }
  }
}
