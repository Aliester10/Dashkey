/// ClockDate — tanggal di bawah jam (PRD §14).
///
/// Format: "TUESDAY, AUG 18, 2026" (uppercase, letter-spacing, muted).
library;

import 'package:flutter/material.dart';

import '../../../../shared/theme/app_colors.dart';

class ClockDate extends StatelessWidget {
  const ClockDate({super.key, this.compact = false});

  /// Mode compact (tanpa nama hari) untuk layout yang sempit.
  final bool compact;

  static const _days = [
    'MONDAY', 'TUESDAY', 'WEDNESDAY', 'THURSDAY',
    'FRIDAY', 'SATURDAY', 'SUNDAY',
  ];
  static const _months = [
    'JAN', 'FEB', 'MAR', 'APR', 'MAY', 'JUN',
    'JUL', 'AUG', 'SEP', 'OCT', 'NOV', 'DEC',
  ];

  String _format(DateTime now) {
    final day = _days[now.weekday - 1];
    final month = _months[now.month - 1];
    if (compact) {
      return '$month ${now.day}, ${now.year}';
    }
    return '$day, $month ${now.day}, ${now.year}';
  }

  @override
  Widget build(BuildContext context) {
    return Text(
      _format(DateTime.now()),
      style: TextStyle(
        color: AppColors.secondaryText.withValues(alpha: 0.75),
        fontSize: 15,
        fontWeight: FontWeight.w500,
        letterSpacing: 1.5,
      ),
    );
  }
}
