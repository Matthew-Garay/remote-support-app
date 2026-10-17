import 'package:freezed_annotation/freezed_annotation.dart';

part 'device_model.g.dart';

enum DeviceOS {
  @JsonValue('windows')
  windows,
  @JsonValue('macos')
  macos,
  @JsonValue('linux')
  linux,
  @JsonValue('android')
  android,
  @JsonValue('ios')
  ios,
  @JsonValue('unknown')
  unknown,
}

enum DeviceStatus {
  @JsonValue('online')
  online,
  @JsonValue('offline')
  offline,
  @JsonValue('busy')
  busy,
  @JsonValue('connecting')
  connecting,
}

@freezed
class DeviceModel with _$DeviceModel {
  factory DeviceModel({
    required String id,
    required String name,
    required DeviceOS os,
    required String username,
    required String ip,
    required double networkSpeed,
    required double cpuUsage,
    required double memoryUsage,
    required bool isOnline,
    required DateTime lastSeen,
  }) = _DeviceModel;

  factory DeviceModel.fromJson(Map<String, dynamic> json) =>
      _$DeviceModelFromJson(json);
}
