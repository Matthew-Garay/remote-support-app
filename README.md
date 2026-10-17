# Remote Support App

Aplicación de soporte remoto de alto rendimiento (TeamViewer/AnyDesk style).
Arquitectura multi-proceso: Flutter UI + Rust kernel + WebRTC/QUIC + STUN/TURN server.

> **Estado**: Planificación / Inicial. Contiene estructura, archivos base y
> especificaciones de arquitectura. El kernel de captura, codificación y inyección
> de inputs están en fase de diseño/implementación.

---

## 🏗️ Arquitectura

```
┌────────────────────────────────────────────────────────────┐
│                    FLUTTER UI (Frontend)                   │
│  - Device list · Config · Chat · File transfer             │
└───────────────┬────────────────────────────────────────────┘
                │ flutter_rust_bridge (FFI)
┌───────────────▼────────────────────────────────────────────┐
│                  RUST CORE SERVICE                         │
│  Screen Capture (DXGI/PipeWire/SCK) · Input Injector      │
│  Video Encoder (NVENC/QuickSync/VToolbox) · WebRTC/QURL   │
└───────────────┬────────────────────────────────────────────┘
                │ UDP (DTLS/SRTP + XChaCha20-Poly1305)
┌───────────────▼────────────────────────────────────────────┐
│                  BACKEND (Rust / Actix-web / Tokio)        │
│  Signal Server · STUN · TURN · ICE · Autenticación         │
└────────────────────────────────────────────────────────────┘
```

---

## 📁 Estructura de carpetas

```
remote-support-app/
├── README.md
├── LICENSE
├── .gitignore
├── .env.example
├── .vscode/
│   └── settings.json
├── docker-compose.yml
├── Makefile
├── docs/
│   ├── ARCHITECTURE.md
│   ├── SPEC.md
│   ├── SECURITY.md
│   ├── DEPLOYMENT.md
│   └── ROADMAP.md
├── frontend/           # Flutter UI
│   ├── pubspec.yaml
│   ├── lib/
│   │   ├── core/
│   │   │   ├── services/
│   │   │   ├── models/
│   │   │   └── utils/
│   │   ├── features/
│   │   │   ├── devices/ · session/ · chat/ · files/ · settings/
│   │   ├── presentation/
│   │   └── shared/
│   ├── ios/ · android/ · windows/ · macos/
│   └── bin/main.dart
├── kernel/             # Rust kernel
│   ├── Cargo.toml
│   ├── build.rs
│   └── src/
│       ├── lib.rs
│       ├── screen_capture/
│       ├── input_simulator/
│       ├── encoder/
│       ├── network/
│       ├── ffi/
│       └── main.rs
├── backend/            # Servidores
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs
│       ├── signal_server/
│       ├── stun_server/
│       └── turn_server/
└── scripts/
    ├── setup.ps1
    ├── dev.sh
    └── run_windows_service.ps1
```

---

## 🚀 Cómo empezar

### Requisitos
- **Flutter SDK** >= 3.24.0 + Dart >= 3.5.0
- **Rust** >= 1.82
- **Docker** + **Docker Compose** (backend)
- **Git** + **VS Code**

### 1. Clonar y configurar

```bash
git clone https://github.com/tu-usuario/remote-support-app.git
cd remote-support-app

# Copiar variables de entorno
cp .env.example .env
```

### 2. Iniciar

```bash
# Windows (PowerShell)
.\scripts\setup.ps1
make run

# macOS / Linux
chmod +x scripts/dev.sh
./scripts/dev.sh
```

### 3. Comandos útiles

```bash
make run         # Todo el proyecto
make frontend    # Solo Flutter
make kernel      # Solo Rust kernel
make backend     # Solo backend
make clean       # Limpiar artefactos
make docker      # Iniciar STUN/TURN
```

---

## 📚 Documentación

- [Arquitectura](docs/ARCHITECTURE.md)
- [Especificaciones técnicas](docs/SPEC.md)
- [Seguridad](docs/SECURITY.md)
- [Despliegue](docs/DEPLOYMENT.md)
- [Roadmap](docs/ROADMAP.md)

---

## 🌐 Licencia

Propietaria. Contacto: [contact@tudominio.com]

---

> ⚠️ **Precaución**: El acceso remoto a sistemas requiere permisos de nivel
> del sistema. Esta aplicación solicitará permisos explícitos al usuario en
> cada sistema operativo (macOS Accessibility, Windows UAC, Linux Polkit).

---

*Creado por [Tu Nombre/Agencia] · [Fecha]*
