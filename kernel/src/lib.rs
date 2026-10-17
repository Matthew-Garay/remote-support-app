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

// Re-export de la API pública para FFI
pub use ffi::KernelApi;
