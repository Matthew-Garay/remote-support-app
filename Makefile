# ============================================
# REMOTE SUPPORT APP - Makefile
# ============================================

ROOT_DIR := $(shell pwd)
FRONTEND_DIR := $(ROOT_DIR)/frontend
KERNEL_DIR := $(ROOT_DIR)/kernel
BACKEND_DIR := $(ROOT_DIR)/backend

GREEN  := \033[0;32m
YELLOW := \033[1;33m
RED    := \033[0;31m
NC     := \033[0m

.PHONY: help run frontend kernel backend docker clean setup test docs

help: ## Show this help
	@echo "$(GREEN)Remote Support App - Commands$(NC)"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "  $(YELLOW)%-15s$(NC) %s\n", $$1, $$2}'

# ============================================
# Run - Todo el proyecto
# ============================================
run: ## Iniciar todo (UI + Kernel + Backend)
	@echo "$(GREEN)Iniciando proyecto completo...$(NC)"
	@cd $(BACKEND_DIR) && cargo run &
	@sleep 3
	@cd $(FRONTEND_DIR) && flutter run

# ============================================
# Frontend - Solo Flutter
# ============================================
frontend: ## Iniciar solo Flutter
	@cd $(FRONTEND_DIR) && flutter run

frontend-release: ## Frontend en modo release
	@cd $(FRONTEND_DIR) && flutter run --release

frontend-windows: ## Compilar Windows
	@cd $(FRONTEND_DIR) && flutter build windows --release

frontend-macos: ## Compilar macOS
	@cd $(FRONTEND_DIR) && flutter build macos

frontend-android: ## Compilar Android
	@cd $(FRONTEND_DIR) && flutter build apk --release

# ============================================
# Kernel - Solo Rust
# ============================================
kernel: ## Iniciar solo Kernel Rust
	@cd $(KERNEL_DIR) && cargo run --release

kernel-release: ## Kernel modo release
	@cd $(KERNEL_DIR) && cargo build --release

kernel-screen-capture: ## Demo de captura de pantalla
	@cd $(KERNEL_DIR) && cargo run --example screen_capture --release

# ============================================
# Backend - Solo Servidor
# ============================================
backend: ## Iniciar solo Backend
	@cd $(BACKEND_DIR) && cargo run --release

backend-debug: ## Backend modo debug
	@cd $(BACKEND_DIR) && cargo run

backends-docker: ## Backend con Docker Compose
	@docker compose -f $(ROOT_DIR)/docker-compose.yml up -d

backends-docker-down: ## Detener backend Docker
	@docker compose -f $(ROOT_DIR)/docker-compose.yml down

backends-docker-logs: ## Ver logs backend Docker
	@docker compose -f $(ROOT_DIR)/docker-compose.yml logs -f

# ============================================
# Docker - ICE/STUN/TURN
# ============================================
docker: ## Iniciar STUN/TURN Docker
	@docker compose -f $(ROOT_DIR)/docker-compose.yml up -d

docker-down: ## Detener STUN/TURN
	@docker compose -f $(ROOT_DIR)/docker-compose.yml down

docker-logs: ## Ver logs STUN/TURN
	@docker compose -f $(ROOT_DIR)/docker-compose.yml logs -f

# ============================================
# Build - Compilación
# ============================================
build: build-frontend build-kernel build-backend ## Compilar todo

build-frontend: ## Compilar frontend
	@cd $(FRONTEND_DIR) && flutter build windows --release

build-kernel: ## Compilar kernel (release)
	@cd $(KERNEL_DIR) && cargo build --release

build-backend: ## Compilar backend (release)
	@cd $(BACKEND_DIR) && cargo build --release

# ============================================
# Clean - Limpieza
# ============================================
clean: ## Limpiar artefactos
	@echo "$(RED)Limpiando...$(NC)"
	@cd $(FRONTEND_DIR) && flutter clean
	@cd $(KERNEL_DIR) && cargo clean
	@cd $(BACKEND_DIR) && cargo clean
	@echo "$(GREEN)Limpieza completada.$(NC)"

clean-docker: ## Limpiar contenedores Docker
	@docker compose -f $(ROOT_DIR)/docker-compose.yml down --volumes

# ============================================
# Setup - Inicialización
# ============================================
setup: ## Configurar entorno
	@echo "$(GREEN)Configurando entorno...$(NC)"
	@which flutter || echo "Flutter no encontrado"
	@which cargo || echo "Rust no encontrado"
	@which docker || echo "Docker no encontrado"
	@cd $(FRONTEND_DIR) && flutter pub get
	@cd $(KERNEL_DIR) && cargo check
	@docker compose -f $(ROOT_DIR)/docker-compose.yml config
	@echo "$(GREEN)Entorno listo.$(NC)"

setup-windows: ## Configurar Windows
	@powershell -executionpolicy bypass -File scripts/setup.ps1

# ============================================
# Test - Pruebas
# ============================================
test: test-frontend test-kernel ## Ejecutar pruebas

test-frontend: ## Pruebas frontend
	@cd $(FRONTEND_DIR) && flutter test

test-kernel: ## Pruebas kernel
	@cd $(KERNEL_DIR) && cargo test

test-backend: ## Pruebas backend
	@cd $(BACKEND_DIR) && cargo test

# ============================================
# Docs - Documentación
# ============================================
docs: ## Abrir README.md
	@start "" "file://$(ROOT_DIR)/README.md"

docs-server: ## Servir docs local
	@python -m http.server 8000 -d $(ROOT_DIR)/docs
	@echo "Documentación en http://localhost:8000"

# ============================================
# Tools - Utilidades
# ============================================
gen-ffi: ## Generar código FFI
	@echo "$(GREEN)Generando FFI...$(NC)"
	@cd $(KERNEL_DIR) && cargo run --bin generate-bridge
