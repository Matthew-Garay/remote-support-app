# Despliegue

## Backend (Signal + STUN/TURN)

Usar Docker Compose:

```bash
make docker
make backends-docker
```

Esto levantará:
- Signal Server (Actix-web) en puerto 8443/8080.
- Coturn STUN/TURN en puertos 3478/5349/49152-65535.

## Frontend

```bash
cd frontend
flutter run
```

## Producción

1. Firmar ejecutables con certificado de confianza.
2. Configurar `.env` con credenciales reales de TURN.
3. Desplegar Signal Server en infraestructura propia (AWS/Azure/GCP).
4. Habilitar HTTPS/TLS en todos los endpoints.
5. Configurar reglas de firewall para puertos STUN/TURN.
