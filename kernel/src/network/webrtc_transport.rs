//! WebRTC transporte P2P sobre `webrtc-0.13`.
//! Implementa el trait [`Transport`] usando `RTCPeerConnection` + `RTCDataChannel`.

use crate::network::types::{PeerConfig, Protocol, Role};
use crate::network::transport::{Transport, TransportError, TransportState};
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Arc, Mutex};
use tracing;
use webrtc::api::APIBuilder;
use serde_json;
use webrtc::api::setting_engine::SettingEngine;
use webrtc::data_channel::data_channel_init::RTCDataChannelInit;
use webrtc::data_channel::data_channel_message::DataChannelMessage;
use webrtc::data_channel::RTCDataChannel;
use webrtc::ice_transport::ice_candidate::{RTCIceCandidate, RTCIceCandidateInit};
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState;
use webrtc::peer_connection::RTCPeerConnection;

const DATA_CHANNEL_LABEL: &str = "remote-support";

pub struct WebRtcTransport {
    pc: Arc<Mutex<Option<Arc<RTCPeerConnection>>>>,
    dc: Arc<Mutex<Option<Arc<RTCDataChannel>>>>,
    config: Arc<Mutex<Option<PeerConfig>>>,
    state: Arc<AtomicU8>,
    local_id: Arc<Mutex<Option<String>>>,
    remote_id: Arc<Mutex<Option<String>>>,
    on_message: Arc<Mutex<Option<Box<dyn Fn(Vec<u8>) + Send + Sync>>>>,
    on_state_change: Arc<Mutex<Option<Box<dyn Fn(TransportState) + Send + Sync>>>>,
    on_offer: Arc<Mutex<Option<Box<dyn Fn(String) + Send + Sync>>>>,
    on_answer: Arc<Mutex<Option<Box<dyn Fn(String) + Send + Sync>>>>,
    on_ice_candidate: Arc<Mutex<Option<Box<dyn Fn(String) + Send + Sync>>>>,
}

impl Default for WebRtcTransport {
    fn default() -> Self {
        Self::new()
    }
}

impl WebRtcTransport {
    pub fn new() -> Self {
        Self {
            pc: Arc::new(Mutex::new(None)),
            dc: Arc::new(Mutex::new(None)),
            config: Arc::new(Mutex::new(None)),
            state: Arc::new(AtomicU8::new(TransportState::Disconnected as u8)),
            local_id: Arc::new(Mutex::new(None)),
            remote_id: Arc::new(Mutex::new(None)),
            on_message: Arc::new(Mutex::new(None)),
            on_state_change: Arc::new(Mutex::new(None)),
            on_offer: Arc::new(Mutex::new(None)),
            on_answer: Arc::new(Mutex::new(None)),
            on_ice_candidate: Arc::new(Mutex::new(None)),
        }
    }

    pub fn set_peer_config(&self, config: PeerConfig) {
        *self.config.lock().unwrap() = Some(config);
    }

    pub fn set_local_id(&self, id: String) {
        *self.local_id.lock().unwrap() = Some(id);
    }

    /// Equivalente a [`Transport::connect`] pero usable desde un `Arc<Self>`.
    pub fn start(&self, config: &crate::network::types::NetworkConfig) -> Result<(), TransportError> {
        self.set_local_id(config.local_id.clone());

        let handle = tokio::runtime::Handle::try_current().map_err(|_| {
            TransportError::Unexpected("no hay runtime Tokio disponible para connect".into())
        })?;

        let state = self.state.clone();
        let on_state_change = self.on_state_change.clone();
        let me = self.share();

        handle.spawn(async move {
            state.store(TransportState::Connecting as u8, Ordering::SeqCst);
            if let Some(cb) = on_state_change.lock().unwrap().as_ref() {
                cb(TransportState::Connecting);
            }
            if let Err(e) = me.negotiate().await {
                tracing::error!("WebRTC negotiate: {e}");
                state.store(TransportState::Error as u8, Ordering::SeqCst);
                if let Some(cb) = on_state_change.lock().unwrap().as_ref() {
                    cb(TransportState::Error);
                }
            }
        });

        Ok(())
    }

