//! Implementación de la API para Flutter.
//!
//! Fase 2: lógica nativa directa (sin codegen). En Fase 5 se generará el
//! puente `flutter_rust_bridge` v2 con `flutter_rust_bridge_codegen`
//! sobre estos mismos métodos.

use crate::encoder::EncoderConfig;
use crate::ffi::{CaptureState, InputEvent, KernelState, PerformanceMetrics, VideoCodec};
use crate::input_simulator::{InputSimulator, KeyEvent, MouseEvent};
use crate::network::NetworkEngine;
use crate::{encoder::VideoEncoder, screen_capture::ScreenCapture};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Estado compartido del kernel (UI + daemon).
#[derive(Clone)]
pub struct KernelApiImpl {
    capture: Arc<Mutex<ScreenCapture>>,
    #[allow(dead_code)]
    input: Arc<Mutex<InputSimulator>>,
    encoder: Arc<Mutex<VideoEncoder>>,
    network: Arc<Mutex<NetworkEngine>>,
}

impl Default for KernelApiImpl {
    fn default() -> Self {
        Self {
            capture: Arc::new(Mutex::new(ScreenCapture::new())),
            input: Arc::new(Mutex::new(InputSimulator::new())),
            encoder: Arc::new(Mutex::new(VideoEncoder::new())),
            network: Arc::new(Mutex::new(NetworkEngine::new())),
        }
    }
}

impl KernelApiImpl {
    /// Inicia captura + encoder con la config FFI.
    pub async fn start_capture(&self, config: crate::ffi::CaptureConfig) -> Result<(), String> {
        let mut cap = self.capture.lock().await;
        cap.start(crate::screen_capture::CaptureConfig {
            width: config.width,
            height: config.height,
            fps: config.fps,
            codec: config.codec.clone(),
        })
        .await;
        let mut enc = self.encoder.lock().await;
        enc.init(EncoderConfig {
            codec: config.codec,
            width: config.width,
            height: config.height,
            fps: config.fps,
            ..Default::default()
        })
        .await
        .map_err(|e| e.to_string())
    }

    /// Detiene la captura.
    pub async fn stop_capture(&self) -> CaptureState {
        self.capture.lock().await.stop().await;
        CaptureState::Stopped
    }

    /// Estado de captura.
    pub async fn capture_state(&self) -> CaptureState {
        if self.capture.lock().await.is_running() {
            CaptureState::Running
        } else {
            CaptureState::Stopped
        }
    }

    /// Reconfigura codec + bitrate.
    pub async fn set_encoder_config(
        &self,
        codec: VideoCodec,
        bitrate: u32,
    ) -> Result<(), String> {
        self.encoder.lock().await.set_config(codec, bitrate).await;
        Ok(())
    }

    /// Métricas del encoder.
    pub async fn encoder_metrics(&self) -> PerformanceMetrics {
        self.encoder.lock().await.metrics().await
    }

    /// Inyecta evento de ratón (Fase 4: SendInput/CGEvent/uinput).
    pub async fn inject_mouse_event(&self, event: InputEvent) -> Result<(), String> {
        let _ev = match event {
            InputEvent::MouseMove { x, y } => MouseEvent {
                x,
                y,
                button: crate::ffi::MouseButton::Left,
                pressed: false,
            },
            InputEvent::MouseButton { button, pressed } => MouseEvent {
                x: 0.0,
                y: 0.0,
                button,
                pressed,
            },
            _ => return Err("evento no es de ratón".into()),
        };
        Ok(())
    }

    /// Inyecta evento de teclado (Fase 4).
    pub async fn inject_key_event(&self, event: InputEvent) -> Result<(), String> {
        let _ev = match event {
            InputEvent::Key { key, down } => KeyEvent { key, down },
            _ => return Err("evento no es de teclado".into()),
        };
        Ok(())
    }

    /// Estado del kernel.
    pub async fn kernel_state(&self) -> KernelState {
        if self.encoder.lock().await.backend_kind().is_some() {
            KernelState::Ready
        } else {
            KernelState::Idle
        }
    }

    /// Reinicia captura + encoder.
    pub async fn restart(&self) -> Result<(), String> {
        self.capture.lock().await.stop().await;
        self.encoder
            .lock()
            .await
            .init(EncoderConfig::default())
            .await
            .map_err(|e| e.to_string())
    }

