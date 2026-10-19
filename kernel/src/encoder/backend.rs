//! Trait común de backends de codificación.
//!
//! Cada backend (NVENC, QuickSync, VideoToolbox, OpenH264-SW)
//! implementa este trait. El orquestador (`VideoEncoder`) los prueba
//! en orden de preferencia y usa el primero disponible.

use super::error::EncoderResult;
use super::types::{EncodedPacket, EncoderBackendKind, EncoderConfig, VideoFrame};

/// Backend de codificación de video (HW o SW).
///
/// El ciclo de vida es: `name` → `is_available` → `init` →
/// `encode*` → `set_bitrate` / `force_keyframe` → `close`.
///
/// `Send` para poder vivir detrás de `Arc<Mutex<VideoEncoder>>`
/// en el API FFI.
pub trait VideoEncoderBackend: Send {
    /// Nombre canónico (`nvenc`, `quicksync`, `videotoolbox`, `openh264-sw`).
    fn name(&self) -> &'static str;

    /// Clase de backend.
    fn kind(&self) -> EncoderBackendKind;

    /// Detección en runtime (GPU/driver/SDK presente).
    /// Barata y sin efectos: solo sondea el SO.
    fn is_available(&self) -> bool;

    /// Inicializa el backend con la configuración dada.
    /// Consume `config` validada por el orquestador.
    fn init(&mut self, config: &EncoderConfig) -> EncoderResult<()>;

    /// Codifica un frame. Devuelve `Ok(None)` solo si el backend
    /// decidió dropear el frame (skip por rate-control).
    fn encode(&mut self, frame: &VideoFrame) -> EncoderResult<Option<EncodedPacket>>;

    /// Ajuste de bitrate en caliente (ABR). Default: no-op.
    fn set_bitrate(&mut self, _bitrate_bps: u32) -> EncoderResult<()> {
        Ok(())
    }

    /// Fuerza IDR en el próximo frame. Default: no-op.
    fn force_keyframe(&mut self) -> EncoderResult<()> {
        Ok(())
    }

    /// Libera recursos nativos. Default: no-op.
    fn close(&mut self) {}
}
