pub fn srgb_byte_to_linear(value: u8) -> f32 {
    let normalized = value as f32 / 255.0;
    if normalized <= 0.04045 {
        normalized / 12.92
    } else {
        ((normalized + 0.055) / 1.055).powf(2.4)
    }
}

pub fn linear_to_srgb_byte(value: f32) -> u8 {
    let clamped = value.clamp(0.0, 1.0);
    let encoded = if clamped <= 0.0031308 {
        clamped * 12.92
    } else {
        1.055 * clamped.powf(1.0 / 2.4) - 0.055
    };
    (encoded * 255.0).round().clamp(0.0, 255.0) as u8
}

pub fn downsample_rgba8_box(
    src: &[u8],
    src_width: u32,
    src_height: u32,
    is_srgb: bool,
) -> (Vec<u8>, u32, u32) {
    let dst_width = (src_width / 2).max(1);
    let dst_height = (src_height / 2).max(1);
    let mut dst = vec![0u8; (dst_width * dst_height * 4) as usize];

    for y in 0..dst_height {
        for x in 0..dst_width {
            let dst_index = ((y * dst_width + x) * 4) as usize;
            let mut sum = [0.0_f32; 4];
            let mut count = 0.0_f32;

            for dy in 0..2 {
                for dx in 0..2 {
                    let sx = (x * 2 + dx).min(src_width.saturating_sub(1));
                    let sy = (y * 2 + dy).min(src_height.saturating_sub(1));
                    let src_index = ((sy * src_width + sx) * 4) as usize;

                    if is_srgb {
                        sum[0] += srgb_byte_to_linear(src[src_index]);
                        sum[1] += srgb_byte_to_linear(src[src_index + 1]);
                        sum[2] += srgb_byte_to_linear(src[src_index + 2]);
                    } else {
                        sum[0] += src[src_index] as f32 / 255.0;
                        sum[1] += src[src_index + 1] as f32 / 255.0;
                        sum[2] += src[src_index + 2] as f32 / 255.0;
                    }
                    sum[3] += src[src_index + 3] as f32 / 255.0;
                    count += 1.0;
                }
            }

            sum[0] /= count;
            sum[1] /= count;
            sum[2] /= count;
            sum[3] /= count;

            if is_srgb {
                dst[dst_index] = linear_to_srgb_byte(sum[0]);
                dst[dst_index + 1] = linear_to_srgb_byte(sum[1]);
                dst[dst_index + 2] = linear_to_srgb_byte(sum[2]);
            } else {
                dst[dst_index] = (sum[0] * 255.0).round().clamp(0.0, 255.0) as u8;
                dst[dst_index + 1] = (sum[1] * 255.0).round().clamp(0.0, 255.0) as u8;
                dst[dst_index + 2] = (sum[2] * 255.0).round().clamp(0.0, 255.0) as u8;
            }
            dst[dst_index + 3] = (sum[3] * 255.0).round().clamp(0.0, 255.0) as u8;
        }
    }

    (dst, dst_width, dst_height)
}

pub fn resample_to_size(rgba_data: &[u8], width: u32, height: u32, target: u32) -> Vec<u8> {
    if width == target && height == target {
        return rgba_data.to_vec();
    }
    let mut out = vec![0u8; (target * target * 4) as usize];
    for y in 0..target {
        let src_y0 = ((y as u64) * (height as u64) / (target as u64)) as u32;
        let src_y1 = ((((y as u64) + 1) * (height as u64) / (target as u64)) as u32)
            .max(src_y0 + 1)
            .min(height);
        for x in 0..target {
            let src_x0 = ((x as u64) * (width as u64) / (target as u64)) as u32;
            let src_x1 = ((((x as u64) + 1) * (width as u64) / (target as u64)) as u32)
                .max(src_x0 + 1)
                .min(width);
            let mut accum = [0u32; 4];
            let mut count = 0u32;
            for sy in src_y0..src_y1 {
                for sx in src_x0..src_x1 {
                    let src_offset = ((sy * width + sx) * 4) as usize;
                    accum[0] += rgba_data[src_offset] as u32;
                    accum[1] += rgba_data[src_offset + 1] as u32;
                    accum[2] += rgba_data[src_offset + 2] as u32;
                    accum[3] += rgba_data[src_offset + 3] as u32;
                    count += 1;
                }
            }
            let divisor = count.max(1);
            let dst_offset = ((y * target + x) * 4) as usize;
            out[dst_offset] = (accum[0] / divisor) as u8;
            out[dst_offset + 1] = (accum[1] / divisor) as u8;
            out[dst_offset + 2] = (accum[2] / divisor) as u8;
            out[dst_offset + 3] = (accum[3] / divisor) as u8;
        }
    }
    out
}
