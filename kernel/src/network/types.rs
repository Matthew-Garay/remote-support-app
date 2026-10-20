//! Tipos de red independientes del transporte.
//!
//! Fase 3: DTOs que el `NetworkEngine` y los `Transport` comparten. No
//! dependen de `webrtc` ni de `quinn`, por lo que el compilado por defecto
//! permanece ligero.

use serde::{Deserialize, Serialize};

/// Protocolo de transporte seleccionado por la configuración.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Protocol {
    /// Transporte local de prueba (UDP en memoria / loopback).
    #[default]
    Local,
    /// WebRTC P2P cifrado (webrtc-rs 0.13, con feature `webrtc-p2p`).
    WebRTC,
    /// QUIC (quinn 0.11, con feature `quic`).
    QUIC,
}

/// Servidor STUN/TURN descrito para la negociación ICE.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IceServer {
    /// URLs (stun:, turn:, turns:).
    pub urls: Vec<String>,
    /// Nombre de usuario para TURN.
    pub username: Option<String>,
    /// Credencial de TURN.
    pub credential: Option<String>,
    /// Tipo de credencial (opcional, p. ej. `OAuth`).
    pub credential_type: Option<String>,
}

/// Configuración de un `Peer` al conectar un `NetworkEngine`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerConfig {
    /// `Offer` (el que inicia) o `Answer` (el que responde).
    pub role: Role,
    /// Protocolo de capa de aplicación.
    pub protocol: Protocol,
    /// URLs STUN/TURN para la fase de ICE.
    pub ice_servers: Vec<IceServer>,
    /// Codec preferido para el flujo de video.
    pub preferred_codec: String,
    /// Estimación de banda disponible en el momento de la conexión.
    pub bandwidth_estimate_bps: Option<u32>,
}

impl Default for PeerConfig {
    fn default() -> Self {
        Self {
            role: Role::Answer,
            protocol: Protocol::Local,
            ice_servers: Vec::new(),
            preferred_codec: "h264".to_string(),
            bandwidth_estimate_bps: Some(1_000_000),
        }
    }
}

/// Rol del `Peer` dentro de la sesión.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Role {
    /// El peer inicia la negociación (`createOffer`).
    Offer,
    /// El peer responde (`createAnswer`).
    Answer,
}

impl Default for Role {
    fn default() -> Self {
        Self::Answer
    }
}

/// Estado interno del motor de red.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum NetworkState {
    #[default]
    Disconnected,
    Connecting,
    Connected,
    Closed,
}

/// Tipo de evento que el `NetworkEngine` publica a sus suscriptores.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NetworkEventKind {
    InputReady,
    ChatMessage,
    VideoFrame,
    BandwidthUpdate,
    IceCandidate,
    DataChannelMessage,
    ConnectionStateChanged,
}

/// Evento genérico del motor de red.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkEvent {
    pub kind: NetworkEventKind,
    pub payload: serde_json::Value,
    pub timestamp_us: u64,
}

/// Configuración concreta para conectar un `NetworkEngine`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Identificador local del peer (logging, journal).
    pub local_id: String,
    /// Dirección del único peer de la sesión (UDP/QUIC).
    pub peer_address: Option<std::net::SocketAddr>,
    /// Protocolo de transporte.
    pub protocol: Protocol,
    /// Estimación de banda disponible (bps), informada por el ABR.
    pub bandwidth_estimate_bps: Option<u32>,
}
