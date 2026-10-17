import 'package:flutter/material.dart';

class ThemeService {
  static ThemeService? _instance;

  static Future<ThemeService> init() async {
    _instance ??= ThemeService._();
    return _instance!;
  }

  ThemeService._();

  final ColorScheme _lightColorScheme = ColorScheme.light(
    primary: const Color(0xFF10B981),
    onPrimary: Colors.white,
    secondary: const Color(0xFF6366F1),
    onSecondary: Colors.white,
    surface: const Color(0xFFF8FAFC),
    onSurface: const Color(0xFF1E293B),
    error: Colors.red,
    onError: Colors.white,
  );

  final ColorScheme _darkColorScheme = ColorScheme.dark(
    primary: const Color(0xFF10B981),
    onPrimary: Colors.black,
    secondary: const Color(0xFF818CF8),
    onSecondary: Colors.black,
    surface: const Color(0xFF1E293B),
    onSurface: const Color(0xFFF1F5F9),
    error: Colors.red,
    onError: Colors.black,
  );

  ThemeData get lightTheme => ThemeData(
        useMaterial3: true,
        colorScheme: _lightColorScheme,
        scaffoldBackgroundColor: _lightColorScheme.surface,
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
      );

  ThemeData get darkTheme => ThemeData(
        useMaterial3: true,
        colorScheme: _darkColorScheme,
        scaffoldBackgroundColor: _darkColorScheme.surface,
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
      );
}
