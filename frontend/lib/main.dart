import 'package:flutter/material.dart';
import 'core/services/app_service.dart';
import 'core/services/theme_service.dart';
import 'features/devices/widgets/device_list_view.dart';
import 'features/session/widgets/session_view.dart';
import 'presentation/theme/app_theme.dart';

void main() {
  WidgetsFlutterBinding.ensureInitialized();
  AppService.init();
  ThemeService.init();
  runApp(const RemoteSupportApp());
}

class RemoteSupportApp extends StatelessWidget {
  const RemoteSupportApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Remote Support App',
      debugShowCheckedModeBanner: false,
      theme: AppTheme.lightTheme,
      darkTheme: AppTheme.darkTheme,
      home: const DeviceListView(),
    );
  }
}
