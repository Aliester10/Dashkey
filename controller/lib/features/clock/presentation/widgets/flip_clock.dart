/// FlipClock — susunan HH : MM : SS (PRD §8).
///
/// - Mengambil waktu dari `clockProvider` (Timer.periodic).
/// - Membandingkan digit dengan detik sebelumnya; hanya digit yang berubah
///   yang menerima flag `animate` (PRD §11, §12).
/// - Semua kartu statis tidak di-rebuild ulang (parent hanya mem-build
///   kembali kartu yang digit-nya berubah).
library;

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../logic/clock_controller.dart';
import 'clock_date.dart';
import 'flip_digit.dart';
import 'flip_separator.dart';

class FlipClock extends ConsumerWidget {
  const FlipClock({
    super.key,
    required this.cardWidth,
    required this.cardHeight,
  });

  /// Ukuran kartu (dihitung responsif oleh ClockPage).
  final double cardWidth;
  final double cardHeight;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final now = ref.watch(clockProvider);

    final digits = [
      (now.hour ~/ 10),
      (now.hour % 10),
      (now.minute ~/ 10),
      (now.minute % 10),
      (now.second ~/ 10),
      (now.second % 10),
    ];

    return Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        Row(
          mainAxisSize: MainAxisSize.min,
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            for (var i = 0; i < 6; i++) ...[
              _DigitSlot(
                value: digits[i],
                cardWidth: cardWidth,
                cardHeight: cardHeight,
                position: i,
                now: now,
              ),
              if (i == 1 || i == 3)
                FlipSeparator(height: cardHeight)
              else if (i < 5)
                SizedBox(width: cardWidth * 0.10),
            ],
          ],
        ),
        const SizedBox(height: 32),
        const ClockDate(),
      ],
    );
  }
}

/// Slot digit — mengingat nilai sebelumnya untuk deteksi perubahan
/// (hanya slot yang berubah yang di-rebuild dengan animate: true).
class _DigitSlot extends ConsumerStatefulWidget {
  const _DigitSlot({
    required this.value,
    required this.cardWidth,
    required this.cardHeight,
    required this.position,
    required this.now,
  });

  final int value;
  final double cardWidth;
  final double cardHeight;
  final int position;
  final DateTime now;

  @override
  ConsumerState<_DigitSlot> createState() => _DigitSlotState();
}

class _DigitSlotState extends ConsumerState<_DigitSlot> {
  int? _previous;

  @override
  Widget build(BuildContext context) {
    final changed = _previous != null && _previous != widget.value;
    final animate = changed;
    _previous = widget.value;

    return FlipDigit(
      value: widget.value,
      width: widget.cardWidth,
      height: widget.cardHeight,
      animate: animate,
    );
  }
}
