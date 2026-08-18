import 'package:dashkey_controller/core/feedback/dashkey_haptic.dart';
import 'package:dashkey_controller/core/feedback/feedback_config.dart';
import 'package:dashkey_controller/core/feedback/feedback_engine.dart';
import 'package:dashkey_controller/shared/widgets/dashkey_button.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:shared_preferences/shared_preferences.dart';

void main() {
  group('Tactile Feedback (prdbutton.md)', () {
    test('hapticForAction mengikuti mapping PRD §5', () {
      expect(hapticForAction('open_app'), DashHaptic.light);
      expect(hapticForAction('obs_toggle_mute'), DashHaptic.selection);
      expect(hapticForAction('media_control'), DashHaptic.selection);
      expect(hapticForAction('obs_start_stream'), DashHaptic.medium);
      expect(hapticForAction(null), DashHaptic.light);
    });

    test('FeedbackProfile default values', () {
      final physical = FeedbackSettings.defaults[FeedbackProfile.physical]!;
      expect(physical.profile, FeedbackProfile.physical);
      expect(physical.hapticEnabled, true);
      expect(physical.hapticIntensity, 1.0);
      expect(physical.soundEnabled, false);

      final silent = FeedbackSettings.defaults[FeedbackProfile.silent]!;
      expect(silent.hapticEnabled, false);
      expect(silent.soundEnabled, false);
    });

    test('FeedbackSettings round-trip JSON', () {
      const settings = FeedbackSettings(
        profile: FeedbackProfile.mechanical,
        hapticIntensity: 1.4,
        animationSpeed: 0.85,
      );
      final back = FeedbackSettings.fromJson(settings.toJson());
      expect(back.profile, FeedbackProfile.mechanical);
      expect(back.hapticIntensity, 1.4);
      expect(back.animationSpeed, 0.85);
    });

    test('FeedbackEngine applyProfile mengganti nilai', () async {
      SharedPreferences.setMockInitialValues({});
      final container = ProviderContainer();
      addTearDown(container.dispose);

      final engine = container.read(feedbackEngineProvider.notifier);
      await engine.applyProfile(FeedbackProfile.mechanical);
      final state = container.read(feedbackEngineProvider);
      expect(state.profile, FeedbackProfile.mechanical);
      expect(state.hapticIntensity, 1.4);

      await engine.setHapticEnabled(false);
      expect(container.read(feedbackEngineProvider).hapticEnabled, false);
    });

    testWidgets('DashKeyButton memanggil onPressed saat tap', (tester) async {
      var pressed = 0;
      await tester.pumpWidget(
        ProviderScope(
          child: MaterialApp(
            home: Scaffold(
              body: Center(
                child: DashKeyButton(
                  onPressed: () => pressed++,
                  child: const Text('TAP'),
                ),
              ),
            ),
          ),
        ),
      );

      await tester.tap(find.byType(DashKeyButton));
      await tester.pump(const Duration(milliseconds: 300));
      expect(pressed, 1);
    });

    testWidgets('DashKeyButton disabled tidak merespons tap', (tester) async {
      var pressed = 0;
      await tester.pumpWidget(
        ProviderScope(
          child: MaterialApp(
            home: Scaffold(
              body: Center(
                child: DashKeyButton(
                  disabled: true,
                  onPressed: () => pressed++,
                  child: const Text('OFF'),
                ),
              ),
            ),
          ),
        ),
      );

      await tester.tap(find.byType(DashKeyButton), warnIfMissed: false);
      await tester.pump(const Duration(milliseconds: 300));
      expect(pressed, 0);
    });
  });
}
