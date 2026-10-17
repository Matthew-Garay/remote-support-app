//! NetworkEngine: Stub para transporte WebRTC / QUIC.
//! Implementar con:
//! - webrtc-rs para WebRTC PeerConnection + DataChannel
//! - quinn para QUIC (opcional)

#[derive(Debug, Clone)]
pub struct NetworkConfig {
    pub ice_servers: Vec<String>,
    pub stun_server: String,
    pub turn_server: Option<String>,
}

pub struct NetworkEngine;

impl NetworkEngine {
    pub fn new() -> Self {
        Self
    }

    pub async fn init(&mut self) {
        tracing::info!("NetworkEngine::init (stub)");
    }

    pub async fn connect(&mut self, config: NetworkConfig) {
        tracing::info!("NetworkEngine::connect: {:?}", config);
        // TODO: Iniciar negociación WebRTC / QUIC
    }

    pub async fn send_data(&mut self, data: Vec<u8>) {
        tracing::info!("NetworkEngine::send_data: {} bytes", data.len());
    }

    pub async fn state(&self) -> String {
        "disconnected".to_string()
    }
}
