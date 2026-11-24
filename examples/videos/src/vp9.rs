//! Thin owner for the RVP9 per-stream decoder state.

pub struct Vp9Decoder {
    inner: rvp9_decoder::Decoder,
    #[cfg(feature = "log")]
    stats_started: std::time::Instant,
    #[cfg(feature = "log")]
    decoded_frames: u64,
}

pub struct DecodedVideoFrame {
    pub width: usize,
    pub height: usize,
    /// Bottom-up BGRA rows, ready for a DirectShow RGB32 sample.
    pub bgra: Vec<u8>,
}

impl Vp9Decoder {
    pub fn new() -> Self {
        Self {
            inner: rvp9_decoder::Decoder::new(),
            #[cfg(feature = "log")]
            stats_started: std::time::Instant::now(),
            #[cfg(feature = "log")]
            decoded_frames: 0,
        }
    }

    pub fn inner_mut(&mut self) -> &mut rvp9_decoder::Decoder {
        &mut self.inner
    }

    /// Decode one WebM VP9 block. RVP9 parses superframes and processes every
    /// constituent frame in decode order so reference state is preserved.
    pub fn decode_packet(&mut self, packet: &[u8]) -> Result<Vec<DecodedVideoFrame>, String> {
        let frames = self
            .inner
            .parse_packet(packet)
            .map_err(|error| format!("Invalid VP9 packet: {error:?}"))?;
        let mut decoded = Vec::new();
        for frame in frames {
            #[cfg(feature = "log")]
            let decode_started = std::time::Instant::now();
            let frame = self
                .inner
                .decode_frame(&frame)
                .map_err(|error| format!("VP9 decode failed: {error:?}"))?;
            #[cfg(feature = "log")]
            let decode_ms = decode_started.elapsed().as_secs_f64() * 1000.0;

            #[cfg(feature = "log")]
            let convert_started = std::time::Instant::now();
            let rgb32 = to_rgb32(&frame)?;
            #[cfg(feature = "log")]
            {
                let convert_ms = convert_started.elapsed().as_secs_f64() * 1000.0;
                self.decoded_frames += 1;
                let elapsed = self.stats_started.elapsed().as_secs_f64();
                let average_fps = self.decoded_frames as f64 / elapsed.max(f64::EPSILON);
                crate::videos_log!(
                    "[videos] VP9 frame {}: decode={:.2} ms, RGB32={:.2} ms, average={:.2} fps",
                    self.decoded_frames,
                    decode_ms,
                    convert_ms,
                    average_fps
                );
            }
            decoded.push(rgb32);
        }
        Ok(decoded)
    }
}

fn to_rgb32(frame: &rvp9_core::frame::Yv12Buffer) -> Result<DecodedVideoFrame, String> {
    let y = frame.y_plane();
    let u = frame.u_plane();
    let v = frame.v_plane();
    let width = y.width();
    let height = y.height();
    if width == 0
        || height == 0
        || u.width() == 0
        || u.height() == 0
        || v.width() == 0
        || v.height() == 0
    {
        return Err("RVP9 returned an empty frame".into());
    }
    let mut bgra = vec![
        0u8;
        width
            .checked_mul(height)
            .and_then(|n| n.checked_mul(4))
            .ok_or("RGB32 frame size overflow")?
    ];
    let chroma_420 = u.width() == width.div_ceil(2)
        && v.width() == width.div_ceil(2)
        && u.height() == height.div_ceil(2)
        && v.height() == height.div_ceil(2);
    if chroma_420 {
        // VP9's normal YV12 layout is 4:2:0. Reuse each chroma pair for
        // its 2x2 luma block instead of doing per-pixel plane scaling.
        for cy in 0..u.height() {
            let y_top = cy * 2;
            let y_bottom = (y_top + 1).min(height - 1);
            let yrow_top = y.row(y_top);
            let yrow_bottom = y.row(y_bottom);
            let urow = u.row(cy);
            let vrow = v.row(cy);
            for cx in 0..u.width() {
                let x_left = cx * 2;
                let cb = urow[cx] as i32 - 128;
                let cr = vrow[cx] as i32 - 128;
                for dx in 0..2 {
                    let x = x_left + dx;
                    if x >= width {
                        continue;
                    }
                    let top_offset = ((height - 1 - y_top) * width + x) * 4;
                    write_yuv_pixel(&mut bgra[top_offset..top_offset + 4], yrow_top[x], cb, cr);
                    if y_bottom != y_top {
                        let bottom_offset = ((height - 1 - y_bottom) * width + x) * 4;
                        write_yuv_pixel(
                            &mut bgra[bottom_offset..bottom_offset + 4],
                            yrow_bottom[x],
                            cb,
                            cr,
                        );
                    }
                }
            }
        }
    } else {
        // Keep a dimension-independent fallback for nonstandard subsampling.
        for sy in 0..height {
            let yrow = y.row(sy);
            let urow = u.row(sy * u.height() / height);
            let vrow = v.row(sy * v.height() / height);
            let dy = height - 1 - sy;
            for sx in 0..width {
                let cb = urow[sx * u.width() / width] as i32 - 128;
                let cr = vrow[sx * v.width() / width] as i32 - 128;
                let offset = (dy * width + sx) * 4;
                write_yuv_pixel(&mut bgra[offset..offset + 4], yrow[sx], cb, cr);
            }
        }
    }
    Ok(DecodedVideoFrame {
        width,
        height,
        bgra,
    })
}

#[inline]
fn write_yuv_pixel(dst: &mut [u8], luma: u8, cb: i32, cr: i32) {
    let c = (luma as i32 - 16).max(0);
    let r = ((298 * c + 409 * cr + 128) >> 8).clamp(0, 255) as u8;
    let g = ((298 * c - 100 * cb - 208 * cr + 128) >> 8).clamp(0, 255) as u8;
    let b = ((298 * c + 516 * cb + 128) >> 8).clamp(0, 255) as u8;
    dst.copy_from_slice(&[b, g, r, 0xff]);
}
