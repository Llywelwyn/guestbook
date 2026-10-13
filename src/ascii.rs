pub const RAMP: &str = " .'`^\",:;Il!i~+_-?][}{1)(|\\/tfjrxnuvczXYUJCLQ0OZmwqpdbkhao*#MW8%B@$";

/// Rows needed to keep a `width`x`height` image in proportion at `columns`
/// monospace cells, assuming glyphs 0.6em wide on a 1em line.
pub fn rows_for(columns: u32, width: u32, height: u32) -> u32 {
    if width == 0 {
        return 0;
    }
    ((columns as f64 * height as f64 / width as f64 * 0.6).round() as u32).max(1)
}

pub fn render_png(bytes: &[u8], columns: u32) -> Option<String> {
    let (width, height, lum) = decode_luminance(bytes)?;
    render_cells(width, height, &lum, columns, rows_for(columns, width, height))
}

fn render_cells(width: u32, height: u32, lum: &[u8], columns: u32, rows: u32) -> Option<String> {
    if width == 0 || height == 0 || columns == 0 || rows == 0 {
        return None;
    }
    let ramp: Vec<char> = RAMP.chars().collect();
    let top = ramp.len() - 1;
    let (w, h) = (width as usize, height as usize);
    let (c, r) = (columns as usize, rows as usize);

    let mut lines: Vec<String> = (0..r)
        .map(|row| {
            let y0 = row * h / r;
            let y1 = ((row + 1) * h / r).max(y0 + 1).min(h);
            let line: String = (0..c)
                .map(|col| {
                    let x0 = col * w / c;
                    let x1 = ((col + 1) * w / c).max(x0 + 1).min(w);
                    let mut sum = 0u64;
                    for y in y0..y1 {
                        for x in x0..x1 {
                            sum += 255 - lum[y * w + x] as u64;
                        }
                    }
                    let ink = sum / ((y1 - y0) * (x1 - x0)) as u64;
                    ramp[ink as usize * top / 255]
                })
                .collect();
            line.trim_end().to_string()
        })
        .collect();

    while lines.last().is_some_and(|l| l.is_empty()) {
        lines.pop();
    }
    let first = lines.iter().position(|l| !l.is_empty())?;
    Some(lines[first..].join("\n"))
}

fn decode_luminance(bytes: &[u8]) -> Option<(u32, u32, Vec<u8>)> {
    let mut decoder = png::Decoder::new(std::io::Cursor::new(bytes));
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut reader = decoder.read_info().ok()?;
    let mut buf = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).ok()?;
    let channels = match info.color_type {
        png::ColorType::Grayscale => 1,
        png::ColorType::GrayscaleAlpha => 2,
        png::ColorType::Rgb => 3,
        png::ColorType::Rgba => 4,
        png::ColorType::Indexed => return None,
    };
    let lum = buf[..info.buffer_size()]
        .chunks_exact(channels)
        .map(|px| {
            let (l, a) = match channels {
                1 => (px[0] as u32, 255),
                2 => (px[0] as u32, px[1] as u32),
                3 => (luma(px), 255),
                _ => (luma(px), px[3] as u32),
            };
            ((l * a + 255 * (255 - a)) / 255) as u8
        })
        .collect();
    Some((info.width, info.height, lum))
}

fn luma(px: &[u8]) -> u32 {
    (px[0] as u32 * 299 + px[1] as u32 * 587 + px[2] as u32 * 114) / 1000
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cells(bytes: &[u8], columns: u32, rows: u32) -> Option<String> {
        let (w, h, lum) = decode_luminance(bytes)?;
        render_cells(w, h, &lum, columns, rows)
    }

    fn encode(width: u32, height: u32, color: png::ColorType, data: &[u8]) -> Vec<u8> {
        let mut out = Vec::new();
        let mut enc = png::Encoder::new(&mut out, width, height);
        enc.set_color(color);
        enc.set_depth(png::BitDepth::Eight);
        enc.write_header().unwrap().write_image_data(data).unwrap();
        out
    }

    #[test]
    fn test_rows_for_keeps_proportion() {
        assert_eq!(rows_for(84, 554, 253), 23);
        assert_eq!(rows_for(10, 0, 10), 0);
        assert_eq!(rows_for(1, 1000, 1), 1);
    }

    #[test]
    fn test_white_image_is_none() {
        let png = encode(4, 4, png::ColorType::Rgba, &[255; 64]);
        assert_eq!(cells(&png, 4, 4), None);
    }

    #[test]
    fn test_transparent_image_is_none() {
        let png = encode(4, 4, png::ColorType::Rgba, &[0; 64]);
        assert_eq!(cells(&png, 4, 4), None);
    }

    #[test]
    fn test_black_pixel_is_densest_glyph() {
        let mut data = vec![255u8; 2 * 2 * 3];
        data[0..3].copy_from_slice(&[0, 0, 0]);
        let png = encode(2, 2, png::ColorType::Rgb, &data);
        assert_eq!(cells(&png, 2, 2).as_deref(), Some("$"));
    }

    #[test]
    fn test_trims_blank_rows_and_trailing_space() {
        let mut data = vec![255u8; 3 * 3];
        data[4] = 0;
        let png = encode(3, 3, png::ColorType::Grayscale, &data);
        assert_eq!(cells(&png, 3, 3).as_deref(), Some(" $"));
    }

    #[test]
    fn test_averages_cells() {
        let data = [0u8, 255, 255, 255];
        let png = encode(2, 2, png::ColorType::Grayscale, &data);
        let glyph = cells(&png, 1, 1).unwrap();
        let ramp: Vec<char> = RAMP.chars().collect();
        assert_eq!(glyph, ramp[63 * (ramp.len() - 1) / 255].to_string());
    }

    #[test]
    fn test_render_png_derives_rows_from_aspect() {
        let data = vec![0u8; 10 * 6];
        let png = encode(10, 6, png::ColorType::Grayscale, &data);
        let art = render_png(&png, 10).unwrap();
        assert_eq!(art.lines().count(), 4);
    }

    #[test]
    fn test_invalid_bytes_is_none() {
        assert_eq!(render_png(b"not a png", 4), None);
    }
}
