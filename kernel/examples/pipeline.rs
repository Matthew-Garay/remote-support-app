//! Ejemplo E2E Fase 2: captura sintética → OpenH264 → stats.
//!
//! ```sh
//! cargo run --example pipeline --release
//! ```
//!
//! Codifica 120 frames 640×360 del generador de `ScreenCapture`,
//! muestra backend negociado, bytes totales, keyframes y FPS.

use remote_kernel::encoder::{EncoderConfig, VideoEncoder};
use remote_kernel::screen_capture::{CaptureConfig, ScreenCapture};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    let mut cap = ScreenCapture::new();
    cap.init().await;
    cap.start(CaptureConfig {
        width: 640,
        height: 360,
        fps: 30,
        codec: remote_kernel::ffi::VideoCodec::H264,
    })
    .await;

    let mut enc = VideoEncoder::new();
    enc.init(EncoderConfig {
        width: 640,
        height: 360,
        fps: 30,
        bitrate_bps: 1_000_000,
        min_bitrate_bps: 250_000,
        max_bitrate_bps: 1_500_000,
        ..Default::default()
    })
    .await
    .expect("init encoder");

    println!("backend: {:?}", enc.backend_kind());

    let t0 = std::time::Instant::now();
    let mut packets = 0u64;
    let mut bytes = 0u64;
    let mut keys = 0u64;
    for _ in 0..120 {
        let Some(frame) = cap.capture_frame().await else {
            break;
        };
        match enc.encode(frame).await {
            Ok(Some(pkt)) => {
                packets += 1;
                bytes += pkt.data.len() as u64;
                if pkt.is_keyframe {
                    keys += 1;
                }
            }
            Ok(None) => {}
            Err(e) => {
                eprintln!("encode error: {e}");
                break;
            }
        }
    }
    let dt = t0.elapsed().as_secs_f64();
    let stats = enc.stats();
    println!("frames: 120 en {dt:.2}s ({:.1} fps)", 120.0 / dt);
    println!("paquetes: {packets}, bytes: {bytes}, keyframes: {keys}");
    println!(
        "stats: out={} drop={} fps={:.1} br={} bps lat={:.2} ms",
        stats.frames_out,
        stats.frames_dropped,
        stats.fps,
        stats.bitrate_bps,
        stats.encode_latency_ms
    );
}
