/// FlipDigit — satu digit kartu flip dengan animasi mekanik (PRD §9–§11).
///
/// Alur animasi (PRD §10):
/// 1. Upper half menampilkan digit lama, berputar 0 → -90° (melipat ke atas).
/// 2. Pada setengah putaran, nilai baru disiapkan.
/// 3. Lower half menampilkan digit baru, berputar +90° → 0° (terbuka).
///
/// Hanya digit yang berubah yang menjalankan animasi (PRD §11);
/// digit statis tidak di-rebuild ulang oleh FlipClock.
library;

import 'package:flutter/material.dart';

import 'flip_card.dart';

class FlipDigit extends StatefulWidget {
  const FlipDigit({
    super.key,
    required this.value,
    required this.width,
    required this.height,
    this.animate = false,
  });

  /// Nilai digit saat ini (0–9).
  final int value;

  /// Ukuran kartu (dihitung responsif oleh ClockPage).
  final double width;
  final double height;

  /// True jika digit ini berubah sejak detik lalu (perlu animasi).
  final bool animate;

  @override
  State<FlipDigit> createState() => _FlipDigitState();
}

class _FlipDigitState extends State<FlipDigit>
    with SingleTickerProviderStateMixin {
  late final AnimationController _controller;

  /// Digit lama — ditampilkan di upper half saat flip berlangsung.
  int? _previousValue;

  @override
  void initState() {
    super.initState();
    _controller = AnimationController(
      vsync: this,
      duration: const Duration(milliseconds: 400),
    );
  }

  @override
  void didUpdateWidget(covariant FlipDigit oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (widget.value != oldWidget.value) {
      _previousValue = oldWidget.value;
      if (widget.animate) {
        _controller.forward(from: 0);
      } else {
        _previousValue = null;
      }
    }
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return ListenableBuilder(
      listenable: _controller,
      builder: (context, _) {
        final progress = _controller.value; // 0..1; 0 = statis
        return FlipCard(
          width: widget.width,
          height: widget.height,
          oldValue: _previousValue ?? widget.value,
          newValue: widget.value,
          progress: progress,
        );
      },
    );
  }
}
