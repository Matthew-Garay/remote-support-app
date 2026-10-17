# Arquitectura - Remote Support App

## Visión general

Arquitectura multi-proceso con separación estricta entre UI (Flutter) y Kernel (Rust).

## Componentes

### 1. Flutter UI (Frontend)
- Interfaz de usuario: dispositivos, configuración, chat, archivos.
- Renderizado de video remoto en `TextureWidget`.
- Comunicación con el kernel vía `flutter_rust_bridge` (FFI).

### 2. Rust Core Service (Kernel)
- Screen Capture: DXGI (Windows), ScreenCaptureKit (macOS), PipeWire/X11/Wayland (Linux).
- Input Simulator: SendInput (Windows), CGEvent (macOS), uinput (Linux).
- Video Encoder: NVENC/QuickSync/VideoToolbox + fallback x264/x265.
- Network Engine: WebRTC (webrtc-rs) / QUIC.

### 3. Backend (Rust / Actix-web)
- Signal Server: autenticación, emparejamiento, negociación SDP/ICE.
- STUN/TURN: Coturn (Docker).

### 4. Comunicación
- UI ↔ Kernel: FFI con compartimiento de memoria (texturas de video).
- Kernel ↔ Kernel: WebRTC DataChannel (inputs) + MediaStream (video).
- Cliente ↔ Signal Server: WebSocket cifrado (DTLS 1.3).

## Modelos de proceso

- Proceso UI: Flutter (ejecución en sesión de usuario).
- Proceso Kernel: Rust daemon (background service). Permite control aunque no haya sesión de UI abierta.

## Diagrama de datos

```
Flutter UI ──FFI──▶ Rust Kernel ──WebRTC──▶ Otro Rust Kernel ──FFI──▶ Flutter UI
   │                                                     │
   ▼                                                     ▼
WebSocket (Signal)                                  WebSocket (Signal)
```

## Seguridad

- Cifrado E2EE: DTLS 1.3 + SRTP para video, XChaCha20-Poly1305 para DataChannels.
- Autenticación: PIN/ID + contraseña temporal cifrada localmente.
- Permisos: solicitud explícita al usuario en cada SO.
