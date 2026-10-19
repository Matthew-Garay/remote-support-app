//! Tipos del pipeline de codificación.
//!
//! Independientes del backend (NVENC, QuickSync, VideoToolbox,
//! OpenH264-SW) y del FFI con Flutter.

use serde::{Deserialize, Serialize};

/// Formato de píxeles de un frame de entrada.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PixelFormat {
    /// RGB 8 bits/canal entrelazado (`w*h*3` bytes).
    Rgb8,
    /// RGBA 8 bits/canal (`w*h*4` bytes).
    Rgba8,
    /// BGRA 8 bits/canal, orden Windows/GDI (`w*h*4` bytes).
    Bgra8,
    /// YUV 4:2:0 planar (`w*h*3/2` bytes: Y + U + V).
    I420,
}

impl PixelFormat {
    /// Tamaño esperado del buffer para `w × h`.
    pub fn buffer_len(self, width: u32, height: u32) -> usize {
        let (n, d) = match self {
            Self::Rgb8 => (3usize, 1usize),
            Self::Rgba8 | Self::Bgra8 => (4, 1),
            Self::I420 => (3, 2),
        };
        (width as usize) * (height as usize) * n / d
    }
}

/// Un frame crudo listo para codificar.
#[derive(Debug, Clone)]
pub struct VideoFrame {
    /// Píxeles en el formato indicado.
    pub data: Vec<u8>,
    /// Ancho en píxeles (par, exigido por H.264).
    pub width: u32,
    /// Alto en píxeles (par, exigido por H.264).
    pub height: u32,
    /// Formato de `data`.
    pub format: PixelFormat,
    /// Timestamp en microsegundos (reloj del capturador).
    pub pts_us: u64,
    /// Forzar keyframe (cambio de resolución, nuevo cliente, pérdida).
    pub force_keyframe: bool,
}

impl VideoFrame {
    /// Valida dimensiones y tamaño del buffer.
    pub fn validate(&self) -> Result<(), String> {
        if self.width == 0 || self.height == 0 {
            return Err(format!("dimensiones 0: {}x{}", self.width, self.height));
        }
        if self.width % 2 != 0 || self.height % 2 != 0 {
            return Err(format!(
                "dimensiones impares {}x{} (H.264 exige pares)",
                self.width, self.height
            ));
        }
        let expected = self.format.buffer_len(self.width, self.height);
        if self.data.len() != expected {
            return Err(format!(
                "buffer {} bytes != esperado {} ({:?} {}x{})",
                self.data.len(),
                expected,
                self.format,
                self.width,
                self.height
            ));
        }
        Ok(())
    }
}

/// Estrategia de control de bitrate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum RateControl {
    /// Bitrate constante: latencia predecible, ideal para remoto.
    #[default]
    Cbr,
    /// Bitrate variable: mejor calidad, picos de red.
    Vbr,
    /// QP fijo: grabación local / debug.
    ConstQp,
}

/// Uso del contenido (afina presets del backend).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ContentHint {
    /// Escritorio / apps (texto, bordes duros). Default en remoto.
    #[default]
    Screen,
    /// Cámara / video natural.
    Camera,
}

/// Configuración completa del encoder.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncoderConfig {
    /// Codec de salida.
    pub codec: crate::ffi::VideoCodec,
    /// Ancho de salida.
    pub width: u32,
    /// Alto de salida.
    pub height: u32,
    /// FPS objetivo.
    pub fps: u32,
    /// Bitrate objetivo en bps.
    pub bitrate_bps: u32,
    /// Piso del ABR (default 25% del objetivo).
    pub min_bitrate_bps: u32,
    /// Techo del ABR (default 150% del objetivo).
    pub max_bitrate_bps: u32,
    /// Control de tasa.
    pub rate_control: RateControl,
    /// Pista de contenido.
    pub content: ContentHint,
    /// Cada cuántos frames forzar IDR (0 = solo a pedido).
    pub keyframe_interval: u32,
    /// Hilos del backend SW (0 = auto). Ignorado por HW.
    pub threads: u16,
}

impl Default for EncoderConfig {
    fn default() -> Self {
        Self {
            codec: crate::ffi::VideoCodec::H264,
            width: 1920,
            height: 1080,
            fps: 30,
            bitrate_bps: 4_000_000,
            min_bitrate_bps: 1_000_000,
            max_bitrate_bps: 6_000_000,
            rate_control: RateControl::Cbr,
            content: ContentHint::Screen,
            keyframe_interval: 150,
            threads: 0,
        }
    }
}

