# Script de build para Windows/GNU con llvm-mingw (entorno sin Visual Studio).
# Uso:  powershell -File scripts\build-gnu.ps1 [cargo args...]
$ErrorActionPreference = 'Stop'

$RepoRoot = Split-Path -Parent $PSScriptRoot
$Kernel   = Join-Path $RepoRoot 'kernel'

$LmwBin = 'C:\Users\CONTRALORIA 01\.cline\data\workspaces\chat\_tools\lmw-bin\llvm-mingw-20261006-ucrt-x86_64\bin'
$env:PATH = "$LmwBin;$env:USERPROFILE\.cargo\bin;$env:PATH"

# Toolchain HOST UCRT (stable-x86_64-pc-windows-gnullvm). Necesario porque los
# build scripts se compilan para el host, y el host msvcrt (windows-gnu) exige
# -lgcc / -lgcc_eh, librerias que llvm-mingw (clang/lld) no incluye.
$Toolchain = 'stable-x86_64-pc-windows-gnullvm'

# Compilador C/C++ real (necesario para ring / openh264-sys2).
$env:CC_x86_64_pc_windows_gnullvm  = Join-Path $LmwBin 'x86_64-w64-mingw32-clang.exe'
$env:CXX_x86_64_pc_windows_gnullvm = Join-Path $LmwBin 'x86_64-w64-mingw32-clang++.exe'
$env:AR_x86_64_pc_windows_gnullvm  = Join-Path $LmwBin 'x86_64-w64-mingw32-ar.exe'

# Linker: rustup usa el self-contained (solo enlaza). llvm-mingw aporta crt e
# import libs propios (UCRT), asi que desactivamos los self-contained de rustup
# para evitar librerias duplicadas.
$env:CARGO_TARGET_X86_64_PC_WINDOWS_GNULLVM_LINKER     = Join-Path $LmwBin 'x86_64-w64-mingw32-clang.exe'
$env:CARGO_TARGET_X86_64_PC_WINDOWS_GNULLVM_RUSTFLAGS  = '-C link-self-contained=no'

# El target GNU (msvcrt) no es compatible con llvm-mingw (ucrt): faltan
# -lgcc / -lgcc_eh. Usamos gnullvm, que es el target UCRT nativo de Rust.
$env:CARGO_BUILD_TARGET = 'x86_64-pc-windows-gnullvm'

Push-Location $Kernel
try {
    & "$env:USERPROFILE\.cargo\bin\cargo.exe" +$Toolchain @args
    exit $LASTEXITCODE
} finally {
    Pop-Location
}
