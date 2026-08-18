import 'package:flutter/material.dart';

class AppColors {
  AppColors._();

  // Neumorphic base: semua surface harus tetap berada di keluarga warna ini.
  static const Color primaryBackground = Color(0xFF20242C);
  static const Color secondaryBackground = Color(0xFF252A33);
  static const Color surface = Color(0xFF2B303A);
  static const Color surfaceLight = Color(0xFF343B47);
  static const Color surfaceHighlight = Color(0xFF414A58);
  static const Color surfaceDeep = Color(0xFF171A20);

  // Text
  static const Color primaryText = Color(0xFFECEFF5);
  static const Color secondaryText = Color(0xFFAAB2C0);
  static const Color disabledText = Color(0xFF697281);

  // Accents
  static const Color blueAccent = Color(0xFF7C8CFF);
  static const Color brightBlueAccent = Color(0xFF91A0FF);
  static const Color googleBlue = Color(0xFF4285F4);
  static const Color successGreen = Color(0xFF10B981);
  static const Color cyanAccent = Color(0xFF06B6D4);
  static const Color dangerRed = Color(0xFFEF4444);
  static const Color discordPurple = Color(0xFF5865F2);
  static const Color ferdiumPurple = Color(0xFF9333EA);

  // Borders & Glow
  static const Color lightBorder = Color(0x223E4653);

  // Highlight dan shadow sengaja lebih kontras agar relief terlihat di HP.
  static Color get lightShadow => const Color(0xFF4A5362).withOpacity(0.78);
  static Color get darkShadow => const Color(0xFF11141A).withOpacity(0.82);
}
