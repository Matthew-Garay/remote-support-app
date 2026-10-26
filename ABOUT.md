# Remote Support App

## Acerca de

Aplicacion de soporte remoto de alto rendimiento, estilo TeamViewer o AnyDesk, con arquitectura multi-proceso que combina una interfaz de usuario en Flutter, un kernel en Rust para captura de pantalla y codificacion de video, y un backend en Rust con Actix-web para senalizacion, STUN y TURN. Permite el acceso remoto a sistemas para soporte tecnico, con transferencia de archivos, chat y sesion de control remoto. Abarca captura de pantalla, codificacion de video con aceleracion por hardware, comunicacion WebRTC/QUIC y servidores de senalizacion NAT traversal.

## Caracteristicas principales

- Interfaz de usuario en Flutter con soporte multiplataforma (Windows, macOS, Linux, Android, iOS).
- Kernel de Rust para captura de pantalla, inyeccion de entradas y codificacion de video.
- Codificacion de video con aceleracion por hardware (NVENC, QuickSync, VToolbox).
- Comunicacion WebRTC/QUIC con cifrado DTLS/SRTP y XChaCha20-Poly1305.
- Backend en Rust con Actix-web y Tokio para senalizacion, STUN y TURN.
- Autenticacion basada en JWT para sesiones de soporte remoto.

## Stack tecnologico

| Categoria | Tecnologia |
|-----------|------------|
| Frontend | Flutter SDK >= 3.24.0, Dart >= 3.5.0 |
| Kernel | Rust >= 1.82, flutter_rust_bridge |
| Backend | Rust, Actix-web 4, Tokio 1.42 |
| Comunicacion remota | WebRTC, QUIC, DTLS/SRTP, XChaCha20-Poly1305 |
| NAT traversal | STUN, TURN (coturn via Docker) |
| Autenticacion | JWT |

## Creditos

Desarrollado por: Matthew Ilveneff Garay Perez
Correo Electronico: 1911.garay.perez@gmail.com