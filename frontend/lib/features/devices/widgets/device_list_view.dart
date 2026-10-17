import 'package:flutter/material.dart';
import 'package:remote_support_app/features/devices/models/device_model.dart';

class DeviceListView extends StatelessWidget {
  const DeviceListView({super.key});

  @override
  Widget build(BuildContext context) {
    final devices = _mockDevices();

    return Scaffold(
      appBar: AppBar(
        title: const Text('Dispositivos conectados'),
        centerTitle: true,
      ),
      body: ListView.builder(
        itemCount: devices.length,
        itemBuilder: (context, index) {
          final device = devices[index];
          return Card(
            margin: const EdgeInsets.all(8),
            child: Padding(
              padding: const EdgeInsets.all(16),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Row(
                    children: [
                      Container(
                        width: 48,
                        height: 48,
                        decoration: BoxDecoration(
                          color: Colors.blue.withOpacity(0.1),
                          borderRadius: BorderRadius.circular(12),
                        ),
                        child: const Icon(
                          Icons.computer_outlined,
                          color: Colors.blue,
                          size: 28,
                        ),
                      ),
                      const SizedBox(width: 16),
                      Expanded(
                        child: Column(
                          crossAxisAlignment: CrossAxisAlignment.start,
                          children: [
                            Text(
                              device.name,
                              style: const TextStyle(
                                fontSize: 18,
                                fontWeight: FontWeight.w600,
                              ),
                            ),
                            Text(
                              '${device.os} • ${device.username}',
                              style: const TextStyle(
                                fontSize: 14,
                                color: Colors.grey,
                              ),
                            ),
                          ],
                        ),
                      ),
                      Container(
                        padding: const EdgeInsets.symmetric(
                          horizontal: 12,
                          vertical: 6,
                        ),
                        decoration: BoxDecoration(
                          color: Colors.green.withOpacity(0.1),
                          borderRadius: BorderRadius.circular(8),
                        ),
                        child: const Text(
                          'Conectado',
                          style: TextStyle(
                            fontSize: 12,
                            fontWeight: FontWeight.w500,
                            color: Colors.green,
                          ),
                        ),
                      ),
                    ],
                  ),
                  const SizedBox(height: 12),
                  Row(
                    children: [
                      Container(
                        padding: const EdgeInsets.all(8),
                        decoration: BoxDecoration(
                          color: Colors.blue.withOpacity(0.1),
                          borderRadius: BorderRadius.circular(8),
                        ),
                        child: const Icon(
                          Icons.signal_wifi_4_bar,
                          size: 16,
                          color: Colors.blue,
                        ),
                      ),
                      const SizedBox(width: 8),
                      Text(
                        '${device.networkSpeed} Mbps',
                        style: const TextStyle(fontSize: 12, color: Colors.grey),
                      ),
                      const Spacer(),
                      Text(
                        '${device.cpuUsage.toInt()}%',
                        style: const TextStyle(fontSize: 12, color: Colors.grey),
                      ),
                      const SizedBox(width: 8),
                      ElevatedButton.icon(
                        onPressed: () {},
                        icon: const Icon(Icons.play_arrow),
                        label: const Text('Iniciar sesión'),
                      ),
                    ],
                  ),
                ],
              ),
            ),
          );
        },
      ),
      floatingActionButton: FloatingActionButton(
        onPressed: () {
          // TODO: Show add device dialog
        },
        child: const Icon(Icons.add),
      ),
    );
  }

  List<DeviceModel> _mockDevices() {
    return [
      DeviceModel(
        id: 'device-001',
        name: 'PC de trabajo Contraloria',
        os: 'Windows 11',
        username: 'Contraloria\\admin',
        ip: '192.168.1.100',
        networkSpeed: 94.2,
        cpuUsage: 23.5,
        memoryUsage: 62.1,
        isOnline: true,
        lastSeen: DateTime.now().subtract(const Duration(minutes: 5)),
      ),
      DeviceModel(
        id: 'device-002',
        name: 'MacBook Pro 16"',
        os: 'macOS 14',
        username: 'juan.perez',
        ip: '192.168.1.101',
        networkSpeed: 88.7,
        cpuUsage: 15.2,
        memoryUsage: 45.3,
        isOnline: true,
        lastSeen: DateTime.now().subtract(const Duration(minutes: 2)),
      ),
      DeviceModel(
        id: 'device-003',
        name: 'Servidor Ubuntu 24.04',
        os: 'Ubuntu 24.04',
        username: 'admin',
        ip: '192.168.1.102',
        networkSpeed: 92.1,
        cpuUsage: 45.0,
        memoryUsage: 78.9,
        isOnline: true,
        lastSeen: DateTime.now().subtract(const Duration(minutes: 1)),
      ),
    ];
  }
}
