//! Main entry point for the Remote Support Kernel
//!
//! Daemon/servicio que corre el pipeline captura → encoder.
//! El puente FFI con Flutter (`flutter_rust_bridge`) se genera en Fase 5;
//! mientras tanto este binario valida el pipeline nativo.
//!
//! ```sh
//! cargo run --release          # pipeline 640x360 a 30 fps (demo)
//! RUST_LOG=debug cargo run     # con logs de negociación de backends
//! ```

mod ffi;
mod screen_capture;
mod input_simulator;
mod encoder;
mod network;

use std::time::Duration;

#[tokio::main]
async fn main() {
    // Inicializar logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    tracing::info!("Remote Support Kernel iniciado (Fase 2: encoder)");

    // Inicializar sub-sistemas
    let mut screen_capture = screen_capture::ScreenCapture::new();
    let mut encoder = encoder::VideoEncoder::new();
    let mut network = network::NetworkEngine::new();

    screen_capture.init().await;
    screen_capture
        .start(screen_capture::CaptureConfig {
            width: 640,
            height: 360,
            fps: 30,
            codec: ffi::VideoCodec::H264,
        })
        .await;
    if let Err(e) = encoder
        .init(encoder::EncoderConfig {
            width: 640,
            height: 360,
            fps: 30,
            bitrate_bps: 1_000_000,
            min_bitrate_bps: 250_000,
            max_bitrate_bps: 1_500_000,
            ..Default::default()
        })
        .await
    {
        tracing::error!("encoder init falló, sin video: {e}");
    }
    network.init().await;

    tracing::info!("backend encoder: {:?}", encoder.backend_kind());

    // Bucle del pipeline: captura → encode → red.
    let mut interval = tokio::time::interval(Duration::from_millis(33)); // ~30 fps
    let mut n: u64 = 0;
    loop {
        interval.tick().await;
        let Some(frame) = screen_capture.capture_frame().await else {
            continue;
        };
        match encoder.encode(frame).await {
            Ok(Some(pkt)) => {
                n += 1;
                // Fase 3: `network.send_video(pkt)` vía WebRTC.
                network.send_data(pkt.data).await;
                if n % 30 == 0 {
                    let s = encoder.stats();
                    tracing::info!(
                        "pipeline: {n} pkts fps={:.1} br={} lat={:.2}ms key={}",
                        s.fps,
                        s.bitrate_bps,
                        s.encode_latency_ms,
                        s.keyframes
                    );
                }
            }
            Ok(None) => tracing::debug!("pipeline: frame dropeado por rate-control"),
            Err(e) => tracing::warn!("pipeline: encode falló ({e})"),
        }
    }
}
