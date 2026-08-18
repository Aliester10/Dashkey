/// FlipSeparator — pemisah ":" antar kelompok digit (PRD §20).
///
/// Visual lebih subtle daripada digit: warna abu-abu, tanpa kartu.
library;

import 'package:flutter/material.dart';

class FlipSeparator extends StatelessWidget {
  const FlipSeparator({super.key, required this.height, this.width = 0});

  /// Tinggi separator (ikut tinggi kartu).
  final double height;

  /// Lebar opsional; 0 = auto dari teks.
  final double width;

  @override
  Widget build(BuildContext context) {
    return SizedBox(
      width: width <= 0 ? height * 0.18 : width,
      height: height,
      child: Center(
        child: Text(
          ':',
          style: TextStyle(
            color: const Color(0xFF777777),
            fontSize: height * 0.45,
            fontWeight: FontWeight.w600,
            height: 1.0,
          ),
        ),
      ),
    );
  }
}
