//! Orquestador del encoder: negocia HW → SW, ABR, stats y API async.
//!
//! Ciclo: `new` → `init(config)` (prueba NVENC → QSV → VTB → SW)
//! → `encode(frame)` → `set_bitrate` / `note_network_estimate` → `metrics()`.
//!
//! El backend OpenH264 es bloqueante: cada `encode()` corre en
//! `spawn_blocking` para no atascar el runtime async.

use std::time::Instant;

use super::abr::AdaptiveBitrate;
use super::backend::VideoEncoderBackend;
use super::backends::{NvencBackend, OpenH264Backend, QuickSyncBackend, VideoToolboxBackend};
use super::detect::preferred_order;
use super::error::{EncoderError, EncoderResult};
use super::types::{EncodedPacket, EncoderBackendKind, EncoderConfig, EncoderStats, VideoFrame};

/// Orquestador público del subsistema de encoding.
pub struct VideoEncoder {
    backend: Option<Box<dyn VideoEncoderBackend>>,
    kind: Option<EncoderBackendKind>,
    config: Option<EncoderConfig>,
    abr: Option<AdaptiveBitrate>,
    stats: EncoderStats,
    last_encode_at: Option<Instant>,
    encode_time_ema_ms: f64,
}

impl VideoEncoder {
    /// Crea el orquestador sin inicializar.
    pub fn new() -> Self {
        Self {
            backend: None,
            kind: None,
            config: None,
            abr: None,
            stats: EncoderStats::default(),
            last_encode_at: None,
            encode_time_ema_ms: 0.0,
        }
    }

    /// Backend activo, si hay.
    pub fn backend_kind(&self) -> Option<EncoderBackendKind> {
        self.kind
    }

    /// Inicializa negociando el mejor backend disponible.
    ///
    /// Prueba en el orden de `preferred_order`; los stubs HW fallan
    /// con `BackendUnavailable` y se continúa. El último (software)
    /// siempre funciona.
    pub async fn init(&mut self, config: EncoderConfig) -> EncoderResult<()> {
        config.validate().map_err(EncoderError::InvalidConfig)?;
        if config.codec != crate::ffi::VideoCodec::H264 {
            return Err(EncoderError::UnsupportedCodec {
                backend: "orquestador",
                codec: config.codec.clone(),
            });
        }
        let mut last_err: Option<EncoderError> = None;
        for kind in preferred_order() {
            let mut b = Self::make_backend(kind);
            if !b.is_available() {
                tracing::debug!("encoder: {} no disponible, siguiente", b.name());
                continue;
            }
            match b.init(&config) {
                Ok(()) => {
                    tracing::info!("encoder: backend activo {}", b.name());
                    self.abr = Some(AdaptiveBitrate::new(
                        config.min_bitrate_bps,
                        config.max_bitrate_bps,
                        config.bitrate_bps,
                    ));
                    self.backend = Some(b);
                    self.kind = Some(kind);
                    self.config = Some(config);
                    self.stats = EncoderStats::default();
                    self.last_encode_at = None;
                    return Ok(());
                }
                Err(EncoderError::BackendUnavailable(msg)) => {
                    tracing::info!("encoder: {} sin SDK ({}), siguiente", kind.name(), msg);
                    last_err = Some(EncoderError::BackendUnavailable(msg));
                }
                Err(e) => {
                    tracing::warn!("encoder: {} fallo ({}), siguiente", kind.name(), e);
                    last_err = Some(e);
                }
            }
        }
        Err(last_err.unwrap_or(EncoderError::BackendUnavailable("sin backends".into())))
    }

    /// Fabrica de backends.
    fn make_backend(kind: EncoderBackendKind) -> Box<dyn VideoEncoderBackend> {
        match kind {
            EncoderBackendKind::Nvenc => Box::new(NvencBackend::new()),
            EncoderBackendKind::QuickSync => Box::new(QuickSyncBackend::new()),
            EncoderBackendKind::VideoToolbox => Box::new(VideoToolboxBackend::new()),
            EncoderBackendKind::Software => Box::new(OpenH264Backend::new()),
        }
    }

    /// Codifica un frame. `Ok(None)` = dropeado por rate-control.
    /// Corre el backend bloqueante en `spawn_blocking`.
    pub async fn encode(&mut self, frame: VideoFrame) -> EncoderResult<Option<EncodedPacket>> {
        frame.validate().map_err(EncoderError::InvalidFrame)?;
        if self.backend.is_none() {
            return Err(EncoderError::NotInitialized);
        }
        let t0 = Instant::now();
        let mut backend = self.backend.take().expect("backend presente");
        let res = tokio::task::spawn_blocking(move || {
            let r = backend.encode(&frame);
            (backend, r)
        })
        .await
        .map_err(|e| EncoderError::BackendFailure {
            backend: "spawn_blocking",
            message: format!("join: {e}"),
        })?;
        let (backend, r) = res;
        self.backend = Some(backend);
        let dt_ms = t0.elapsed().as_secs_f64() * 1000.0;
        self.encode_time_ema_ms = if self.encode_time_ema_ms == 0.0 {
            dt_ms
        } else {
            self.encode_time_ema_ms * 0.9 + dt_ms * 0.1
        };
        self.stats.frames_in += 1;
        match r {
            Ok(Some(pkt)) => {
                self.stats.frames_out += 1;
                self.stats.bytes_out += pkt.data.len() as u64;
                if pkt.is_keyframe {
                    self.stats.keyframes += 1;
                }
                self.update_rate_stats();
                Ok(Some(pkt))
            }
            Ok(None) => {
                self.stats.frames_dropped += 1;
                Ok(None)
            }
            Err(e) => Err(e),
        }
    }