impl EncoderConfig {
    /// Valida antes de inicializar un backend.
    pub fn validate(&self) -> Result<(), String> {
        if self.width == 0 || self.height == 0 {
            return Err("resolución 0".into());
        }
        if self.width % 2 != 0 || self.height % 2 != 0 {
            return Err(format!(
                "resolución impar {}x{} (H.264 exige pares)",
                self.width, self.height
            ));
        }
        if self.fps == 0 || self.fps > 120 {
            return Err(format!("fps fuera de rango: {}", self.fps));
        }
        if self.bitrate_bps == 0 {
            return Err("bitrate 0".into());
        }
        if self.min_bitrate_bps > self.bitrate_bps
            || self.bitrate_bps > self.max_bitrate_bps
        {
            return Err(format!(
                "bitrate fuera de rango: min={} target={} max={}",
                self.min_bitrate_bps, self.bitrate_bps, self.max_bitrate_bps
            ));
        }
        Ok(())
    }

    /// Bitrate inicial sugerido (H.264 escritorio ≈ 0.07 bpp).
    pub fn suggested_bitrate(width: u32, height: u32, fps: u32) -> u32 {
        let est = (width as f64) * (height as f64) * (fps as f64) * 0.07;
        (est as u32).clamp(500_000, 12_000_000)
    }
}

/// Paquete codificado listo para WebRTC (NAL Annex-B).
#[derive(Debug, Clone)]
pub struct EncodedPacket {
    /// Bytes Annex-B (`0x00000001`).
    pub data: Vec<u8>,
    /// `true` si contiene un IDR.
    pub is_keyframe: bool,
    /// PTS del frame origen (µs).
    pub pts_us: u64,
    /// Backend productor (`nvenc`, `quicksync`, `videotoolbox`, `openh264-sw`).
    pub backend: &'static str,
}

impl EncodedPacket {
    /// Cuenta NAL units buscando start codes.
    pub fn nal_count(&self) -> usize {
        let d = &self.data;
        let mut n = 0;
        let mut i = 0;
        while i + 3 < d.len() {
            let sc3 = d[i] == 0 && d[i + 1] == 0 && d[i + 2] == 1;
            let sc4 = d[i] == 0 && d[i + 1] == 0 && d[i + 2] == 0 && d[i + 3] == 1;
            if sc3 || sc4 {
                n += 1;
                i += if sc3 { 3 } else { 4 };
            } else {
                i += 1;
            }
        }
        n
    }
}

/// Backend activo tras la negociación.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EncoderBackendKind {
    /// NVIDIA NVENC (Win/Linux + GPU NVIDIA + Video Codec SDK).
    Nvenc,
    /// Intel QuickSync vía Media SDK / oneVPL (iGPU Intel).
    QuickSync,
    /// Apple VideoToolbox (macOS / iOS).
    VideoToolbox,
    /// Cisco OpenH264 por software (todas las plataformas).
    Software,
}

impl EncoderBackendKind {
    /// Nombre canónico (logs y [`EncodedPacket::backend`]).
    pub const fn name(self) -> &'static str {
        match self {
            Self::Nvenc => "nvenc",
            Self::QuickSync => "quicksync",
            Self::VideoToolbox => "videotoolbox",
            Self::Software => "openh264-sw",
        }
    }
}

/// Estadísticas vivas (ventana deslizante).
#[derive(Debug, Clone, Default)]
pub struct EncoderStats {
    /// Frames recibidos del capturador.
    pub frames_in: u64,
    /// Frames codificados OK.
    pub frames_out: u64,
    /// Frames dropeados (saturación / skip).
    pub frames_dropped: u64,
    /// Keyframes emitidos.
    pub keyframes: u64,
    /// Bytes emitidos (acumulado).
    pub bytes_out: u64,
    /// FPS medido (media móvil).
    pub fps: f64,
    /// Bitrate medido en bps (media móvil).
    pub bitrate_bps: u32,
    /// Latencia media de `encode()` en ms.
    pub encode_latency_ms: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buffer_len_ok() {
        assert_eq!(PixelFormat::Rgb8.buffer_len(1920, 1080), 1920 * 1080 * 3);
        assert_eq!(PixelFormat::Bgra8.buffer_len(1280, 720), 1280 * 720 * 4);
        assert_eq!(PixelFormat::I420.buffer_len(64, 64), 64 * 64 * 3 / 2);
    }

    #[test]
    fn frame_validate() {
        let ok = VideoFrame {
            data: vec![0u8; 64 * 64 * 3 / 2],
            width: 64,
            height: 64,
            format: PixelFormat::I420,
            pts_us: 0,
            force_keyframe: false,
        };
        assert!(ok.validate().is_ok());
        let odd = VideoFrame { width: 65, ..ok.clone() };
        assert!(odd.validate().is_err());
        let short = VideoFrame { data: vec![0u8; 10], ..ok };
        assert!(short.validate().is_err());
    }

    #[test]
    fn config_ranges() {
        let mut c = EncoderConfig::default();
        assert!(c.validate().is_ok());
        c.width = 1919;
        assert!(c.validate().is_err());
        c.width = 1920;
        c.min_bitrate_bps = 9_999_999;
        assert!(c.validate().is_err());
    }
}
