//! Backend NVIDIA NVENC (stub con detección real).
//!
//! Estado: **detección implementada, codificación pendiente**.
//! Para activarlo hay que enlazar el NVIDIA Video Codec SDK
//! (`nvEncodeAPI.h` + `libnvidia-encode`) vía `bindgen` o `nv-codec-headers`.
//!
//! Mientras tanto: `is_available()` sondea el driver y, si hay GPU,
//! `init()` devuelve un error que indica exactamente qué falta.
//! El orquestador cae a software sin romper la sesión.
//!
//! Integración futura (cuando se vendorice el SDK):
//! 1. `NvEncodeAPIGetMaxSupportedVersion` → negociar API ≥ 11.
//! 2. `NvEncOpenEncodeSessionEx` (device DX11/CUDA).
//! 3. Preset `P1` (ultra-low-latency) + `NV_ENC_TUNING_INFO_ULTRA_LOW_LATENCY`.
//! 4. `NV_ENC_CODEC_H264_GUID`, perfil High, `NV_ENC_PARAMS_RC_CBR`.
//! 5. Input ABGR → NV12 en GPU (sin roundtrip a CPU).
//! 6. Output Annex-B → [`EncodedPacket`][super::super::types::EncodedPacket].

use super::super::backend::VideoEncoderBackend;
use super::super::detect::has_nvenc;
use super::super::error::{EncoderError, EncoderResult};
use super::super::types::{EncodedPacket, EncoderBackendKind, EncoderConfig, VideoFrame};

/// Stub NVENC.
pub struct NvencBackend {
    initialized: bool,
}

impl NvencBackend {
    /// Crea el stub.
    pub fn new() -> Self {
        Self { initialized: false }
    }
}

impl Default for NvencBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl VideoEncoderBackend for NvencBackend {
    fn name(&self) -> &'static str {
        "nvenc"
    }

    fn kind(&self) -> EncoderBackendKind {
        EncoderBackendKind::Nvenc
    }

    fn is_available(&self) -> bool {
        has_nvenc()
    }

    fn init(&mut self, config: &EncoderConfig) -> EncoderResult<()> {
        config.validate().map_err(EncoderError::InvalidConfig)?;
        if config.codec != crate::ffi::VideoCodec::H264 {
            return Err(EncoderError::UnsupportedCodec {
                backend: "nvenc",
                codec: config.codec.clone(),
            });
        }
        // GPU presente pero SDK no enlazado → error accionable.
        Err(EncoderError::BackendUnavailable(
            "NVENC detectado pero el SDK no está enlazado: vendorizar \
             NVIDIA Video Codec SDK (nvEncodeAPI.h) y habilitar \
             bindgen en kernel/src/encoder/backends/nvenc.rs"
                .into(),
        ))
    }

    fn encode(&mut self, _frame: &VideoFrame) -> EncoderResult<Option<EncodedPacket>> {
        if !self.initialized {
            return Err(EncoderError::NotInitialized);
        }
        unreachable!("init siempre falla hasta enlazar el SDK");
    }
}
