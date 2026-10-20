//! Network engine: WebRTC / QUIC para P2P cifrado.
//! Implementar con webrtc-rs 0.13.

pub mod network_engine;
pub mod signaling;
pub mod transport;
pub mod types;
pub mod webrtc_transport;

pub use network_engine::NetworkEngine;
pub use webrtc_transport::WebRtcTransport;
