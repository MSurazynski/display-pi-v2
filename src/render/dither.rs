use image::{
    Rgb, RgbImage,
    imageops::{ColorMap, dither},
};

const EPAPER_PALETTE: [[u8; 3]; 6] = [
    [0, 0, 0],       // black
    [255, 255, 255], // white
    [0, 255, 0],     // green
    [0, 0, 255],     // blue
    [255, 0, 0],     // red
    [255, 255, 0],   // yellow
];

pub fn convert_pixmap_to_epaper_pallete(pixmap: &mut tiny_skia::Pixmap) {
    let width = pixmap.width();
    let height = pixmap.height();

    // Convert tiny_skia RBG buffer to image::RgbImage
    let rgb_data: Vec<u8> = pixmap
        .data()
        .chunks_exact(4)
        .flat_map(|px| [px[0], px[1], px[2]])
        .collect();

    let mut image = RgbImage::from_vec(width, height, rgb_data).expect("valid RGB image buffer");

    dither(&mut image, &EpaperColorMap);

    for (rgba, rgb) in pixmap.data_mut().chunks_exact_mut(4).zip(image.pixels()) {
        rgba[0] = rgb[0];
        rgba[1] = rgb[1];
        rgba[2] = rgb[2];
        rgba[3] = 255;
    }
}

fn nearest_palette(color: &[f32; 3]) -> [u8; 3] {
    EPAPER_PALETTE
        .iter()
        .min_by_key(|c| {
            let dr = color[0] - c[0] as f32;
            let dg = color[1] - c[1] as f32;
            let db = color[2] - c[2] as f32;
            (dr * dr + dg * dg + db * db) as i32
        })
        .copied()
        .unwrap()
}

pub fn floyd_steinberg(pixmap: &mut tiny_skia::Pixmap) {
    let width = pixmap.width() as i32;
    let height = pixmap.height() as i32;

    // Error buffer in f32, one [r,g,b] triple per pixel.
    let mut errors = vec![[0f32; 3]; (width * height) as usize];

    let data = pixmap.data_mut();

    for y in 0..height {
        for x in 0..width {
            let pixel_idx = (y * width + x) as usize;
            let byte_idx = pixel_idx * 4; // RGBA, 4 bytes per pixel

            let err = errors[pixel_idx];

            // Current value + accumulated error, clamped to [0, 255].
            let current = [
                (data[byte_idx] as f32 + err[0]).clamp(0.0, 255.0),
                (data[byte_idx + 1] as f32 + err[1]).clamp(0.0, 255.0),
                (data[byte_idx + 2] as f32 + err[2]).clamp(0.0, 255.0),
            ];

            // Find nearest palette color.
            let nearest = nearest_palette(&current);

            // Write it back (keep alpha opaque).
            data[byte_idx] = nearest[0];
            data[byte_idx + 1] = nearest[1];
            data[byte_idx + 2] = nearest[2];
            data[byte_idx + 3] = 255;

            // Quantization error to diffuse.
            let quant_err = [
                current[0] - nearest[0] as f32,
                current[1] - nearest[1] as f32,
                current[2] - nearest[2] as f32,
            ];

            let mut spread = |nx: i32, ny: i32, factor: f32| {
                if nx >= 0 && nx < width && ny >= 0 && ny < height {
                    let nidx = (ny * width + nx) as usize;
                    errors[nidx][0] += quant_err[0] * factor;
                    errors[nidx][1] += quant_err[1] * factor;
                    errors[nidx][2] += quant_err[2] * factor;
                }
            };

            spread(x + 1, y, 7.0 / 16.0);
            spread(x - 1, y + 1, 3.0 / 16.0);
            spread(x, y + 1, 5.0 / 16.0);
            spread(x + 1, y + 1, 1.0 / 16.0);
        }
    }
}
struct EpaperColorMap;

impl EpaperColorMap {
    fn nearest_index(color: &Rgb<u8>) -> usize {
        let [r, g, b] = color.0;

        EPAPER_PALETTE
            .iter()
            .enumerate()
            .min_by_key(|(_, c)| {
                let dr = r as i32 - c[0] as i32;
                let dg = g as i32 - c[1] as i32;
                let db = b as i32 - c[2] as i32;

                dr * dr + dg * dg + db * db
            })
            .map(|(i, _)| i)
            .unwrap()
    }
}

impl ColorMap for EpaperColorMap {
    type Color = Rgb<u8>;

    fn index_of(&self, color: &Self::Color) -> usize {
        Self::nearest_index(color)
    }

    fn lookup(&self, index: usize) -> Option<Self::Color> {
        EPAPER_PALETTE.get(index).copied().map(Rgb)
    }

    fn has_lookup(&self) -> bool {
        true
    }

    fn map_color(&self, color: &mut Self::Color) {
        let idx = self.index_of(color);
        *color = Rgb(EPAPER_PALETTE[idx]);
    }
}
