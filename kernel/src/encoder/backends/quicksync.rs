//! Backend Intel QuickSync (stub con detección real).
//!
//! Estado: **detección implementada, codificación pendiente**.
//! Para activarlo hay que enlazar Intel Media SDK / oneVPL
//! (`mfxvideo++.h` + `libmfxhw64` / `libvpl`) vía `bindgen`.
//!
//! Integración futura:
//! 1. `MFXInit` (impl `MFX_IMPL_HARDWARE_ANY`).
//! 2. `MFXVideoENCODE_Init` con `MFX_CODEC_AVC`, target usage `MFX_TARGETUSAGE_BEST_SPEED`
//!    (equivalente a preset veryfast para baja latencia).
//! 3. `MFX_RATECONTROL_CBR`, `GopPicSize = fps*5`, `GopRefDist = 1`.
//! 4. `MFX_FOURCC_NV12` como entrada (convertir BGRA→NV12 en GPU con VPP).

use super::super::backend::VideoEncoderBackend;
use super::super::detect::has_quicksync;
use super::super::error::{EncoderError, EncoderResult};
use super::super::types::{EncodedPacket, EncoderBackendKind, EncoderConfig, VideoFrame};

/// Stub QuickSync.
pub struct QuickSyncBackend {
    initialized: bool,
}

impl QuickSyncBackend {
    /// Crea el stub.
    pub fn new() -> Self {
        Self { initialized: false }
    }
}

impl Default for QuickSyncBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl VideoEncoderBackend for QuickSyncBackend {
    fn name(&self) -> &'static str {
        "quicksync"
    }

    fn kind(&self) -> EncoderBackendKind {
        EncoderBackendKind::QuickSync
    }

    fn is_available(&self) -> bool {
        has_quicksync()
    }

    fn init(&mut self, config: &EncoderConfig) -> EncoderResult<()> {
        config.validate().map_err(EncoderError::InvalidConfig)?;
        if config.codec != crate::ffi::VideoCodec::H264 {
            return Err(EncoderError::UnsupportedCodec {
                backend: "quicksync",
                codec: config.codec.clone(),
            });
        }
        Err(EncoderError::BackendUnavailable(
            "QuickSync detectado pero Media SDK/oneVPL no enlazado: \
             vendorizar intel-mediasdk / oneVPL y habilitar \
             bindgen en kernel/src/encoder/backends/quicksync.rs"
                .into(),
        ))
    }

    fn encode(&mut self, _frame: &VideoFrame) -> EncoderResult<Option<EncodedPacket>> {
        if !self.initialized {
            return Err(EncoderError::NotInitialized);
        }
        unreachable!("init siempre falla hasta enlazar Media SDK/oneVPL");
    }
}
