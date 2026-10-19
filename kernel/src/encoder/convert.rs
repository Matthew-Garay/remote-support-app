//! Conversión de píxeles a I420 (YUV 4:2:0 planar).
//!
//! OpenH264 solo acepta YUV; el capturador entrega RGB/BGRA.
//! Conversión BT.601 de precisión entera (compatible con
//! `YUVBuffer::from_rgb8_source` de openh264, pero sin depender
//! del crate para poder testearla aislada).
//!
//! Layout I420: plano Y (`w*h`), plano U (`w/2*h/2`), plano V
//! (`w/2*h/2`). Submuestreo por promedio 2×2.

use super::types::PixelFormat;

/// Convierte `src` a I420. `src` debe medir
/// `format.buffer_len(w, h)` y `w`, `h` pares.
pub fn to_i420(
    src: &[u8],
    width: u32,
    height: u32,
    format: PixelFormat,
) -> Result<Vec<u8>, String> {
    if width % 2 != 0 || height % 2 != 0 {
        return Err(format!("dimensiones impares {width}x{height}"));
    }
    let w = width as usize;
    let h = height as usize;
    match format {
        PixelFormat::I420 => {
            if src.len() != w * h * 3 / 2 {
                return Err(format!("I420 len {} inválido", src.len()));
            }
            Ok(src.to_vec())
        }
        PixelFormat::Rgb8 => {
            expect(src.len(), w * h * 3, "RGB8")?;
            Ok(rgb_to_i420(src, w, h, 3, [0, 1, 2]))
        }
        PixelFormat::Rgba8 => {
            expect(src.len(), w * h * 4, "RGBA8")?;
            Ok(rgb_to_i420(src, w, h, 4, [0, 1, 2]))
        }
        PixelFormat::Bgra8 => {
            expect(src.len(), w * h * 4, "BGRA8")?;
            Ok(rgb_to_i420(src, w, h, 4, [2, 1, 0]))
        }
    }
}

fn expect(got: usize, want: usize, what: &str) -> Result<(), String> {
    if got != want {
        return Err(format!("{what}: {got} bytes != {want}"));
    }
    Ok(())
}

/// RGB(A/BGRA) → I420 con coeficientes BT.601 enteros.
///
/// ```text
/// Y = ( 66*R + 129*G +  25*B + 128) >> 8 + 16
/// U = (-38*R -  74*G + 112*B + 128) >> 8 + 128
/// V = (112*R -  94*G -  18*B + 128) >> 8 + 128
/// ```
fn rgb_to_i420(src: &[u8], w: usize, h: usize, bpp: usize, idx: [usize; 3]) -> Vec<u8> {
    let y_len = w * h;
    let uv_len = (w / 2) * (h / 2);
    let mut out = vec![0u8; y_len + uv_len * 2];
    let (y_plane, uv) = out.split_at_mut(y_len);
    let (u_plane, v_plane) = uv.split_at_mut(uv_len);

    // Luma por píxel.
    for y in 0..h {
        for x in 0..w {
            let o = (y * w + x) * bpp;
            let r = src[o + idx[0]] as i32;
            let g = src[o + idx[1]] as i32;
            let b = src[o + idx[2]] as i32;
            y_plane[y * w + x] = ((66 * r + 129 * g + 25 * b + 128) >> 8).clamp(0, 255) as u8 + 16;
        }
    }
    // Croma promediado 2×2.
    for y in (0..h).step_by(2) {
        for x in (0..w).step_by(2) {
            let mut su = 0i32;
            let mut sv = 0i32;
            for dy in 0..2 {
                for dx in 0..2 {
                    let o = ((y + dy) * w + (x + dx)) * bpp;
                    let r = src[o + idx[0]] as i32;
                    let g = src[o + idx[1]] as i32;
                    let b = src[o + idx[2]] as i32;
                    su += (-38 * r - 74 * g + 112 * b + 128) >> 8;
                    sv += (112 * r - 94 * g - 18 * b + 128) >> 8;
                }
            }
            let i = (y / 2) * (w / 2) + (x / 2);
            u_plane[i] = ((su / 4).clamp(-128, 127) + 128) as u8;
            v_plane[i] = ((sv / 4).clamp(-128, 127) + 128) as u8;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn black_white_roundtrip() {
        // Negro RGB(0,0,0) → Y≈16; blanco → Y≈235.
        let black = vec![0u8; 4 * 4 * 3];
        let yuv = to_i420(&black, 4, 4, PixelFormat::Rgb8).unwrap();
        assert_eq!(yuv.len(), 4 * 4 * 3 / 2);
        assert!(yuv[..16].iter().all(|&y| y <= 20));

        let white = vec![255u8; 4 * 4 * 3];
        let yuv = to_i420(&white, 4, 4, PixelFormat::Rgb8).unwrap();
        assert!(yuv[..16].iter().all(|&y| y >= 230));
    }

    #[test]
    fn bgra_red_is_luma_approx_81() {
        // Rojo puro BGRA(0,0,255,255) → Y ≈ 81 (BT.601).
        let red = vec![0u8, 0, 255, 255].repeat(4 * 4);
        let yuv = to_i420(&red, 4, 4, PixelFormat::Bgra8).unwrap();
        assert!(yuv[..16].iter().all(|&y| (70..=95).contains(&y)));
    }

    #[test]
    fn rejects_bad_input() {
        assert!(to_i420(&[0u8; 10], 4, 4, PixelFormat::Rgb8).is_err());
        assert!(to_i420(&[0u8; 48], 5, 4, PixelFormat::Rgb8).is_err());
    }
}
