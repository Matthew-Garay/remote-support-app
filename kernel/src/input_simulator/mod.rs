//! Inyección de inputs del sistema: SendInput (Windows), CGEvent (macOS), uinput (Linux).

pub mod input_simulator;

pub use input_simulator::{InputSimulator, KeyEvent, MouseEvent};
