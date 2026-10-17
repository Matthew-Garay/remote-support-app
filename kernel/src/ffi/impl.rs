//! Implementación de la API FFI para Flutter.
//! Conecta con los módulos reales del kernel.

use crate::ffi::{CaptureEvent, CaptureState, InputEvent, KernelState, PerformanceMetrics, VideoCodec};
use crate::screen_capture::ScreenCapture;
use crate::input_simulator::InputSimulator;
use crate::encoder::VideoEncoder;
use crate::network::NetworkEngine;
use flutter_rust_bridge::Stream;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct KernelApiImpl {
    capture: Arc<Mutex<ScreenCapture>>,
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
    // TODO: Implementar cada método llamando a los módulos reales
}

#[flutter_rust_bridge::bridge]
pub trait KernelApi {
    fn start_capture(&mut self, config: crate::ffi::CaptureConfig) -> Stream<CaptureEvent>;
    fn stop_capture(&mut self) -> CaptureState;
    fn capture_state(&self) -> CaptureState;
    fn set_encoder_config(&mut self, codec: VideoCodec, bitrate: u32) -> Result<(), String>;
    fn encoder_metrics(&self) -> PerformanceMetrics;
    fn inject_mouse_event(&self, event: InputEvent) -> Result<(), String>;
    fn inject_key_event(&self, event: InputEvent) -> Result<(), String>;
    fn kernel_state(&self) -> KernelState;
    fn restart(&mut self) -> Result<(), String>;
}
