# Especificaciones Técnicas

## Stack Tecnológico

| Capa | Tecnología |
|------|------------|
| UI | Flutter (Dart) |
| Kernel | Rust |
| FFI | flutter_rust_bridge v2 (Fase 5) |
| Video codec | H.264 (OpenH264 SW real; H.265/AV1 futuros) |
| HW encoder | NVENC / QuickSync / VideoToolbox (detección real, SDK por enlazar) |
| SW encoder | Cisco OpenH264 0.9 (`source`, `ScreenContentRealTime`, CBR) |
| Transporte | WebRTC / QUIC |
| Cifrado | DTLS 1.3 + SRTP / XChaCha20-Poly1305 |
| Backend | Rust (Actix-web / Tokio) |
| ICE | STUN / TURN (Coturn) |

## Encoder (Fase 2 ✅)

- **Negociación**: NVENC → QuickSync → VideoToolbox → OpenH264-SW.
  Detección en runtime sin enlazar SDKs (`LoadLibraryExW` / `dlopen` +
  nodos `/dev/nvidia*`, `/dev/dri/renderD*`, overrides
  `REMOTE_SUPPORT_FORCE_{NVENC,QSV,VTB}=1`).
- **Software real**: OpenH264 0.9, `UsageType::ScreenContentRealTime`,
  `RateControlMode::Bitrate`, `SpsPpsStrategy::IncreasingId`,
  `skip_frames(true)`, `Complexity::Medium`, salida Annex-B.
- **Entrada**: RGB8 / RGBA8 / BGRA8 / I420 → conversión BT.601 propia
  a I420 (`kernel/src/encoder/convert.rs`).
- **ABR**: controlador con histéresis (umbral 20%, cooldown 500 ms,
  `note_network_estimate()` llamado por el NetworkEngine en Fase 3).
- **Salida**: `EncodedPacket { data: Annex-B, is_keyframe, pts_us, backend }`.
- **Async**: `encode()` corre en `spawn_blocking` (OpenH264 bloquea).
- **Validaciones**: `cargo test` (tipos, conversión, ABR, pipeline E2E),
  `cargo run --example pipeline`, `cargo bench --bench encoder_bench`.

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
