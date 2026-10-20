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

// ---------------------------------------------------------------------------
// Fase 3: tipos de red (WebRTC + señalización) para el puente FRB.
// ---------------------------------------------------------------------------

/// Configuración de conexión P2P (señalización WebSocket + WebRTC).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConnectConfig {
    /// URL del servidor de señalización (p. ej. `ws://127.0.0.1:8080/ws`).
    pub ws_url: String,
    /// Identificador de este dispositivo (registro en el signal server).
    pub device_id: String,
    /// Hash del password para autenticarse en el signal server.
    pub password_hash: String,
    /// `true` inicia la oferta SDP (Offer); `false` espera (Answer).
    pub offering: bool,
    /// URLs STUN (p. ej. `stun:stun.l.google.com:19302`).
    pub stun_urls: Vec<String>,
    /// URLs TURN (vacío si no hay relés).
    pub turn_urls: Vec<String>,
    /// Usuario TURN (opcional).
    pub turn_username: String,
    /// Credencial TURN (opcional).
    pub turn_credential: String,
}

impl Default for NetworkConnectConfig {
    fn default() -> Self {
        Self {
            ws_url: "ws://127.0.0.1:8080/ws".to_string(),
            device_id: String::new(),
            password_hash: String::new(),
            offering: false,
            stun_urls: vec!["stun:stun.l.google.com:19302".to_string()],
            turn_urls: Vec::new(),
            turn_username: String::new(),
            turn_credential: String::new(),
        }
    }
}

/// Evento de red emitido hacia Flutter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkEvent {
    /// Cambio de estado (`disconnected | connecting | connected | error`).
    ConnectionStateChanged { state: String },
    /// Mensaje entrante del DataChannel (bytes crudos: chat/inputs).
    DataChannelMessage { data: Vec<u8> },
    /// Resultado de la autenticación en el signal server.
    AuthResult { success: bool, message: String },
}

