//! ScreenCapture: Stub para captura de pantalla.
//! Implementar con:
//! - Windows: DXGI Desktop Duplication API
//! - macOS: ScreenCaptureKit
//! - Linux: PipeWire / Wayland / XCB / X11

#[derive(Debug, Clone)]
pub struct CaptureConfig {
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub codec: crate::ffi::VideoCodec,
}

pub struct ScreenCapture;

impl ScreenCapture {
    pub fn new() -> Self {
        Self
    }

    pub async fn init(&mut self) {
        tracing::info!("ScreenCapture::init (stub)");
    }

    pub async fn start(&mut self, config: CaptureConfig) {
        tracing::info!("ScreenCapture::start: {:?}", config);
        // TODO: Iniciar captura con la API nativa de cada SO
    }

    pub async fn stop(&mut self) {
        tracing::info!("ScreenCapture::stop (stub)");
    }

    pub async fn capture_frame(&mut self) -> Option<Vec<u8>> {
        // TODO: Devolver frame YUV420 / RGB capturado
        None
    }
}
