//! VideoEncoder: Stub para codificación de video con aceleración por HW.
//! Implementar con:
//! - NVENC (NVIDIA)
//! - QuickSync (Intel)
//! - VideoToolbox (Apple)
//! - Fallback: x264 / x265

#[derive(Debug, Clone)]
pub struct EncoderConfig {
    pub codec: crate::ffi::VideoCodec,
    pub bitrate: u32,
    pub width: u32,
    pub height: u32,
    pub fps: u32,
}

pub struct VideoEncoder;

impl VideoEncoder {
    pub fn new() -> Self {
        Self
    }

    pub async fn init(&mut self) {
        tracing::info!("VideoEncoder::init (stub)");
    }

    pub async fn set_config(&mut self, codec: crate::ffi::VideoCodec, bitrate: u32) {
        tracing::info!("VideoEncoder::set_config: codec={:?}, bitrate={}", codec, bitrate);
    }

    pub async fn encode(&mut self, frame: Vec<u8>) -> Option<Vec<u8>> {
        // TODO: Codificar frame con HW/SW encoder
        None
    }

    pub async fn metrics(&self) -> crate::ffi::PerformanceMetrics {
        crate::ffi::PerformanceMetrics {
            fps: 0.0,
            bitrate: 0,
            latency_ms: 0,
            cpu_usage: 0.0,
            memory_usage: 0,
        }
    }
}
