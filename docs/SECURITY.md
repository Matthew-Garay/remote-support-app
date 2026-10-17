# Seguridad

## Cifrado

- Video: DTLS 1.3 + SRTP (WebRTC nativo).
- DataChannel (inputs, chat, archivos): XChaCha20-Poly1305.
- Señalización: WebSocket sobre TLS (WSS).

## Autenticación

- ID de dispositivo único generado localmente.
- Contraseña temporal cifrada localmente (no se envía en texto plano).
- Autenticación mutua entre clientes antes de establecer P2P.

## Permisos

| SO | Permiso requerido | Cómo se solicita |
|----|-------------------|------------------|
| Windows | UAC elevado / Servicio | Instalador con elevación + Windows Service |
| macOS | Screen Recording + Accessibility | Modal del sistema (no se puede saltar) |
| Linux | uinput / root | Reglas udev o ejecución con sudo |

## Auditoría

- Registro de todas las sesiones de control remoto.
- Logs de eventos de inyección de input.
- Alerta si se intenta conectar sin permisos válidos.

## Advertencia

Esta aplicación requiere privilegios elevados. No distribuir binarios sin firmar y sin revisar por seguridad.
