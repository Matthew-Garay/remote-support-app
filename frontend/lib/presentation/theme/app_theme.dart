import 'package:flutter/material.dart';

class AppTheme {
  static ThemeData get lightTheme {
    final colorScheme = ColorScheme.light(
      primary: const Color(0xFF10B981),
      onPrimary: Colors.white,
      secondary: const Color(0xFF6366F1),
      onSecondary: Colors.white,
      surface: const Color(0xFFF8FAFC),
      onSurface: const Color(0xFF1E293B),
      error: Colors.red,
      onError: Colors.white,
    );

    return ThemeData(
      useMaterial3: true,
      colorScheme: colorScheme,
      scaffoldBackgroundColor: colorScheme.surface,
      cardTheme: const CardTheme(
        elevation: 2,
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.all(Radius.circular(12)),
        ),
      ),
      appBarTheme: const AppBarTheme(
        backgroundColor: Colors.white,
        foregroundColor: Colors.black,
        elevation: 0,
        centerTitle: true,
      ),
      floatingActionButtonTheme: const FloatingActionButtonThemeData(
        backgroundColor: Color(0xFF10B981),
        foregroundColor: Colors.white,
        elevation: 4,
      ),
    );
  }

  static ThemeData get darkTheme {
    final colorScheme = ColorScheme.dark(
      primary: const Color(0xFF10B981),
      onPrimary: Colors.black,
      secondary: const Color(0xFF6366F1),
      onSecondary: Colors.black,
      surface: const Color(0xFF1E293B),
      onSurface: const Color(0xFFF1F5F9),
      error: Colors.red,
      onError: Colors.black,
    );

    return ThemeData(
      useMaterial3: true,
      colorScheme: colorScheme,
      scaffoldBackgroundColor: colorScheme.surface,
      cardTheme: const CardTheme(
        elevation: 2,
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.all(Radius.circular(12)),
        ),
      ),
      appBarTheme: const AppBarTheme(
        backgroundColor: Color(0xFF1E293B),
        foregroundColor: Colors.white,
        elevation: 0,
        centerTitle: true,
      ),
      floatingActionButtonTheme: const FloatingActionButtonThemeData(
        backgroundColor: Color(0xFF10B981),
        foregroundColor: Colors.white,
        elevation: 4,
      ),
    );
  }
}
