//! FFI API for flutter_rust_bridge
//!
//! Esta es la API pública que Dart puede llamar. Todas las funciones aquí
//! deben tener el atributo #[flutter_rust_bridge::bridge].

use flutter_rust_bridge::Stream;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

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

/// API principal del kernel
#[flutter_rust_bridge::bridge]
pub trait KernelApi {
    // === Captura de pantalla ===

    /// Iniciar captura de pantalla
    fn start_capture(&mut self, config: CaptureConfig) -> Stream<CaptureEvent>;

    /// Detener captura de pantalla
    fn stop_capture(&mut self) -> CaptureState;

    /// Obtener estado actual de la captura
    fn capture_state(&self) -> CaptureState;

    // === Codificación de video ===

    /// Configurar codificación de video
    fn set_encoder_config(&mut self, codec: VideoCodec, bitrate: u32) -> Result<(), String>;

    /// Obtener métricas del encoder
    fn encoder_metrics(&self) -> PerformanceMetrics;

    // === Inyección de input (control remoto) ===

    /// Inyectar evento de mouse
    fn inject_mouse_event(&self, event: InputEvent) -> Result<(), String>;

    /// Inyectar evento de teclado
    fn inject_key_event(&self, event: InputEvent) -> Result<(), String>;

    // === Estado del kernel ===

    /// Obtener estado del kernel
    fn kernel_state(&self) -> KernelState;

    /// Reinciar el kernel (para pruebas)
    fn restart(&mut self) -> Result<(), String>;
}
