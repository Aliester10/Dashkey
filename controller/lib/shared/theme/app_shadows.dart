import 'package:flutter/material.dart';
import 'app_colors.dart';

class AppShadows {
  AppShadows._();

  /// Raised shadow for normal neumorphic button/container
  static List<BoxShadow> get raised {
    return [
      BoxShadow(
        color: AppColors.darkShadow.withValues(alpha: 0.5),
        offset: const Offset(6, 6),
        blurRadius: 12,
        spreadRadius: 0,
      ),
      BoxShadow(
        color: AppColors.lightShadow.withValues(alpha: 0.4),
        offset: const Offset(-4, -4),
        blurRadius: 10,
        spreadRadius: 0,
      ),
    ];
  }

  /// Hover shadow (slightly raised and glowing)
  static List<BoxShadow> get hover {
    return [
      BoxShadow(
        color: AppColors.darkShadow.withValues(alpha: 0.6),
        offset: const Offset(8, 8),
        blurRadius: 16,
        spreadRadius: 0,
      ),
      BoxShadow(
        color: AppColors.lightShadow.withValues(alpha: 0.5),
        offset: const Offset(-6, -6),
        blurRadius: 14,
        spreadRadius: 0,
      ),
    ];
  }

  // Note: Flutter doesn't natively support inner shadows (inset) with BoxShadow.
  // Inset effects are typically achieved by drawing custom paths or using multiple linear gradients/borders.
  // For the pressed state, we will scale the button down slightly which simulates the press,
  // and we can drop the shadow completely or reduce it to give a flat/inset feel.
  static List<BoxShadow> get pressed {
    return [
      BoxShadow(
        color: AppColors.darkShadow.withValues(alpha: 0.3),
        offset: const Offset(2, 2),
        blurRadius: 4,
        spreadRadius: 0,
      ),
      BoxShadow(
        color: AppColors.lightShadow.withValues(alpha: 0.2),
        offset: const Offset(-2, -2),
        blurRadius: 4,
        spreadRadius: 0,
      ),
    ];
  }
}
