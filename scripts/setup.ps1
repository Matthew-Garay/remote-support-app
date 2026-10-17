# ============================================
# REMOTE SUPPORT APP - Setup Windows (PowerShell)
# ============================================

Write-Host "Configurando entorno de desarrollo..." -ForegroundColor Green

# Verificar Flutter
if (-not (Get-Command flutter -ErrorAction SilentlyContinue)) {
    Write-Host "Flutter no encontrado en PATH. Instala Flutter SDK primero." -ForegroundColor Red
    exit 1
}

# Verificar Rust
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "Rust/Cargo no encontrado en PATH. Instala Rust primero." -ForegroundColor Red
    exit 1
}

# Verificar Docker (opcional)
if (-not (Get-Command docker -ErrorAction SilentlyContinue)) {
    Write-Host "Docker no encontrado (opcional para desarrollo local)." -ForegroundColor Yellow
}

# Instalar dependencias Flutter
Write-Host "Instalando dependencias Flutter..." -ForegroundColor Cyan
flutter pub get

# Pre-compilar kernel Rust
Write-Host "Pre-compilando kernel Rust..." -ForegroundColor Cyan
cargo check --manifest-path ./kernel/Cargo.toml

# Verificar Docker Compose
Write-Host "Verificando Docker Compose..." -ForegroundColor Cyan
docker compose config

Write-Host "Entorno configurado correctamente." -ForegroundColor Green
