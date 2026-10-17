# Script para finalizar git dentro de remote-support-app
Push-Location -LiteralPath 'C:\Users\CONTRALORIA 01\.cline\data\workspaces\chat\remote-support-app'

# Eliminar script temporal
Remove-Item -Force 'init_git.ps1' -ErrorAction SilentlyContinue

# Agregar cambios y hacer commit
git add -A
git commit -m "chore: eliminar script temporal de inicialización"

# Mostrar estado final
git status
git log --oneline -3

Pop-Location
