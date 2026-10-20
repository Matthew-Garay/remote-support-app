//! Cliente de señalización WebSocket.
//!
//! Conecta al `SignalServer` (actix-web + actix-ws) y reenvía mensajes
//! de señalización (SDP offer/answer, candidatos ICE) entre el WebSocket
//! y el `WebRtcTransport`.

use crate::network::types::{NetworkEvent, NetworkEventKind};
use crate::network::webrtc_transport::WebRtcTransport;
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message as WsMessage};

/// Mensajes de señalización (mismo protocolo que `backend/src/signal_server/signal_server.rs`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SignalMessage {
    Register {
        device_id: String,
        password_hash: String,
    },
    ConnectRequest {
        target_device_id: String,
    },
    Offer {
        sdp: String,
    },
    Answer {
        sdp: String,
    },
    IceCandidate {
        candidate: String,
    },
    AuthResult {
        success: bool,
        message: String,
    },
    Close,
}

/// Cliente de señalización WebSocket.
///
/// Diseño: `connect()` divide el stream en lectura/escritura y lanza dos
/// tareas — la lectora despacha mensajes entrantes hacia el `WebRtcTransport`
/// y la escritora consume un canal `mpsc` por el que `send_signal()` publica.
pub struct SignalingClient {
    ws_url: Mutex<String>,
    device_id: Mutex<String>,
    password_hash: Mutex<String>,
    transport: Arc<WebRtcTransport>,
    tx: Mutex<Option<mpsc::Sender<String>>>,
    on_event: Arc<Mutex<Option<Box<dyn Fn(NetworkEvent) + Send + Sync>>>>,
}

impl SignalingClient {
    /// Crea el cliente (credenciales vacías: llamar a `set_credentials` antes de `connect`).
    pub fn new(transport: Arc<WebRtcTransport>) -> Self {
        Self {
            ws_url: Mutex::new("ws://127.0.0.1:8080/ws".to_string()),
            device_id: Mutex::new(String::new()),
            password_hash: Mutex::new(String::new()),
            transport,
            tx: Mutex::new(None),
            on_event: Arc::new(Mutex::new(None)),
        }
    }

    /// Asigna URL/credenciales antes de `connect()`.
    pub fn set_credentials(&self, ws_url: String, device_id: String, password_hash: String) {
        *self.ws_url.lock().unwrap() = ws_url;
        *self.device_id.lock().unwrap() = device_id;
        *self.password_hash.lock().unwrap() = password_hash;
    }

    /// Callback para eventos de red generados por la señalización (auth, etc.).
    pub fn set_on_event<F>(&self, cb: F)
    where
        F: Fn(NetworkEvent) + Send + Sync + 'static,
    {
        *self.on_event.lock().unwrap() = Some(Box::new(cb));
    }

    /// Conecta al servidor, lanza las tareas reader/writer y envía `Register`.
    pub async fn connect(&self) -> Result<(), String> {
        let url = self.ws_url.lock().unwrap().clone();
        let (ws_stream, _) = connect_async(&url)
            .await
            .map_err(|e| format!("conexión WebSocket {url}: {e}"))?;
        let (write, read) = ws_stream.split();
        let (tx, mut rx) = mpsc::channel::<String>(64);

        // Tarea escritora: consume el canal y escribe al WebSocket.
        tokio::spawn(async move {
            let mut write = write;
            while let Some(text) = rx.recv().await {
                if write.send(WsMessage::Text(text)).await.is_err() {
                    break;
                }
            }
        });

        // Tarea lectora: despacha los mensajes entrantes hacia el transporte.
        let transport = self.transport.clone();
        let on_event = self.on_event.clone();
        tokio::spawn(async move {
            let mut read = read;
            while let Some(msg) = read.next().await {
                match msg {
                    Ok(WsMessage::Text(text)) => {
                        Self::dispatch(&transport, &on_event, &text).await
                    }
                    Ok(WsMessage::Close(_)) => break,
                    Err(e) => {
                        tracing::warn!("signal reader: {e}");
                        break;
                    }
                    _ => {}
                }
            }
            tracing::info!("signal: WebSocket cerrado");
        });

        *self.tx.lock().unwrap() = Some(tx);

        let device_id = self.device_id.lock().unwrap().clone();
        let password_hash = self.password_hash.lock().unwrap().clone();
        self.send_signal(SignalMessage::Register {
            device_id,
            password_hash,
        })
        .await
    }

    /// Despacha un mensaje de señalización entrante hacia el transporte.
    async fn dispatch(
        transport: &Arc<WebRtcTransport>,
        on_event: &Arc<Mutex<Option<Box<dyn Fn(NetworkEvent) + Send + Sync>>>>,
        text: &str,
    ) {
        let msg: SignalMessage = match serde_json::from_str(text) {
            Ok(m) => m,
            Err(e) => {
                tracing::warn!("signal: mensaje no parseado ({e}): {text}");
                return;
            }
        };
        match msg {
            SignalMessage::Offer { sdp } => {
                if let Err(e) = transport.set_remote_description(sdp, "offer").await {
                    tracing::warn!("signal offer: {e}");
                }
            }
            SignalMessage::Answer { sdp } => {
                if let Err(e) = transport.set_remote_description(sdp, "answer").await {
                    tracing::warn!("signal answer: {e}");
                }
            }
            SignalMessage::IceCandidate { candidate } => {
                if let Err(e) = transport.add_ice_candidate(candidate).await {
                    tracing::warn!("signal ice: {e}");
                }
            }
            SignalMessage::AuthResult { success, message } => {
                tracing::info!("signal auth: success={success} msg={message}");
                let ts = now_us();
                if let Some(cb) = on_event.lock().unwrap().as_ref() {
                    cb(NetworkEvent {
                        kind: NetworkEventKind::ConnectionStateChanged,
                        payload: serde_json::json!({ "auth": success, "message": message }),
                        timestamp_us: ts,
                    });
                }
            }
            SignalMessage::Close => tracing::info!("signal: servidor pidió cierre"),
            _ => {}
        }
    }

    /// Serializa y envía un mensaje de señalización por el canal del writer.
    async fn send_signal(&self, msg: SignalMessage) -> Result<(), String> {
        let json = serde_json::to_string(&msg).map_err(|e| e.to_string())?;
        let tx = self.tx.lock().unwrap().clone();
        let tx = tx.ok_or_else(|| "WebSocket no conectado".to_string())?;
        tx.send(json).await.map_err(|e| format!("ws send: {e}"))
    }

    /// Envía un `Offer` (llamado desde el callback `on_offer` del transporte).
    pub async fn send_offer(&self, sdp: String) -> Result<(), String> {
        self.send_signal(SignalMessage::Offer { sdp }).await
    }

    /// Envía un `Answer` (llamado desde el callback `on_answer` del transporte).
    pub async fn send_answer(&self, sdp: String) -> Result<(), String> {
        self.send_signal(SignalMessage::Answer { sdp }).await
    }

    /// Envía un candidato ICE (llamado desde `on_ice_candidate`).
    pub async fn send_ice_candidate(&self, candidate: String) -> Result<(), String> {
        self.send_signal(SignalMessage::IceCandidate { candidate }).await
    }

    /// Pide conectarse a otro dispositivo (`ConnectRequest`).
    pub async fn request_connection(&self, target_device_id: String) -> Result<(), String> {
        self.send_signal(SignalMessage::ConnectRequest {
            target_device_id,
        })
        .await
    }
}

fn now_us() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_micros() as u64)
        .unwrap_or_default()
}