    // ---------------------------------------------------------------------------
    // Red (Fase 3): WebRTC + señalización
    // ---------------------------------------------------------------------------

    /// Conecta el motor de red: señalización WebSocket + negociación WebRTC.
    pub async fn network_connect(
        &self,
        config: crate::ffi::NetworkConnectConfig,
    ) -> Result<(), String> {
        let net = self.network.lock().await;

        let role = if config.offering {
            crate::network::types::Role::Offer
        } else {
            crate::network::types::Role::Answer
        };

        let mut ice_servers = Vec::new();
        if !config.stun_urls.is_empty() {
            ice_servers.push(crate::network::types::IceServer {
                urls: config.stun_urls.clone(),
                username: None,
                credential: None,
                credential_type: None,
            });
        }
        if !config.turn_urls.is_empty() {
            ice_servers.push(crate::network::types::IceServer {
                urls: config.turn_urls.clone(),
                username: Some(config.turn_username.clone()),
                credential: Some(config.turn_credential.clone()),
                credential_type: None,
            });
        }

        net.configure(
            crate::network::types::NetworkConfig {
                local_id: config.device_id.clone(),
                peer_address: None,
                protocol: crate::network::types::Protocol::WebRTC,
                bandwidth_estimate_bps: None,
            },
            crate::network::types::PeerConfig {
                role,
                protocol: crate::network::types::Protocol::WebRTC,
                ice_servers,
                preferred_codec: "h264".to_string(),
                bandwidth_estimate_bps: None,
            },
        );

        net.connect(config.ws_url, config.device_id, config.password_hash)
            .await
    }

    /// Solicita conectarse a otro dispositivo (servidor de señalización).
    pub async fn network_request_connection(
        &self,
        target_device_id: String,
    ) -> Result<(), String> {
        self.network
            .lock()
            .await
            .request_connection(target_device_id)
            .await
    }

    /// Envía bytes por el DataChannel WebRTC.
    pub async fn network_send_data(&self, data: Vec<u8>) -> Result<usize, String> {
        self.network.lock().await.send_data(data).await
    }

    /// Estado de la conexión de red.
    pub async fn network_state(&self) -> String {
        self.network.lock().await.state().await
    }

    /// Cierra la conexión de red.
    pub async fn network_disconnect(&self) -> Result<(), String> {
        self.network.lock().await.disconnect().await
    }

    /// Suscripción a eventos de red.
    ///
    /// NOTA Fase 5: cuando se ejecute `flutter_rust_bridge_codegen generate`,
    /// se añadirá una variante que reciba `StreamSink<NetworkEvent>` (tipo
    /// generado por FRB en el propio crate); hasta entonces se usa un
    /// callback simple, funcional para pruebas desde Rust.
    pub async fn network_subscribe<F>(&self, callback: F) -> Result<(), String>
    where
        F: Fn(crate::ffi::NetworkEvent) + Send + Sync + 'static,
    {
        let net = self.network.lock().await;
        net.set_on_event(move |ev| {
            let ffi_event = match ev.kind {
                crate::network::types::NetworkEventKind::ConnectionStateChanged => {
                    crate::ffi::NetworkEvent::ConnectionStateChanged {
                        state: ev
                            .payload
                            .get("state")
                            .and_then(|v| v.as_str())
                            .unwrap_or_default()
                            .to_string(),
                    }
                }
                crate::network::types::NetworkEventKind::DataChannelMessage => {
                    let data = ev
                        .payload
                        .get("data")
                        .and_then(|v| v.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|x| x.as_u64())
                                .map(|x| x as u8)
                                .collect()
                        })
                        .unwrap_or_default();
                    crate::ffi::NetworkEvent::DataChannelMessage { data }
                }
                // AuthResult (y futuros eventos) → variante genérica.
                _ => crate::ffi::NetworkEvent::AuthResult {
                    success: ev
                        .payload
                        .get("auth")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false),
                    message: ev
                        .payload
                        .get("message")
                        .and_then(|v| v.as_str())
                        .unwrap_or_default()
                        .to_string(),
                },
            };
            callback(ffi_event);
        });
        Ok(())
    }
}
