//! Backend de software: Cisco OpenH264 (H.264 SVC).
//!
//! Fallback multiplataforma cuando no hay GPU/driver HW.
//! Decisiones:
//! - `UsageType::ScreenContentRealTime`: texto y bordes duros.
//! - `RateControlMode::Bitrate`: bitrate predecible para red.
//! - `SpsPpsStrategy::IncreasingId`: SPS/PPS fresca por IDR para
//!   que un cliente que se une a mitad de stream decodifique.
//! - `EncodedBitStream::to_vec()` emite Annex-B con start codes.

use super::backend::VideoEncoderBackend;
use super::convert::to_i420;
use super::error::{EncoderError, EncoderResult};
use super::types::{EncodedPacket, EncoderBackendKind, EncoderConfig, VideoFrame};

use openh264::encoder::{
    BitRate, Complexity, Encoder, EncoderConfig as O264Config, FrameRate, FrameType,
    RateControlMode, SpsPpsStrategy, UsageType,
};
use openh264::formats::YUVBuffer;
use openh264::OpenH264API;

/// Backend OpenH264 por software.
pub struct OpenH264Backend {
    encoder: Option<Encoder>,
    config: Option<EncoderConfig>,
    pending_keyframe: bool,
    frames_since_idr: u32,
}

impl OpenH264Backend {
    /// Crea el backend sin inicializar.
    pub fn new() -> Self {
        Self {
            encoder: None,
            config: None,
            pending_keyframe: true, // primer frame siempre IDR
            frames_since_idr: 0,
        }
    }

    /// Traduce [`EncoderConfig`] a [`O264Config`].
    fn build_o264_config(cfg: &EncoderConfig) -> O264Config {
        let usage = match cfg.content {
            super::types::ContentHint::Screen => UsageType::ScreenContentRealTime,
            super::types::ContentHint::Camera => UsageType::CameraVideoRealTime,
        };
        let rc = match cfg.rate_control {
            super::types::RateControl::Cbr | super::types::RateControl::Vbr => {
                RateControlMode::Bitrate
            }
            // QP fijo ≈ control por calidad.
            super::types::RateControl::ConstQp => RateControlMode::Quality,
        };
        // Complejidad media: equilibrio 1080p30 en CPU moderno.
        // En escritorio con poco movimiento OpenH264 rinde ~60-120 fps
        // en un Ryzen/i7 reciente a 1080p (modo screen, skip habilitado).
        O264Config::new()
            .usage_type(usage)
            .rate_control_mode(rc)
            .bitrate(BitRate::from_bps(cfg.bitrate_bps))
            .max_frame_rate(FrameRate::from_hz(cfg.fps as f32))
            .sps_pps_strategy(SpsPpsStrategy::IncreasingId)
            .skip_frames(true)
            .complexity(Complexity::Medium)
    }

    /// (Re)crea el encoder nativo.
    fn create_encoder(&mut self, cfg: &EncoderConfig) -> EncoderResult<()> {
        let api = OpenH264API::from_source();
        let o264 = Self::build_o264_config(cfg);
        match Encoder::with_api_config(api, o264) {
            Ok(enc) => {
                self.encoder = Some(enc);
                self.config = Some(cfg.clone());
                self.pending_keyframe = true;
                self.frames_since_idr = 0;
                tracing::info!(
                    "openh264-sw: init {}x{}@{} {} bps",
                    cfg.width,
                    cfg.height,
                    cfg.fps,
                    cfg.bitrate_bps
                );
                Ok(())
            }
            Err(e) => Err(EncoderError::BackendFailure {
                backend: "openh264-sw",
                message: format!("with_api_config: {e:?}"),
            }),
        }
    }
}

impl Default for OpenH264Backend {
    fn default() -> Self {
        Self::new()
    }
}

