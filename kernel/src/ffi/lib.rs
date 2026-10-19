//! API pública del kernel para Flutter.
//!
//! Fase 2: tipos serializables + `KernelApiImpl` nativo.
//! Fase 5: codegen `flutter_rust_bridge` v2 sobre estos tipos
//! (los tipos ya derivan Serialize/Deserialize y son `Clone`).

use serde::{Deserialize, Serialize};

/// Tipos de codificación de video
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VideoCodec {
    H264,
    H265,
    AV1,
}

/// Resolución de video
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoResolution {
    pub width: u32,
    pub height: u32,
}

/// Métricas de rendimiento
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub fps: f64,
    pub bitrate: u32,
    pub latency_ms: u32,
    pub cpu_usage: f64,
    pub memory_usage: u64,
}

/// Configuración de captura de pantalla
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureConfig {
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub codec: VideoCodec,
}

/// Evento de captura de pantalla
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureEvent {
    pub frame_data: Vec<u8>, // YUV420 o RGB
    pub width: u32,
    pub height: u32,
    pub timestamp: u64,
}

/// Evento de inyección de input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InputEvent {
    MouseMove { x: f64, y: f64 },
    MouseButton { button: MouseButton, pressed: bool },
    Key { key: String, down: bool },
}

/// Botones de mouse
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    X1,
    X2,
}

/// Estado de la captura
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CaptureState {
    Running,
    Paused,
    Stopped,
}

/// Estado del kernel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KernelState {
    Idle,
    Capturing,
    Encoding,
    Networking,
    Ready,
    Error(String),
}

// NOTA Fase 2: el trait `KernelApi` con `#[flutter_rust_bridge::bridge]`
// se genera en Fase 5 con FRB v2 (`flutter_rust_bridge_codegen`).
// La lógica vive en `KernelApiImpl` (ver `impl.rs`).