    /// Equivalente a [`Transport::disconnect`] pero usable desde un `Arc<Self>`.
    pub fn stop(&self) -> Result<(), TransportError> {
        let handle = tokio::runtime::Handle::try_current().map_err(|_| {
            TransportError::Unexpected("no hay runtime Tokio disponible para disconnect".into())
        })?;

        let pc = self.pc.lock().unwrap().clone();
        let state = self.state.clone();
        let on_state_change = self.on_state_change.clone();

        handle.spawn(async move {
            if let Some(pc) = pc {
                let _ = pc.close().await;
            }
            state.store(TransportState::Disconnected as u8, Ordering::SeqCst);
            if let Some(cb) = on_state_change.lock().unwrap().as_ref() {
                cb(TransportState::Disconnected);
            }
        });

        *self.pc.lock().unwrap() = None;
        *self.dc.lock().unwrap() = None;

        Ok(())
    }

    pub fn set_on_message<F>(&self, cb: F)
    where
        F: Fn(Vec<u8>) + Send + Sync + 'static,
    {
        *self.on_message.lock().unwrap() = Some(Box::new(cb));
    }

    pub fn set_on_state_change<F>(&self, cb: F)
    where
        F: Fn(TransportState) + Send + Sync + 'static,
    {
        *self.on_state_change.lock().unwrap() = Some(Box::new(cb));
    }

    pub fn set_on_offer<F>(&self, cb: F)
    where
        F: Fn(String) + Send + Sync + 'static,
    {
        *self.on_offer.lock().unwrap() = Some(Box::new(cb));
    }

    pub fn set_on_answer<F>(&self, cb: F)
    where
        F: Fn(String) + Send + Sync + 'static,
    {
        *self.on_answer.lock().unwrap() = Some(Box::new(cb));
    }

    pub fn set_on_ice_candidate<F>(&self, cb: F)
    where
        F: Fn(String) + Send + Sync + 'static,
    {
        *self.on_ice_candidate.lock().unwrap() = Some(Box::new(cb));
    }

    /// Crea un manejador que comparte las mismas `Arc` (para spawn 'static).
    pub fn share(&self) -> Self {
        Self {
            pc: self.pc.clone(),
            dc: self.dc.clone(),
            config: self.config.clone(),
            state: self.state.clone(),
            local_id: self.local_id.clone(),
            remote_id: self.remote_id.clone(),
            on_message: self.on_message.clone(),
            on_state_change: self.on_state_change.clone(),
            on_offer: self.on_offer.clone(),
            on_answer: self.on_answer.clone(),
            on_ice_candidate: self.on_ice_candidate.clone(),
        }
    }

    pub fn local_id(&self) -> Option<String> {
        self.local_id.lock().unwrap().clone()
    }

    pub fn remote_id(&self) -> Option<String> {
        self.remote_id.lock().unwrap().clone()
    }

    pub async fn set_remote_description(&self, sdp: String, sdp_type: &str) -> Result<(), TransportError> {
        let pc = self.pc.lock().unwrap().clone().ok_or_else(|| {
            TransportError::Unavailable("PeerConnection no inicializado".into())
        })?;

        let desc = match sdp_type {
            "offer" => RTCSessionDescription::offer(sdp),
            "answer" => RTCSessionDescription::answer(sdp),
            _ => return Err(TransportError::InvalidProtocol(Protocol::WebRTC)),
        }
        .map_err(|e| TransportError::Unexpected(format!("sdp: {e}")))?;

        pc.set_remote_description(desc).await.map_err(|e| {
            TransportError::Unexpected(format!("set_remote_description: {e}"))
        })?;

        if sdp_type == "offer" {
            let config = self.config.lock().unwrap().clone();
            if let Some(PeerConfig { role: Role::Answer, .. }) = config {
                let on_answer = self.on_answer.clone();
                let pc_clone = pc.clone();
                let on_state_change = self.on_state_change.clone();
                let state = self.state.clone();

                tokio::spawn(async move {
                    match pc_clone.create_answer(None).await {
                        Ok(answer) => {
                            let sdp = answer.sdp.clone();
                            if let Err(e) = pc_clone.set_local_description(answer.clone()).await {
                                tracing::error!("set_local_description (answer): {e}");
                            } else {
                                if let Some(cb) = on_answer.lock().unwrap().as_ref() {
                                    cb(sdp);
                                }
                            }
                        }
                        Err(e) => tracing::error!("create_answer: {e}"),
                    }
                    state.store(TransportState::Connecting as u8, Ordering::SeqCst);
                    if let Some(cb) = on_state_change.lock().unwrap().as_ref() {
                        cb(TransportState::Connecting);
                    }
                });
            }
        }

        Ok(())
    }

