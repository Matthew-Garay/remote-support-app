//! Bitrate adaptativo (ABR) guiado por la red.
//!
//! El NetworkEngine (Fase 3) reporta el bitrate disponible estimado
//! (Google Congestion Control / REMB) vía `note_network_estimate()`.
//! El controlador mapea esa estimación al rango `[min, max]` con
//! histéresis para no oscilar: solo cambia si la diferencia supera
//! el 20% y pasaron ≥ 500 ms desde el último cambio.

use std::time::{Duration, Instant};

/// Controlador ABR con histéresis.
#[derive(Debug)]
pub struct AdaptiveBitrate {
    min_bps: u32,
    max_bps: u32,
    current_bps: u32,
    last_change: Option<Instant>,
    cooldown: Duration,
    /// Cambio mínimo relativo para actuar (0.20 = 20%).
    threshold: f64,
}

impl AdaptiveBitrate {
    /// Crea el controlador con piso, techo y bitrate inicial.
    pub fn new(min_bps: u32, max_bps: u32, initial_bps: u32) -> Self {
        Self {
            min_bps,
            max_bps,
            current_bps: initial_bps.clamp(min_bps, max_bps),
            last_change: None,
            cooldown: Duration::from_millis(500),
            threshold: 0.20,
        }
    }

    /// Bitrate actual objetivo.
    pub fn current(&self) -> u32 {
        self.current_bps
    }

    /// Sincroniza el rango cuando cambia `EncoderConfig`.
    pub fn set_range(&mut self, min_bps: u32, max_bps: u32, target_bps: u32) {
        self.min_bps = min_bps;
        self.max_bps = max_bps;
        self.current_bps = target_bps.clamp(min_bps, max_bps);
        self.last_change = None;
    }

    /// Nueva estimación de la red. Devuelve `Some(nuevo_bitrate)` si hay
    /// que reconfigurar el backend, `None` si se mantiene.
    pub fn note_network_estimate(&mut self, available_bps: u32) -> Option<u32> {
        let want = available_bps.clamp(self.min_bps, self.max_bps);
        let diff = (want as f64 - self.current_bps as f64).abs() / self.current_bps as f64;
        if diff < self.threshold {
            return None;
        }
        if let Some(t) = self.last_change {
            if t.elapsed() < self.cooldown {
                return None;
            }
        }
        self.current_bps = want;
        self.last_change = Some(Instant::now());
        Some(want)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ignores_small_changes() {
        let mut abr = AdaptiveBitrate::new(1_000_000, 6_000_000, 4_000_000);
        // +10% < umbral 20% → None.
        assert_eq!(abr.note_network_estimate(4_400_000), None);
        assert_eq!(abr.current(), 4_000_000);
    }

    #[test]
    fn clamps_to_range() {
        let mut abr = AdaptiveBitrate::new(1_000_000, 6_000_000, 4_000_000);
        assert_eq!(abr.note_network_estimate(100_000), Some(1_000_000));
    }
}
