//! Remote Support App Kernel
//!
//! Kernel de bajo nivel encargado de:
//! - Captura de pantalla
//! - Inyección de inputs
//! - Codificación de video
//! - Comunicación de red (WebRTC/QUIC)

pub mod screen_capture;
pub mod input_simulator;
pub mod encoder;
pub mod network;
pub mod ffi;

// Re-export de la API pública para FFI y para ejemplos/tests externos.
pub use ffi::{
    CaptureConfig as FfiCaptureConfig, CaptureEvent, CaptureState, InputEvent, KernelState,
    MouseButton, PerformanceMetrics, VideoCodec, VideoResolution,
};
pub use encoder::{
    ContentHint, EncodedPacket, EncoderBackendKind, EncoderConfig, EncoderError, EncoderStats,
    PixelFormat, RateControl, VideoEncoder, VideoEncoderBackend, VideoFrame,
};
pub use screen_capture::{CaptureConfig, ScreenCapture};
