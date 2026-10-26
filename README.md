# Remote Support App

## A. Titulo del Proyecto y Resumen Ejecutivo

Remote Support App es una aplicacion de soporte remoto de alto rendimiento, estilo TeamViewer o AnyDesk, con arquitectura multi-proceso que combina una interfaz de usuario en Flutter, un kernel en Rust para captura de pantalla y codificacion de video, y un backend en Rust con Actix-web para senalizacion, STUN y TURN. El problema que resuelve es permitir el acceso remoto a sistemas para soporte tecnico, con transferencia de archivos, chat y sesion de control remoto. Su alcance tecnologico abarca captura de pantalla, codificacion de video con aceleracion por hardware, comunicacion WebRTC/QUIC y servidores de senalizacion NAT traversal.

## B. Arquitectura del Sistema

El programa sigue una arquitectura multi-proceso con tres componentes principales. El frontend es una aplicacion Flutter que se comunica con el kernel de Rust mediante flutter_rust_bridge (FFI). El kernel de Rust se encarga de la captura de pantalla (DXGI/PipeWire/SCK), inyeccion de entradas, codificacion de video (NVENC/QuickSync/VToolbox) y transporte WebRTC/QUIC con cifrado DTLS/SRTP y XChaCha20-Poly1305. El backend en Rust con Actix-web y Tokio ejecuta el Signal Server para senalizacion WebRTC, servidores STUN y TURN para traversal de NAT, y autenticacion JWT. El flujo de datos va del cliente Flutter al kernel Rust mediante FFI, del kernel al peer remoto mediante WebRTC/QUIC sobre UDP, y del cliente al backend Rust mediante WebSocket sobre WSS para senalizacion y autenticacion. No existe base de datos persistente en la version actual; el TURN se ejecuta externamente mediante Docker Compose (coturn).

## C. Caracteristicas Tecnicas Clave

- Interfaz de usuario en Flutter con soporte multiplataforma (Windows, macOS, Linux, Android, iOS).
- Kernel de Rust para captura de pantalla, inyeccion de entradas y codificacion de video.
- Codificacion de video con aceleracion por hardware (NVENC, QuickSync, VToolbox).
- Comunicación WebRTC/QUIC con cifrado DTLS/SRTP y XChaCha20-Poly1305.
- Backend en Rust con Actix-web y Tokio para senalizacion, STUN y TURN.
- Senalizacion WebSocket en puerto 8080 y servidor HTTPS en puerto 8443.
- Autenticacion basada en JWT para sesiones de soporte remoto.
- TURN server ejecutado externamente mediante Docker Compose (coturn).
- Comunicacion entre Flutter y Rust mediante flutter_rust_bridge (FFI).
- No utiliza base de datos persistente en la version actual.
- Seguridad basada en JWT, WSS/TLS y rate limiting (pendiente de implementacion completa en el kernel de captura).

## D. Especificacion del Stack Tecnologico

| Categoria | Tecnologia / Libreria |
|-----------|----------------------|
| Frontend | Flutter SDK >= 3.24.0, Dart >= 3.5.0 |
| Kernel | Rust >= 1.82, flutter_rust_bridge |
| Backend | Rust, Actix-web 4, Tokio 1.42 |
| WebSocket | actix-ws 4 |
| Serializacion | serde, serde_json |
| Logging | tracing, tracing-subscriber |
| Comunicacion remota | WebRTC, QUIC, DTLS/SRTP, XChaCha20-Poly1305 |
| NAT traversal | STUN, TURN (coturn via Docker) |
| Autenticacion | JWT |
| Variables de entorno | dotenvy 0.15 |

## E. Guia de Instalacion, Configuracion y Despliegue

### Requisitos previos del sistema

- Flutter SDK >= 3.24.0 y Dart >= 3.5.0.
- Rust >= 1.82.
- Docker y Docker Compose (para el backend y TURN).
- Git y Visual Studio Code.
- Sistema operativo Windows, macOS o Linux.

### Secuencia de comandos para clonacion e instalacion

`ash
git clone https://github.com/Matthew-Garay/remote-support-app.git
cd remote-support-app
cp .env.example .env
`

### Configuracion de variables de entorno

Crear archivo .env en la raiz del proyecto con la siguiente estructura:

`
SERVER_PORT=8443
SIGNAL_WS_PORT=8080
JWT_SECRET=tu_secreto_jwt
TURN_SERVER=tu_servidor_turn
TURN_USERNAME=tu_usuario_turn
TURN_CREDENTIAL=tu_credencial_turn
`

### Comandos para ejecucion en entorno de Desarrollo

`ash
# Windows (PowerShell)
.\scripts\setup.ps1
make run

# macOS / Linux
chmod +x scripts/dev.sh
./scripts/dev.sh
`

Comandos utiles:

- make run: ejecuta todo el proyecto
- make frontend: solo Flutter
- make kernel: solo kernel Rust
- make backend: solo backend
- make clean: limpia artefactos
- make docker: inicia STUN/TURN mediante Docker Compose

### Comandos para ejecucion en entorno de Produccion

Desplegar el backend en un servidor con Docker Compose, configurar WSS/TLS con certificados SSL validos y ejecutar el frontend Flutter compilado para la plataforma objetivo.

## F. Creditos de Autoría

Desarrollado por: Matthew Ilveneff Garay Perez
Correo Electronico: 1911.garay.perez@gmail.com
