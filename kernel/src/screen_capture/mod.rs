//! Pantalla de captura: Win32 DXGI, macOS ScreenCaptureKit, Linux PipeWire/Wayland/X11.

pub mod capture_engine;

// Re-export
pub use capture_engine::ScreenCapture;
