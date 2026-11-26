use chrono::Local;
use image::{DynamicImage, RgbImage};
use plotters::{
    backend::BitMapBackend,
    drawing::IntoDrawingArea,
    style::{BLACK, FontStyle, IntoTextStyle, RGBColor},
};

use crate::{
    epd7in3e::{Epd7in3e, FrameBuffer},
    render::{
        dither::{convert_pixmap_to_epaper_pallete, floyd_steinberg},
        image_util::image_to_pixmap,
    },
};

pub async fn make_dashboard() -> Result<(), Box<dyn std::error::Error>> {
    let mut buf = vec![0u8; (480 * 800 * 3) as usize];

    {
        let root = BitMapBackend::with_buffer(&mut buf, (480, 800)).into_drawing_area();

        root.fill(&RGBColor(251, 239, 194))?;

        const HEADER_HEIGHT: u32 = 50;

        let (header_section, root) = root.split_vertically(HEADER_HEIGHT);

        let now = Local::now();
        let weekday = now.format("%A");
        let month = now.format("%B");
        let day = now.format("%d");

        let text_style = ("Yanone Kaffeesatz", 35, FontStyle::Bold)
            .with_color(BLACK)
            .into_text_style(&root);
        let day = format!("Todayyy is {weekday}, {day} {month}");
        root.draw_text(&day, &text_style, (50, 50)).unwrap();
        header_section
            .draw_text(&day, &text_style, (50, 50))
            .unwrap();
    }

    let img = RgbImage::from_raw(480, 800, buf).expect("buffer has wrong size");

    let mut img = image_to_pixmap(DynamicImage::ImageRgb8(img));
    convert_pixmap_to_epaper_pallete(&mut img);
    floyd_steinberg(&mut img);

    let image_bytes = img.encode_png()?;
    // save_image(ImageType::Dashboard, image_bytes).await;

    let mut epd = Epd7in3e::new()?;
    let fb = FrameBuffer::from_png(&image_bytes)?;
    epd.display(&fb)?;

    Ok(())
}