impl VideoEncoderBackend for OpenH264Backend {
    fn name(&self) -> &'static str {
        "openh264-sw"
    }

    fn kind(&self) -> EncoderBackendKind {
        EncoderBackendKind::Software
    }

    fn is_available(&self) -> bool {
        // OpenH264 compila de fuentes (feature `source`): siempre presente.
        true
    }

    fn init(&mut self, config: &EncoderConfig) -> EncoderResult<()> {
        if config.codec != crate::ffi::VideoCodec::H264 {
            return Err(EncoderError::UnsupportedCodec {
                backend: "openh264-sw",
                codec: config.codec.clone(),
            });
        }
        config.validate().map_err(EncoderError::InvalidConfig)?;
        self.create_encoder(config)
    }

    fn encode(&mut self, frame: &VideoFrame) -> EncoderResult<Option<EncodedPacket>> {
        frame.validate().map_err(EncoderError::InvalidFrame)?;
        let cfg = self.config.clone().ok_or(EncoderError::NotInitialized)?;

        // Cambio de resolución → recrear encoder (OpenH264 soporta
        // resolución dinámica, pero recrear evita artefactos).
        if frame.width != cfg.width || frame.height != cfg.height {
            tracing::info!(
                "openh264-sw: resolución {}x{} → {}x{}, recreando",
                cfg.width,
                cfg.height,
                frame.width,
                frame.height
            );
            let mut new_cfg = cfg.clone();
            new_cfg.width = frame.width;
            new_cfg.height = frame.height;
            self.create_encoder(&new_cfg)?;
        }

        let encoder = self.encoder.as_mut().ok_or(EncoderError::NotInitialized)?;

        // Keyframe: pedido explícito, primero, o por intervalo.
        let need_idr = frame.force_keyframe
            || self.pending_keyframe
            || (cfg.keyframe_interval > 0 && self.frames_since_idr >= cfg.keyframe_interval);
        if need_idr {
            encoder.force_intra_frame();
        }

        // RGB/BGRA → I420 → YUVBuffer.
        let i420 = to_i420(&frame.data, frame.width, frame.height, frame.format)
            .map_err(EncoderError::InvalidFrame)?;
        let yuv = YUVBuffer::from_vec(i420, frame.width as usize, frame.height as usize);

        match encoder.encode(&yuv) {
            Ok(bitstream) => {
                let is_key = matches!(
                    bitstream.frame_type(),
                    FrameType::IDR | FrameType::I
                );
                let data = bitstream.to_vec();
                self.frames_since_idr = if is_key { 0 } else { self.frames_since_idr + 1 };
                self.pending_keyframe = false;
                if data.is_empty() {
                    // Skip por rate-control: el backend dropeó el frame.
                    Ok(None)
                } else {
                    Ok(Some(EncodedPacket {
                        data,
                        is_keyframe: is_key,
                        pts_us: frame.pts_us,
                        backend: "openh264-sw",
                    }))
                }
            }
            Err(e) => Err(EncoderError::BackendFailure {
                backend: "openh264-sw",
                message: format!("encode: {e:?}"),
            }),
        }
    }

    fn set_bitrate(&mut self, bitrate_bps: u32) -> EncoderResult<()> {
        let cfg = self.config.clone().ok_or(EncoderError::NotInitialized)?;
        if bitrate_bps < cfg.min_bitrate_bps || bitrate_bps > cfg.max_bitrate_bps {
            return Err(EncoderError::InvalidConfig(format!(
                "bitrate {bitrate_bps} fuera de [{}, {}]",
                cfg.min_bitrate_bps, cfg.max_bitrate_bps
            )));
        }
        // OpenH264 0.9 no expone set_option en caliente vía API segura:
        // recreamos el encoder (rápido, < 5 ms) preservando el resto.
        let mut new_cfg = cfg;
        new_cfg.bitrate_bps = bitrate_bps;
        self.create_encoder(&new_cfg)
    }

    fn force_keyframe(&mut self) -> EncoderResult<()> {
        if self.encoder.is_none() {
            return Err(EncoderError::NotInitialized);
        }
        self.pending_keyframe = true;
        Ok(())
    }

    fn close(&mut self) {
        self.encoder = None;
        self.config = None;
    }
}

