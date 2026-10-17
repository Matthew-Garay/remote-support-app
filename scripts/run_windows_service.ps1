# ============================================
# REMOTE SUPPORT APP - Windows Service Installer
# ============================================
# Instala el kernel Rust como servicio de Windows para ejecutar sin sesión de UI.

param(
    [string]$ServiceName = "RemoteSupportKernel",
    [string]$DisplayName = "Remote Support Kernel",
    [string]$BinaryPath = "$PWD\kernel\target\release\remote-kernel.exe"
)

Write-Host "Instalando servicio de Windows: $ServiceName" -ForegroundColor Cyan

# Verificar que el binario exista
if (-not (Test-Path $BinaryPath)) {
    Write-Host "Binario no encontrado: $BinaryPath" -ForegroundColor Red
    Write-Host "Compila primero con: cargo build --release --manifest-path ./kernel/Cargo.toml" -ForegroundColor Yellow
    exit 1
}

# Crear servicio (requiere privilegios de administrador)
New-Service -Name $ServiceName -BinaryPathName $BinaryPath -DisplayName $DisplayName -StartupType Automatic

Write-Host "Servicio instalado. Inicia con: Start-Service $ServiceName" -ForegroundColor Green
