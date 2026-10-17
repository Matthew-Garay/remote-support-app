#!/usr/bin/env bash
set -e

echo "Configurando entorno de desarrollo..."

# Verificar dependencias
command -v flutter >/dev/null 2>&1 || { echo "Flutter no encontrado"; exit 1; }
command -v cargo >/dev/null 2>&1 || { echo "Rust/Cargo no encontrado"; exit 1; }

# Flutter dependencias
echo "Instalando dependencias Flutter..."
flutter pub get

# Rust kernel check
echo "Pre-compilando kernel Rust..."
cargo check --manifest-path ./kernel/Cargo.toml

# Docker compose config
echo "Verificando Docker Compose..."
docker compose config

echo "Entorno configurado correctamente."