    pub async fn add_ice_candidate(&self, candidate_json: String) -> Result<(), TransportError> {
        let pc = self.pc.lock().unwrap().clone().ok_or_else(|| {
            TransportError::Unavailable("PeerConnection no inicializado".into())
        })?;

        let init: RTCIceCandidateInit = serde_json::from_str(&candidate_json)
            .map_err(|e| TransportError::Unexpected(format!("parse ICE candidate: {e}")))?;

        pc.add_ice_candidate(init).await.map_err(|e| {
            TransportError::Unexpected(format!("add_ice_candidate: {e}"))
        })?;

        Ok(())
    }

    pub async fn close(&self) -> Result<(), TransportError> {
        let pc = self.pc.lock().unwrap().take();
        if let Some(pc) = pc {
            pc.close()
                .await
                .map_err(|e| TransportError::Unexpected(format!("close: {e}")))?;
        }
        *self.dc.lock().unwrap() = None;
        self.state.store(TransportState::Disconnected as u8, Ordering::SeqCst);
        Ok(())
    }

    async fn negotiate_part1(&self) -> Result<Arc<RTCPeerConnection>, TransportError> {
        let config = self
            .config
            .lock()
            .unwrap()
            .clone()
            .ok_or_else(|| TransportError::Unavailable("PeerConfig no asignado".into()))?;

        let mut se = SettingEngine::default();
        se.set_include_loopback_candidate(true);

        let api = APIBuilder::new().with_setting_engine(se).build();

        let ice_servers: Vec<RTCIceServer> = config
            .ice_servers
            .iter()
            .map(|s| RTCIceServer {
                urls: s.urls.clone(),
                username: s.username.clone().unwrap_or_default(),
                credential: s.credential.clone().unwrap_or_default(),
            })
            .collect();

        let rtc_config = RTCConfiguration {
            ice_servers,
            ..Default::default()
        };

        api.new_peer_connection(rtc_config)
            .await
            .map(Arc::new)
            .map_err(|e| TransportError::Unexpected(format!("new_peer_connection: {e}")))
    }