    /// Ajuste manual de bitrate (UI / settings).
    pub async fn set_bitrate(&mut self, bitrate_bps: u32) -> EncoderResult<()> {
        let backend = self.backend.as_mut().ok_or(EncoderError::NotInitialized)?;
        backend.set_bitrate(bitrate_bps)?;
        if let Some(cfg) = self.config.as_mut() {
            cfg.bitrate_bps = bitrate_bps;
        }
        if let Some(abr) = self.abr.as_mut() {
            if let Some(cfg) = &self.config {
                abr.set_range(cfg.min_bitrate_bps, cfg.max_bitrate_bps, bitrate_bps);
            }
        }
        Ok(())
    }

    /// Estimación de la red (Fase 3: GCC/REMB). Reconfigura si el ABR lo pide.
    pub async fn note_network_estimate(&mut self, available_bps: u32) -> EncoderResult<()> {
        let want = match self.abr.as_mut() {
            Some(abr) => abr.note_network_estimate(available_bps),
            None => return Err(EncoderError::NotInitialized),
        };
        if let Some(bps) = want {
            tracing::info!("encoder: ABR {} -> {} bps (red estima {})", self.abr.as_ref().map(|a| a.current()).unwrap_or(0), bps, available_bps);
            self.set_bitrate(bps).await?;
        }
        Ok(())
    }

    /// Fuerza IDR en el próximo frame.
    pub async fn force_keyframe(&mut self) -> EncoderResult<()> {
        self.backend.as_mut().ok_or(EncoderError::NotInitialized)?.force_keyframe()
    }

    /// Actualiza FPS y bitrate medidos (ventana de 1 s).
    fn update_rate_stats(&mut self) {
        let now = Instant::now();
        if let Some(prev) = self.last_encode_at {
            let dt = now.duration_since(prev).as_secs_f64();
            if dt > 0.0 {
                let inst_fps = 1.0 / dt;
                self.stats.fps = if self.stats.fps == 0.0 {
                    inst_fps
                } else {
                    self.stats.fps * 0.9 + inst_fps * 0.1
                };
                // Bitrate ≈ bytes_out / tiempo desde el primer frame.
                // Aproximación: usa EMA del tamaño × fps.
                if self.stats.frames_out > 0 {
                    let avg_bytes = self.stats.bytes_out as f64 / self.stats.frames_out as f64;
                    self.stats.bitrate_bps = (avg_bytes * 8.0 * self.stats.fps) as u32;
                }
            }
        }
        self.last_encode_at = Some(now);
        self.stats.encode_latency_ms = self.encode_time_ema_ms;
    }

    /// Métricas para el FFI / UI.
    pub fn stats(&self) -> EncoderStats {
        self.stats.clone()
    }

    // === Compatibilidad con la API vieja (main.rs + FFI) ===

    /// Inicializa con defaults 1080p30. Equivale al `init()` stub anterior.
    pub async fn init_default(&mut self) {
        let cfg = EncoderConfig::default();
        if let Err(e) = self.init(cfg).await {
            tracing::warn!("encoder: init_default falló ({e})");
        }
    }

    /// Compat: configura codec + bitrate conservando el resto.
    pub async fn set_config(&mut self, codec: crate::ffi::VideoCodec, bitrate: u32) {
        let mut cfg = self.config.clone().unwrap_or_default();
        cfg.codec = codec;
        cfg.bitrate_bps = bitrate;
        cfg.min_bitrate_bps = cfg.min_bitrate_bps.min(bitrate);
        cfg.max_bitrate_bps = cfg.max_bitrate_bps.max(bitrate);
        if self.backend.is_some() {
            if self.init(cfg).await.is_err() {
                tracing::warn!("encoder: set_config no pudo re-inicializar");
            }
        } else if let Err(e) = self.init(cfg).await {
            tracing::warn!("encoder: set_config falló ({e})");
        }
    }

    /// Compat: codifica bytes RGB crudos con la config activa.
    /// Asume `w×h` de la config y formato RGB8.
    pub async fn encode_raw(&mut self, frame: Vec<u8>) -> Option<Vec<u8>> {
        let (w, h) = match &self.config {
            Some(c) => (c.width, c.height),
            None => return None,
        };
        let vf = VideoFrame {
            data: frame,
            width: w,
            height: h,
            format: super::types::PixelFormat::Rgb8,
            pts_us: now_us(),
            force_keyframe: false,
        };
        match self.encode(vf).await {
            Ok(Some(pkt)) => Some(pkt.data),
            _ => None,
        }
    }

    /// Compat: métricas en el tipo FFI.
    pub async fn metrics(&self) -> crate::ffi::PerformanceMetrics {
        crate::ffi::PerformanceMetrics {
            fps: self.stats.fps,
            bitrate: self.stats.bitrate_bps,
            latency_ms: self.stats.encode_latency_ms as u32,
            cpu_usage: 0.0, // Fase 7: telemetría de proceso
            memory_usage: 0,
        }
    }
}

impl Default for VideoEncoder {
    fn default() -> Self {
        Self::new()
    }
}

/// Reloj monotónico en microsegundos.
fn now_us() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_micros() as u64)
        .unwrap_or(0)
}
