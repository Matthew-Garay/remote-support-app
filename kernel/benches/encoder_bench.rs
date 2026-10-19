//! Bench del pipeline: conversión BGRA→I420 + encode OpenH264.
//!
//! Ejecutar con GPU/CI modesta:
//! `cargo bench --bench encoder_bench`
//!
//! Mide 640×360 (rápido en CI) y reporta throughput. En una máquina
//! de desarrollo 1080p30 con OpenH264-screen debería dar > 30 fps.

use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use remote_kernel::encoder::{EncoderConfig, PixelFormat, VideoFrame};
use remote_kernel::encoder::backends::OpenH264Backend;
use remote_kernel::encoder::VideoEncoderBackend;

/// Frame sintético BGRA 640×360 (degradado + ruido determinista).
fn synth_frame(w: u32, h: u32, seed: u64) -> VideoFrame {
    let mut data = Vec::with_capacity((w * h * 4) as usize);
    for y in 0..h {
        for x in 0..w {
            let r = ((x * 255 / w) ^ (seed as u32)) as u8;
            let g = (y * 255 / h) as u8;
            let b = ((x ^ y) & 0xFF) as u8;
            data.extend_from_slice(&[b, g, r, 255]);
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

fn bench_encode_360p(c: &mut Criterion) {
    let mut group = c.benchmark_group("encode_640x360_openh264");
    group.throughput(Throughput::Elements(1));
    group.bench_function("bgra_to_h264", |b| {
        let mut enc = OpenH264Backend::new();
        let cfg = EncoderConfig {
            width: 640,
            height: 360,
            fps: 30,
            bitrate_bps: 1_000_000,
            min_bitrate_bps: 250_000,
            max_bitrate_bps: 1_500_000,
            ..Default::default()
        };
        enc.init(&cfg).expect("init openh264");
        let mut i = 0u64;
        b.iter(|| {
            i += 1;
            let f = synth_frame(640, 360, i);
            let _ = enc.encode(&f).expect("encode");
        });
    });
    group.finish();
}

criterion_group!(benches, bench_encode_360p);
criterion_main!(benches);
