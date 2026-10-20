//! NetworkEngine: motor de red WebRTC con señalización.
//! Orquesta un `WebRtcTransport` + `SignalingClient` para P2P cifrado.

use crate::network::signaling::SignalingClient;
use crate::network::transport::{Transport, TransportState};
use crate::network::types::{
    NetworkConfig as TypesNetworkConfig, NetworkEvent, NetworkEventKind, PeerConfig,
};
use crate::network::webrtc_transport::WebRtcTransport;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Arc, Mutex};

/// Motor de red: estado 0=Disconnected, 1=Connecting, 2=Connected, 3=Error.
pub struct NetworkEngine {
    transport: Arc<WebRtcTransport>,
    signaling: Arc<SignalingClient>,
    state: Arc<AtomicU8>,
    config: Arc<Mutex<Option<TypesNetworkConfig>>>,
    peer_config: Arc<Mutex<Option<PeerConfig>>>,
    on_event: Arc<Mutex<Option<Box<dyn Fn(NetworkEvent) + Send + Sync>>>>,
}

impl NetworkEngine {
    pub fn new() -> Self {
        let transport = Arc::new(WebRtcTransport::new());
        let signaling = Arc::new(SignalingClient::new(transport.clone()));

        Self {
            transport,
            signaling,
            state: Arc::new(AtomicU8::new(0)),
            config: Arc::new(Mutex::new(None)),
            peer_config: Arc::new(Mutex::new(None)),
            on_event: Arc::new(Mutex::new(None)),
        }
    }

    /// Asigna el callback de eventos de red (DataChannel, estado, auth).
    pub fn set_on_event<F>(&self, cb: F)
    where
        F: Fn(NetworkEvent) + Send + Sync + 'static,
    {
        let cb = Arc::new(cb);
        let cb_engine = cb.clone();
        *self.on_event.lock().unwrap() = Some(Box::new(move |ev| cb_engine(ev)));
        self.signaling.set_on_event(move |ev| cb(ev));
    }

    /// Asigna configuración y cablea los callbacks del transporte hacia
    /// la señalización y los eventos del motor.
    pub fn configure(&self, config: TypesNetworkConfig, peer_config: PeerConfig) {
        self.transport.set_peer_config(peer_config.clone());
        self.transport.set_local_id(config.local_id.clone());
        *self.config.lock().unwrap() = Some(config);
        *self.peer_config.lock().unwrap() = Some(peer_config);

        // offer / answer / candidato ICE → servidor de señalización.
        let sig = self.signaling.clone();
        self.transport.set_on_offer(move |sdp| {
            let sig = sig.clone();
            tokio::spawn(async move {
                if let Err(e) = sig.send_offer(sdp).await {
                    tracing::warn!("send_offer: {e}");
                }
            });
        });

        let sig = self.signaling.clone();
        self.transport.set_on_answer(move |sdp| {
            let sig = sig.clone();
            tokio::spawn(async move {
                if let Err(e) = sig.send_answer(sdp).await {
                    tracing::warn!("send_answer: {e}");
                }
            });
        });

        let sig = self.signaling.clone();
        self.transport.set_on_ice_candidate(move |candidate| {
            let sig = sig.clone();
            tokio::spawn(async move {
                if let Err(e) = sig.send_ice_candidate(candidate).await {
                    tracing::warn!("send_ice_candidate: {e}");
                }
            });
        });

        // Mensajes entrantes del DataChannel → eventos hacia la UI.
        let on_event = self.on_event.clone();
        self.transport.set_on_message(move |data| {
            let ts = now_us();
            if let Some(cb) = on_event.lock().unwrap().as_ref() {
                cb(NetworkEvent {
                    kind: NetworkEventKind::DataChannelMessage,
                    payload: serde_json::json!({ "data": data }),
                    timestamp_us: ts,
                });
            }
        });

        // Estado del transporte → estado atómico + eventos hacia la UI.
        let state = self.state.clone();
        let on_event = self.on_event.clone();
        self.transport.set_on_state_change(move |st| {
            let value = match st {
                TransportState::Disconnected => 0,
                TransportState::Connecting => 1,
                TransportState::Connected => 2,
                TransportState::Error => 3,
            };
            state.store(value, Ordering::SeqCst);
            let ts = now_us();
            if let Some(cb) = on_event.lock().unwrap().as_ref() {
                cb(NetworkEvent {
                    kind: NetworkEventKind::ConnectionStateChanged,
                    payload: serde_json::json!({ "state": st.to_string() }),
                    timestamp_us: ts,
                });
            }
        });
    }

    /// Inicialización (preparado). No conecta todavía.
    pub async fn init(&self) {
        tracing::info!("NetworkEngine listo (WebRTC + señalización)");
    }

    /// Conecta la señalización (WebSocket) y arranca la negociación WebRTC.
    pub async fn connect(
        &self,
        ws_url: String,
        device_id: String,
        password_hash: String,
    ) -> Result<(), String> {
        self.signaling
            .set_credentials(ws_url, device_id, password_hash);
        self.signaling.connect().await?;

        let config = self.config.lock().unwrap().clone().ok_or_else(|| {
            "config no asignada (llame a configure() antes de connect())".to_string()
        })?;

        self.state.store(1, Ordering::SeqCst);
        self.transport
            .start(&config)
            .map_err(|e| format!("transport connect: {e}"))
    }

    /// Pide conectarse a otro dispositivo vía el servidor de señalización.
    pub async fn request_connection(&self, target_device_id: String) -> Result<(), String> {
        self.signaling.request_connection(target_device_id).await
    }

    /// Envía datos por el DataChannel WebRTC.
    pub async fn send_data(&self, data: Vec<u8>) -> Result<usize, String> {
        self.transport
            .send_data(&data)
            .map_err(|e| format!("send_data: {e}"))
    }

    /// Estado actual: `disconnected | connecting | connected | error`.
    pub async fn state(&self) -> String {
        match self.state.load(Ordering::SeqCst) {
            0 => "disconnected",
            1 => "connecting",
            2 => "connected",
            _ => "error",
        }
        .to_string()
    }

    /// Cierra la conexión (transporte WebRTC).
    pub async fn disconnect(&self) -> Result<(), String> {
        self.state.store(0, Ordering::SeqCst);
        self.transport
            .stop()
            .map_err(|e| format!("disconnect: {e}"))
    }
}

impl Default for NetworkEngine {
    fn default() -> Self {
        Self::new()
    }
}

fn now_us() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_micros() as u64)
        .unwrap_or_default()
}
