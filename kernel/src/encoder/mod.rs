//! Video encoder: negociación HW → SW, ABR y stats.
//!
//! - [`VideoEncoder`][video_encoder::VideoEncoder]: orquestador público.
//! - [`backends`]: NVENC / QuickSync / VideoToolbox (stubs con detección)
//!   + OpenH264 por software (real).
//! - [`detect`]: sondeo de GPU/driver por plataforma.
//! - [`convert`]: RGB/BGRA → I420.
//! - [`abr`]: bitrate adaptativo con histéresis.

pub mod video_encoder;
pub mod backend;
pub mod backends;
pub mod detect;
pub mod convert;
pub mod abr;
pub mod error;
pub mod types;

pub use video_encoder::VideoEncoder;
pub use backend::VideoEncoderBackend;
pub use error::{EncoderError, EncoderResult};
pub use types::{
    ContentHint, EncodedPacket, EncoderBackendKind, EncoderConfig, EncoderStats, PixelFormat,
    RateControl, VideoFrame,
};
