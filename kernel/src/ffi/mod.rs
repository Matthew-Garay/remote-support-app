//! Módulo FFI: puente entre Flutter y Rust.
//! Define la API pública expuesta a Dart a través de flutter_rust_bridge.

mod lib;
mod impl_;

pub use lib::*;
pub use impl_::*;
