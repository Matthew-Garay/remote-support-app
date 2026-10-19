//! Backends de codificación: HW (NVENC/QuickSync/VideoToolbox) + SW (OpenH264).
//!
//! - `openh264_sw`: software real, siempre disponible.
//! - `nvenc` / `quicksync` / `videotoolbox`: detección real + stubs de
//!   codificación documentados (enlazar SDK del vendor para activarlos).

pub mod openh264_sw;
pub mod nvenc;
pub mod quicksync;
pub mod videotoolbox;

pub use openh264_sw::OpenH264Backend;
pub use nvenc::NvencBackend;
pub use quicksync::QuickSyncBackend;
pub use videotoolbox::VideoToolboxBackend;
