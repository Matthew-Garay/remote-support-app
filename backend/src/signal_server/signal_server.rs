//! SignalServer: maneja conexiones WebSocket y negociación SDP/ICE.
//! Implementación mínima para desarrollo.

use actix_web::{web, Error, HttpResponse};
use actix_ws::{Message, WebSocketContext};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SignalMessage {
    Register { device_id: String, password_hash: String },
    ConnectRequest { target_device_id: String },
    Offer { sdp: String },
    Answer { sdp: String },
    IceCandidate { candidate: String },
    AuthResult { success: bool, message: String },
    Close,
}

#[derive(Debug, Default)]
pub struct SignalServer {
    clients: RwLock<HashMap<String, actix_web::web::Payload>>,
}

impl SignalServer {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn ws_handler(&self, req: actix_web::HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
        let (res, mut session, mut stream) = actix_ws::handle(&req, stream)?;
        // TODO: Implementar manejo de señales reales (register, offer/answer, ICE)
        tracing::info!("Nueva conexión WebSocket desde {}", req.peer_addr().map(|a| a.to_string()).unwrap_or_default());
        Ok(res)
    }
}
