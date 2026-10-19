//! Backend Apple VideoToolbox (stub con detección real).
//!
//! Estado: **detección implementada, codificación pendiente**.
//! VideoToolbox es framework del sistema en macOS/iOS: no requiere SDK
//! externo, solo `objc2-video-toolbox` (o `bindgen` sobre
//! `VideoToolbox.framework`). En macOS `is_available()` es `true`.
//!
//! Integración futura (macOS):
//! 1. `VTCompressionSessionCreate` (codec `kCMVideoCodecType_H264`).
//! 2. `kVTCompressionPropertyKey_RealTime = true`,
//!    `kVTCompressionPropertyKey_ProfileLevel = H264_Baseline_AutoLevel`,
//!    `kVTCompressionPropertyKey_AverageBitRate = bitrate`,
//!    `kVTCompressionPropertyKey_MaxKeyFrameInterval = fps*5`,
//!    `kVTCompressionPropertyKey_AllowFrameReordering = false` (cero B-frames).
//! 3. `VTCompressionSessionEncodeFrame` con `CVPixelBuffer` BGRA directo
//!    (cero copias CPU si el capturador usa IOSurface).
//! 4. Callback `VTCompressionOutputCallback` → Annex-B con SPS/PPS
//!    antepuestos a cada IDR.

use super::super::backend::VideoEncoderBackend;
use super::super::detect::has_videotoolbox;
use super::super::error::{EncoderError, EncoderResult};
use super::super::types::{EncodedPacket, EncoderBackendKind, EncoderConfig, VideoFrame};

/// Stub VideoToolbox.
pub struct VideoToolboxBackend {
    initialized: bool,
}

impl VideoToolboxBackend {
    /// Crea el stub.
    pub fn new() -> Self {
        Self { initialized: false }
    }
}

impl Default for VideoToolboxBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl VideoEncoderBackend for VideoToolboxBackend {
    fn name(&self) -> &'static str {
        "videotoolbox"
    }

    fn kind(&self) -> EncoderBackendKind {
        EncoderBackendKind::VideoToolbox
    }

    fn is_available(&self) -> bool {
        has_videotoolbox()
    }

    fn init(&mut self, config: &EncoderConfig) -> EncoderResult<()> {
        config.validate().map_err(EncoderError::InvalidConfig)?;
        if config.codec != crate::ffi::VideoCodec::H264 {
            return Err(EncoderError::UnsupportedCodec {
                backend: "videotoolbox",
                codec: config.codec.clone(),
            });
        }
        Err(EncoderError::BackendUnavailable(
            "VideoToolbox presente pero no enlazado: agregar \
             objc2-video-toolbox como dependencia macOS-only y \
             completar kernel/src/encoder/backends/videotoolbox.rs"
                .into(),
        ))
    }

    fn encode(&mut self, _frame: &VideoFrame) -> EncoderResult<Option<EncodedPacket>> {
        if !self.initialized {
            return Err(EncoderError::NotInitialized);
        }
        unreachable!("init siempre falla hasta enlazar VideoToolbox");
    }
}
