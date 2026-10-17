# Especificaciones Técnicas

## Stack Tecnológico

| Capa | Tecnología |
|------|------------|
| UI | Flutter (Dart) |
| Kernel | Rust |
| FFI | flutter_rust_bridge |
| Video codec | H.264 / H.265 / AV1 |
| HW encoder | NVENC / QuickSync / VideoToolbox |
| Transporte | WebRTC / QUIC |
| Cifrado | DTLS 1.3 + SRTP / XChaCha20-Poly1305 |
| Backend | Rust (Actix-web / Tokio) |
| ICE | STUN / TURN (Coturn) |

## Requisitos por SO

### Windows
- DXGI Desktop Duplication API (captura GPU).
- SendInput API (inyección input).
- NVENC (opcional, aceleración NVIDIA).
- Windows Service para ejecutar sin sesión de usuario.

### macOS
- ScreenCaptureKit (captura baja latencia).
- CoreGraphics / CGEvent (inyección input).
- VideoToolbox (aceleración Apple Silicon/Intel).
- Permisos: Screen Recording + Accessibility.

### Linux
- PipeWire / Wayland / XCB / X11 (captura).
- uinput / libei (inyección input).
- Root o reglas udev para uinput.

## Coordenadas y DPI

- Coordenadas del ratón: relativo 0.0–1.0 en X/Y.
- Conversión: `pos_absoluta = pos_relativa * resolución_host`.
- DPI-aware: consultar `GetDeviceCaps` / `CGDisplayBounds` / `xcb_setup_ports`.

## Streaming

- FPS objetivo: 30–60 FPS.
- Congestion control: Google Congestion Control (GCC) o búfer adaptativo.
- Bitrate adaptativo: reducir FPS o calidad si cae el ancho de banda.
