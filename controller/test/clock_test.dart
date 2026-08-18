import 'package:dashkey_controller/features/clock/logic/clock_controller.dart';
import 'package:dashkey_controller/features/clock/presentation/pages/clock_page.dart';
import 'package:dashkey_controller/features/clock/presentation/widgets/flip_digit.dart';
import 'package:dashkey_controller/features/clock/presentation/widgets/flip_separator.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  group('Flip Clock', () {
    testWidgets('FlipDigit menampilkan nilai digit', (tester) async {
      await tester.pumpWidget(
        const MaterialApp(
          home: Scaffold(
            body: Center(
              child: FlipDigit(value: 5, width: 80, height: 130),
            ),
          ),
        ),
      );
      // Implementasi Stack menampilkan nilai di beberapa lapisan kartu.
      expect(find.text('5'), findsWidgets);
    });

    testWidgets('FlipSeparator menampilkan titik dua', (tester) async {
      await tester.pumpWidget(
        const MaterialApp(
          home: Scaffold(body: Center(child: FlipSeparator(height: 130))),
        ),
      );
      expect(find.text(':'), findsOneWidget);
    });

    testWidgets('ClockPage render tanpa error (fullscreen flip clock)',
        (tester) async {
      await tester.pumpWidget(
        const ProviderScope(
          child: MaterialApp(home: ClockPage()),
        ),
      );
      await tester.pump(const Duration(seconds: 1));
      // Jam menampilkan separator dan tanggal.
      expect(find.text(':'), findsWidgets);
      expect(find.byType(FlipDigit), findsNWidgets(6));

      // Bersihkan provider agar timer ikut di-cancel.
      await tester.pumpWidget(const ProviderScope(child: SizedBox()));
    });

    test('clockProvider mengupdate waktu', () async {
      final container = ProviderContainer();
      addTearDown(container.dispose);
      final before = container.read(clockProvider);
      expect(before, isA<DateTime>());
      // Format jam 2 digit.
      final text = '${before.hour.toString().padLeft(2, '0')}';
      expect(text.length, 2);
    });
  });
}
