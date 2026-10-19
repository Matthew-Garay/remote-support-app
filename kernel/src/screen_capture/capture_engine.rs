//! ScreenCapture: captura de pantalla por plataforma.
//!
//! Estado Fase 2: API real que produce [`VideoFrame`][crate::encoder::VideoFrame].
//! La captura nativa (DXGI / ScreenCaptureKit / PipeWire) llega en Fase 4;
//! mientras tanto `capture_frame()` genera un frame de prueba (patrón)
//! para validar el pipeline encoder → red sin hardware de captura.
//!
//! - Windows: DXGI Desktop Duplication API (Fase 4).
//! - macOS: ScreenCaptureKit (Fase 4).
//! - Linux: PipeWire / Wayland / XCB / X11 (Fase 4).

use crate::encoder::{PixelFormat, VideoFrame};

/// Configuración de captura.
#[derive(Debug, Clone)]
pub struct CaptureConfig {
    /// Ancho en píxeles.
    pub width: u32,
    /// Alto en píxeles.
    pub height: u32,
    /// FPS objetivo.
    pub fps: u32,
    /// Codec (informativo: lo usa el encoder, no el capturador).
    pub codec: crate::ffi::VideoCodec,
}

impl Default for CaptureConfig {
    fn default() -> Self {
        Self {
            width: 1920,
            height: 1080,
            fps: 30,
            codec: crate::ffi::VideoCodec::H264,
        }
    }
}

/// Capturador de pantalla.
pub struct ScreenCapture {
    config: Option<CaptureConfig>,
    frame_no: u64,
    running: bool,
}

impl ScreenCapture {
    /// Crea el capturador detenido.
    pub fn new() -> Self {
        Self {
            config: None,
            frame_no: 0,
            running: false,
        }
    }

    /// Inicializa (solicita permisos según SO en Fase 4).
    pub async fn init(&mut self) {
        tracing::info!("ScreenCapture::init");
    }

    /// Inicia la captura con la configuración dada.
    pub async fn start(&mut self, config: CaptureConfig) {
        tracing::info!("ScreenCapture::start: {:?}", config);
        self.config = Some(config);
        self.frame_no = 0;
        self.running = true;
    }

    /// Detiene la captura.
    pub async fn stop(&mut self) {
        tracing::info!("ScreenCapture::stop");
        self.running = false;
    }

    /// ¿Capturando?
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Captura un frame.
    ///
    /// Fase 2: genera un patrón de prueba BGRA (degradado animado por
    /// `frame_no`) para validar el pipeline sin captura nativa.
    /// Fase 4: reemplaza el cuerpo por DXGI/ScreenCaptureKit/PipeWire.
    pub async fn capture_frame(&mut self) -> Option<VideoFrame> {
        let cfg = self.config.clone()?;
        if !self.running {
            return None;
        }
        let w = cfg.width.max(2);
        let h = cfg.height.max(2);
        let n = self.frame_no;
        self.frame_no += 1;

        // Patrón BGRA: degradado + barra móvil (detecta tearing / orden).
        let mut data = Vec::with_capacity((w * h * 4) as usize);
        for y in 0..h {
            for x in 0..w {
                let bar = ((x + (n as u32 * 8) % w) % w) < w / 8;
                let r = ((x * 255 / w) as u8).wrapping_add((n % 64) as u8);
                let g = (y * 255 / h) as u8;
                let b = if bar { 255 } else { 64 };
                data.extend_from_slice(&[b, g, r, 255]);
            }
        }
        Some(VideoFrame {
            data,
            width: w,
            height: h,
            format: PixelFormat::Bgra8,
            pts_us: now_us(),
            force_keyframe: n == 0,
        })
    }
}

impl Default for ScreenCapture {
    fn default() -> Self {
        Self::new()
    }
}

/// Reloj en microsegundos.
fn now_us() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_micros() as u64)
        .unwrap_or(0)
}
