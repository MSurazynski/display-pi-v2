use std::io::Cursor;

use image::ImageReader;
use tiny_skia::{IntRect, IntSize, Pixmap, PixmapPaint};

pub fn bytes_to_pixmap(image_bytes: &[u8]) -> Pixmap {
    let img = ImageReader::new(Cursor::new(image_bytes))
        .with_guessed_format()
        .unwrap()
        .decode()
        .unwrap();

    let rgba = img.to_rgba8();
    let width = img.width();
    let height = img.height();
    let size = IntSize::from_wh(width, height).unwrap();
    let mut data = rgba.into_raw();

    for px in data.chunks_exact_mut(4) {
        let alpha = px[3] as u16;

        px[0] = ((px[0] as u16 * alpha + 127) / 255) as u8;
        px[1] = ((px[1] as u16 * alpha + 127) / 255) as u8;
        px[2] = ((px[2] as u16 * alpha + 127) / 255) as u8;
    }

    Pixmap::from_vec(data, size).unwrap()
}

pub fn rotate_to_vertical(pixmap: &mut Pixmap) {
    if pixmap.width() > pixmap.height() {
        let mut canvas = Pixmap::new(pixmap.height(), pixmap.width()).unwrap();
        let paint = PixmapPaint::default();
        let transform =
            tiny_skia::Transform::from_rotate(90.0).post_translate(canvas.width() as f32, 0.0);
        canvas.draw_pixmap(0, 0, pixmap.as_ref(), &paint, transform, None);
        *pixmap = canvas;
    }
}

pub fn rotate_by_degrees(pixmap: &mut Pixmap, degree: i32) {
    // For 90/270, the output dimensions are swapped.
    let (new_w, new_h) = if degree == 90 || degree == 270 {
        (pixmap.height(), pixmap.width())
    } else {
        (pixmap.width(), pixmap.height())
    };

    let mut canvas = Pixmap::new(new_w, new_h).unwrap();

    let center_x = pixmap.width() / 2;
    let center_y = pixmap.height() / 2;

    let paint = PixmapPaint::default();
    let transform =
        tiny_skia::Transform::from_rotate_at(degree as f32, center_x as f32, center_y as f32);

    canvas.draw_pixmap(0, 0, pixmap.as_ref(), &paint, transform, None);

    *pixmap = canvas;
}

pub fn crop(pixmap: &mut Pixmap, crop_width: u32, crop_height: u32) -> Option<()> {
    if crop_width > pixmap.width() || crop_height > pixmap.height() {
        return None;
    }

    let x = (pixmap.width() - crop_width) / 2;
    let y = (pixmap.height() - crop_height) / 2;

    let rect = IntRect::from_xywh(x as i32, y as i32, crop_width, crop_height).unwrap();
    let new_pixmap = pixmap.clone_rect(rect).unwrap();

    *pixmap = new_pixmap;
    Some(())
}
