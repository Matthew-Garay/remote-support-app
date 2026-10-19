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
    #[allow(dead_code)]
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
}
