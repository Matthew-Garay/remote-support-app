//! Capa de transporte abstracta.
//!
//! Fase 3: `Transport` es la interfaz de la que se beneficia
//! `NetworkEngine`. Cualquier nuevo protocolo (WebRTC, QUIC, UDP, etc.) solo
//! necesita implementar este trait.

use crate::network::types::Protocol;

pub trait Transport: Send + Sync {
    /// Nombre corto del transporte (p. ej. `udp`, `webrtc`, `quic`).
    fn name(&self) -> &'static str;

    /// Protocolo declarado por este transporte.
    fn protocol(&self) -> Protocol;

    /// Envía datos a la capa de aplicación del peer remoto.
    fn send_data(&self, data: &[u8]) -> Result<usize, TransportError>;

    /// Inicia la conexión hacia el peer descrito por `config`.
    fn connect(&mut self, config: &crate::network::types::NetworkConfig) -> Result<(), TransportError>;

    /// Cierra la transporte (deja la mitad de la negociación para el caller).
    fn disconnect(&mut self) -> Result<(), TransportError>;

    /// Estado actual del transporte.
    fn state(&self) -> TransportState;

    /// `true` si hay una ruta viable hacia el peer.
    fn is_connected(&self) -> bool {
        matches!(self.state(), TransportState::Connected)
    }

    /// Identificador local del peer (p. ej. `udp:127.0.0.1:5000`).
    fn local_id(&self) -> Option<String> {
        None
    }

    /// Identificador remoto del peer (resuelto tras ICE).
    fn remote_id(&self) -> Option<String> {
        None
    }
}

/// Estado observable del transporte.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportState {
    Disconnected,
    Connecting,
    Connected,
    Error,
}

impl std::fmt::Display for TransportState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Disconnected => write!(f, "disconnected"),
            Self::Connecting => write!(f, "connecting"),
            Self::Connected => write!(f, "connected"),
            Self::Error => write!(f, "error"),
        }
    }
}

/// Traducto de errores de `Transport` para `NetworkEngine`.
#[derive(Debug, thiserror::Error)]
pub enum TransportError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),

    #[error("peer no encontrado: {0}")]
    PeerNotFound(String),

    #[error("transporte no disponible: {0}")]
    Unavailable(String),

    #[error("cierre inesperado: {0}")]
    Unexpected(String),

    #[error("protocolo no soportado: {0:?}")]
    InvalidProtocol(Protocol),
}

/// Convierte un `u8` simple en `TransportState` (atomic/serde-friendly).
pub(crate) fn from_u8(value: u8) -> TransportState {
    match value {
        0 => TransportState::Disconnected,
        1 => TransportState::Connecting,
        2 => TransportState::Connected,
        _ => TransportState::Error,
    }
}
