//! InputSimulator: Stub para inyección de teclado y ratón.
//! Implementar con:
//! - Windows: SendInput API
//! - macOS: CGEvent / CoreGraphics
//! - Linux: uinput / libei

#[derive(Debug, Clone)]
pub struct MouseEvent {
    pub x: f64,
    pub y: f64,
    pub button: crate::ffi::MouseButton,
    pub pressed: bool,
}

#[derive(Debug, Clone)]
pub struct KeyEvent {
    pub key: String,
    pub down: bool,
}

pub struct InputSimulator;

impl InputSimulator {
    pub fn new() -> Self {
        Self
    }

    pub async fn init(&mut self) {
        tracing::info!("InputSimulator::init (stub)");
    }

    pub async fn inject_mouse(&mut self, event: MouseEvent) {
        tracing::info!("InputSimulator::inject_mouse: {:?}", event);
        // TODO: Llamar API nativa del SO
    }

    pub async fn inject_key(&mut self, event: KeyEvent) {
        tracing::info!("InputSimulator::inject_key: {:?}", event);
        // TODO: Llamar API nativa del SO
    }
}
