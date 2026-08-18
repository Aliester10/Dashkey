import 'package:flutter/material.dart';
import 'package:google_fonts/google_fonts.dart';

import 'app_colors.dart';

class AppTheme {
  AppTheme._();

  static ThemeData get darkTheme {
    final base = ThemeData.dark(useMaterial3: true);
    final interTextTheme = GoogleFonts.interTextTheme(base.textTheme);

    return base.copyWith(
      scaffoldBackgroundColor: AppColors.primaryBackground,
      colorScheme: const ColorScheme.dark(
        primary: AppColors.blueAccent,
        onPrimary: AppColors.primaryText,
        secondary: AppColors.cyanAccent,
        onSecondary: AppColors.primaryText,
        surface: AppColors.surface,
        onSurface: AppColors.primaryText,
        background: AppColors.primaryBackground,
      ),
      appBarTheme: const AppBarTheme(
        backgroundColor: AppColors.primaryBackground,
        foregroundColor: AppColors.primaryText,
        elevation: 0,
        surfaceTintColor: Colors.transparent,
      ),
      dividerTheme: const DividerThemeData(
        color: AppColors.lightBorder,
        thickness: 1,
        space: 1,
      ),
      cardTheme: CardThemeData(
        color: AppColors.surface,
        elevation: 0,
        margin: EdgeInsets.zero,
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(20),
          side: const BorderSide(color: AppColors.lightBorder),
        ),
      ),
      inputDecorationTheme: InputDecorationTheme(
        filled: true,
        fillColor: AppColors.primaryBackground,
        border: OutlineInputBorder(
          borderRadius: BorderRadius.circular(16),
          borderSide: const BorderSide(color: AppColors.lightBorder),
        ),
        enabledBorder: OutlineInputBorder(
          borderRadius: BorderRadius.circular(16),
          borderSide: const BorderSide(color: AppColors.lightBorder),
        ),
        focusedBorder: OutlineInputBorder(
          borderRadius: BorderRadius.circular(16),
          borderSide: const BorderSide(color: AppColors.blueAccent, width: 1.5),
        ),
      ),
      textTheme: interTextTheme.copyWith(
        titleLarge: interTextTheme.titleLarge?.copyWith(
          color: AppColors.primaryText,
          fontWeight: FontWeight.w600,
          fontSize: 24,
        ),
        titleMedium: interTextTheme.titleMedium?.copyWith(
          color: AppColors.primaryText,
          fontWeight: FontWeight.w500,
          fontSize: 14,
        ),
        bodyMedium: interTextTheme.bodyMedium?.copyWith(
          color: AppColors.primaryText,
          fontSize: 14,
        ),
        bodySmall: interTextTheme.bodySmall?.copyWith(
          color: AppColors.secondaryText,
          fontSize: 13,
        ),
      ),
      iconTheme: const IconThemeData(
        color: AppColors.primaryText,
      ),
    );
  }
}
