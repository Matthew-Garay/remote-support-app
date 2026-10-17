import 'package:flutter/foundation.dart';
import 'package:shared_preferences/shared_preferences.dart';

enum AppMode { development, production }

class AppService {
  static AppService? _instance;
  static AppService get instance => _instance ??= AppService._();

  late final SharedPreferences _prefs;

  AppService._();

  static Future<void> init() async {
    _instance ??= AppService._();
    await _instance!._init();
  }

  Future<void> _init() async {
    _prefs = await SharedPreferences.getInstance();
  }

  // ===== App Settings =====

  AppMode get appMode {
    final mode = _prefs.getString('app_mode');
    return mode == 'production' ? AppMode.production : AppMode.development;
  }

  Future<void> setAppMode(AppMode mode) async {
    await _prefs.setString('app_mode', mode.name);
  }

  bool get isDevelopment => appMode == AppMode.development;

  double get targetFPS {
    return _prefs.getDouble('target_fps') ?? 60.0;
  }

  Future<void> setTargetFPS(double fps) async {
    await _prefs.setDouble('target_fps', fps);
  }

  int get maxBitrate {
    return _prefs.getInt('max_bitrate') ?? 6000000;
  }

  Future<void> setMaxBitrate(int bitrate) async {
    await _prefs.setInt('max_bitrate', bitrate);
  }

  // ===== Session =====

  String? get currentSessionId {
    return _prefs.getString('current_session_id');
  }

  Future<void> setCurrentSessionId(String? id) async {
    if (id == null) {
      await _prefs.remove('current_session_id');
    } else {
      await _prefs.setString('current_session_id', id);
    }
  }

  String? get currentDeviceId {
    return _prefs.getString('current_device_id');
  }

  Future<void> setCurrentDeviceId(String? id) async {
    if (id == null) {
      await _prefs.remove('current_device_id');
    } else {
      await _prefs.setString('current_device_id', id);
    }
  }

  // ===== Theme =====

  bool get isDarkMode {
    return _prefs.getBool('is_dark_mode') ?? false;
  }

  Future<void> setDarkMode(bool enabled) async {
    await _prefs.setBool('is_dark_mode', enabled);
  }

  // ===== Analytics =====

  Future<void> logEvent(String eventName, {Map<String, dynamic>? extra}) async {
    await _prefs.setString('last_event', '$eventName|${DateTime.now().millisecondsSinceEpoch}');
  }
}
