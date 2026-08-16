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
