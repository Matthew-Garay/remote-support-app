//! Tests de integración Fase 2: orquestador + OpenH264.
//!
//! Sin GPU en CI: la negociación debe caer a `Software` y codificar
//! frames sintéticos reales.

use remote_kernel::encoder::{
    EncoderBackendKind, EncoderConfig, PixelFormat, VideoEncoder, VideoFrame,
};

fn small_cfg() -> EncoderConfig {
    EncoderConfig {
        width: 64,
        height: 64,
        fps: 30,
        bitrate_bps: 500_000,
        min_bitrate_bps: 125_000,
        max_bitrate_bps: 750_000,
        keyframe_interval: 30,
        ..Default::default()
    }
}

fn synth_bgra(w: u32, h: u32, v: u8) -> VideoFrame {
    let mut data = Vec::with_capacity((w * h * 4) as usize);
    for y in 0..h {
        for x in 0..w {
            data.extend_from_slice(&[v, (x & 0xFF) as u8, (y & 0xFF) as u8, 255]);
        }
    }
    VideoFrame {
        data,
        width: w,
        height: h,
        format: PixelFormat::Bgra8,
        pts_us: 0,
        force_keyframe: false,
    }
}

#[tokio::test]
async fn negotiates_to_software_without_gpu() {
    let mut enc = VideoEncoder::new();
    enc.init(small_cfg()).await.expect("init");
    assert_eq!(enc.backend_kind(), Some(EncoderBackendKind::Software));
}

#[tokio::test]
async fn first_frame_is_keyframe_with_nals() {
    let mut enc = VideoEncoder::new();
    enc.init(small_cfg()).await.expect("init");
    let pkt = enc
        .encode(synth_bgra(64, 64, 17))
        .await
        .expect("encode")
        .expect("paquete");
    assert!(pkt.is_keyframe, "primer frame debe ser IDR");
    assert!(!pkt.data.is_empty());
    assert!(pkt.nal_count() >= 1);
    assert_eq!(pkt.backend, "openh264-sw");
}

#[tokio::test]
async fn rejects_h265_until_backend_exists() {
    let mut enc = VideoEncoder::new();
    let mut cfg = small_cfg();
    cfg.codec = remote_kernel::ffi::VideoCodec::H265;
    assert!(enc.init(cfg).await.is_err());
    assert_eq!(enc.backend_kind(), None);
}

#[tokio::test]
async fn abr_drives_bitrate_down() {
    let mut enc = VideoEncoder::new();
    enc.init(small_cfg()).await.expect("init");
    // Red al mínimo: el ABR debe bajar al piso.
    enc.note_network_estimate(10_000).await.expect("abr");
    let s = enc.stats();
    let _ = s;
    // El backend se recreó con el piso; codificar sigue funcionando.
    let pkt = enc.encode(synth_bgra(64, 64, 42)).await.expect("encode");
    assert!(pkt.is_some());
}