    async fn negotiate(&self) -> Result<(), TransportError> {
        let config = self
            .config
            .lock()
            .unwrap()
            .clone()
            .ok_or_else(|| TransportError::Unavailable("PeerConfig no asignado".into()))?;

        let pc = self.negotiate_part1().await?;
        let pc_for_dc = pc.clone();

        let dc_ref = self.dc.clone();
        let on_message = self.on_message.clone();
        let on_state_change = self.on_state_change.clone();
        let state = self.state.clone();

        pc.on_data_channel(Box::new(move |dc: Arc<RTCDataChannel>| {
            let dc_ref = dc_ref.clone();
            let on_message = on_message.clone();
            Box::pin(async move {
                *dc_ref.lock().unwrap() = Some(dc.clone());

                let on_message_cb = on_message.clone();
                dc.on_message(Box::new(move |msg: DataChannelMessage| {
                    let on_message_cb = on_message_cb.clone();
                    Box::pin(async move {
                        let data: Vec<u8> = msg.data.to_vec();
                        if let Some(cb) = on_message_cb.lock().unwrap().as_ref() {
                            cb(data);
                        }
                    })
                }));
            })
        }));

        // ---- Candidatos ICE → señalización ----
        let on_ice_candidate = self.on_ice_candidate.clone();
        pc.on_ice_candidate(Box::new(move |candidate: Option<RTCIceCandidate>| {
            let on_ice_candidate = on_ice_candidate.clone();
            Box::pin(async move {
                if let Some(c) = candidate {
                    match c.to_json() {
                        Ok(init) => {
                            if let Ok(json) = serde_json::to_string(&init) {
                                if let Some(cb) = on_ice_candidate.lock().unwrap().as_ref() {
                                    cb(json);
                                }
                            }
                        }
                        Err(e) => tracing::warn!("ICE candidate to_json: {e}"),
                    }
                }
            })
        }));

        pc.on_peer_connection_state_change(Box::new(move |ps: RTCPeerConnectionState| {
            let on_state_change = on_state_change.clone();
            let state = state.clone();
            Box::pin(async move {
                let new_state = match ps {
                    RTCPeerConnectionState::Connected => TransportState::Connected,
                    RTCPeerConnectionState::Connecting => TransportState::Connecting,
                    RTCPeerConnectionState::New => TransportState::Connecting,
                    RTCPeerConnectionState::Disconnected => TransportState::Disconnected,
                    RTCPeerConnectionState::Failed => TransportState::Error,
                    RTCPeerConnectionState::Closed => TransportState::Disconnected,
                    _ => TransportState::Disconnected,
                };
                state.store(new_state as u8, Ordering::SeqCst);
                if let Some(cb) = on_state_change.lock().unwrap().as_ref() {
                    cb(new_state);
                }
            })
        }));

        let dc_init = RTCDataChannelInit {
            ordered: Some(true),
            negotiated: None,
            ..Default::default()
        };
        let dc = pc_for_dc
            .create_data_channel(DATA_CHANNEL_LABEL, Some(dc_init))
            .await
            .map_err(|e| TransportError::Unexpected(format!("create_data_channel: {e}")))?;
        *self.dc.lock().unwrap() = Some(dc.clone());

        let on_message_cb = self.on_message.clone();
        dc.on_message(Box::new(move |msg: DataChannelMessage| {
            let on_message_cb = on_message_cb.clone();
            Box::pin(async move {
                let data: Vec<u8> = msg.data.to_vec();
                if let Some(cb) = on_message_cb.lock().unwrap().as_ref() {
                    cb(data);
                }
            })
        }));

        *self.pc.lock().unwrap() = Some(pc);
        self.state.store(TransportState::Connecting as u8, Ordering::SeqCst);

        if matches!(config.role, Role::Offer) {
            let pc_clone = self.pc.lock().unwrap().clone().unwrap();
            let on_offer = self.on_offer.clone();
            let on_state_change = self.on_state_change.clone();
            let state = self.state.clone();

            tokio::spawn(async move {
                match pc_clone.create_offer(None).await {
                    Ok(offer) => {
                        let sdp = offer.sdp.clone();
                        if let Err(e) = pc_clone.set_local_description(offer.clone()).await {
                            tracing::error!("set_local_description (offer): {e}");
                        } else {
                            if let Some(cb) = on_offer.lock().unwrap().as_ref() {
                                cb(sdp);
                            }
                        }
                    }
                    Err(e) => tracing::error!("create_offer: {e}"),
                }
                state.store(TransportState::Connecting as u8, Ordering::SeqCst);
                if let Some(cb) = on_state_change.lock().unwrap().as_ref() {
                    cb(TransportState::Connecting);
                }
            });
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Implementación del trait `Transport`
// ---------------------------------------------------------------------------

impl Transport for WebRtcTransport {
    fn name(&self) -> &'static str {
        "webrtc"
    }

    fn protocol(&self) -> Protocol {
        Protocol::WebRTC
    }

    fn send_data(&self, data: &[u8]) -> Result<usize, TransportError> {
        let dc = self.dc.lock().unwrap().clone();
        let dc = dc.ok_or_else(|| TransportError::Unavailable("DataChannel no inicializado".into()))?;

        let ready = dc.ready_state();
        if ready != webrtc::data_channel::data_channel_state::RTCDataChannelState::Open {
            return Err(TransportError::Unexpected(
                "DataChannel no está abierto".into(),
            ));
        }

        let dc_for_send = dc.clone();
        let payload = bytes::Bytes::from(data.to_vec());

        match tokio::runtime::Handle::try_current() {
            Ok(handle) => {
                handle.spawn(async move {
                    if let Err(e) = dc_for_send.send(&payload).await {
                        tracing::warn!("data_channel.send: {e}");
                    }
                });
                Ok(data.len())
            }
            Err(_) => Err(TransportError::Unexpected(
                "no hay runtime Tokio disponible para send_data".into(),
            )),
        }
    }

    fn connect(&mut self, config: &crate::network::types::NetworkConfig) -> Result<(), TransportError> {
        self.start(config)
    }

    fn disconnect(&mut self) -> Result<(), TransportError> {
        self.stop()
    }

    fn state(&self) -> TransportState {
        crate::network::transport::from_u8(self.state.load(Ordering::SeqCst))
    }

    fn local_id(&self) -> Option<String> {
        self.local_id.lock().unwrap().clone()
    }

    fn remote_id(&self) -> Option<String> {
        self.remote_id.lock().unwrap().clone()
    }
}