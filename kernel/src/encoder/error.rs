//! Errores del subsistema de codificación de video.
//!
//! Todos los backends (NVENC, QuickSync, VideoToolbox, OpenH264-SW)
//! mapean sus errores nativos a [`EncoderError`].

use thiserror::Error;

/// Error unificado del encoder.
#[derive(Debug, Error)]
pub enum EncoderError {
    /// El encoder no está inicializado (`init()` no fue llamado o falló).
    #[error("encoder no inicializado")]
    NotInitialized,

    /// Configuración inválida (resolución 0, fps 0, bitrate 0, etc.).
    #[error("configuración inválida: {0}")]
    InvalidConfig(String),

    /// El backend de hardware no está disponible en este host
    /// (sin GPU/driver/SDK). El orquestador debe caer a software.
    #[error("backend de hardware no disponible: {0}")]
    BackendUnavailable(String),

    /// Falló la llamada nativa del backend.
    #[error("falló la codificación ({backend}): {message}")]
    BackendFailure {
        /// Nombre del backend (`nvenc`, `quicksync`, `videotoolbox`, `openh264-sw`).
        backend: &'static str,
        /// Detalle del fallo.
        message: String,
    },

    /// El frame de entrada no coincide con la configuración
    /// (tamaño de buffer, dimensiones impares, formato desconocido).
    #[error("frame inválido: {0}")]
    InvalidFrame(String),

    /// El codec solicitado no lo soporta el backend activo.
    #[error("codec no soportado por el backend {backend}: {codec:?}")]
    UnsupportedCodec {
        /// Nombre del backend activo.
        backend: &'static str,
        /// Codec solicitado.
        codec: crate::ffi::VideoCodec,
    },
}

/// Resultado del subsistema de encoding.
pub type EncoderResult<T> = Result<T, EncoderError>;
