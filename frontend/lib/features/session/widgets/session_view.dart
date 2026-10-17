import 'package:flutter/material.dart';

class SessionView extends StatelessWidget {
  const SessionView({super.key});

  @override
  Widget build(BuildContext context) {
    return const Scaffold(
      body: _SessionBody(),
    );
  }
}

class _SessionBody extends StatelessWidget {
  const _SessionBody();

  @override
  Widget build(BuildContext context) {
    return Stack(
      children: [
        // Placeholder para el contenido remoto
        Container(
          decoration: BoxDecoration(
            color: Colors.grey[800],
            borderRadius: BorderRadius.circular(12),
          ),
          child: const Center(
            child: Column(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                CircularProgressIndicator(color: Colors.white),
                SizedBox(height: 16),
                Text(
                  'Conectando...',
                  style: TextStyle(color: Colors.white),
                ),
              ],
            ),
          ),
        ),
        // Overlay de controles
        Positioned(
          bottom: 16,
          right: 16,
          child: Container(
            decoration: BoxDecoration(
              color: Colors.black54,
              borderRadius: BorderRadius.circular(12),
              boxShadow: [
                BoxShadow(
                  color: Colors.black.withOpacity(0.3),
                  blurRadius: 8,
                ),
              ],
            ),
            child: Row(
              mainAxisSize: MainAxisSize.min,
              children: [
                _ControlButton(
                  icon: Icons.mouse_arrow_resize,
                  tooltip: 'Mover cursor',
                  onPressed: () {},
                ),
                _ControlButton(
                  icon: Icons.keyboard,
                  tooltip: 'Teclado',
                  onPressed: () {},
                ),
                _ControlButton(
                  icon: Icons.videocam_off,
                  tooltip: 'Apagar cámara',
                  onPressed: () {},
                ),
                _ControlButton(
                  icon: Icons.minimize,
                  tooltip: 'Minimizar',
                  onPressed: () {},
                ),
                _ControlButton(
                  icon: Icons.quit_drug,
                  tooltip: 'Finalizar sesión',
                  color: Colors.red,
                  onPressed: () {},
                ),
              ],
            ),
          ),
        ),
      ],
    );
  }
}

class _ControlButton extends StatelessWidget {
  final IconData icon;
  final String tooltip;
  final Color? color;
  final VoidCallback? onPressed;

  const _ControlButton({
    required this.icon,
    required this.tooltip,
    this.color,
    this.onPressed,
  });

  @override
  Widget build(BuildContext context) {
    return Tooltip(
      message: tooltip,
      child: ElevatedButton(
        style: ElevatedButton.styleFrom(
          backgroundColor: color ?? Colors.white,
          foregroundColor: Colors.black,
          padding: const EdgeInsets.all(12),
          shape: RoundedRectangleBorder(
            borderRadius: BorderRadius.circular(8),
          ),
        ),
        onPressed: onPressed,
        child: Icon(icon, size: 20),
      ),
    );
  }
}
