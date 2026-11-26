use chrono::{Datelike, Local};
use plotters::{
    backend::{BitMapBackend, DrawingBackend},
    drawing::IntoDrawingArea,
    style::{BLACK, FontStyle, IntoTextStyle, RED, RGBColor, TextStyle},
};

use crate::{
    epd7in3e::{Epd7in3e, FrameBuffer},
    render::{
        dither::{convert_pixmap_to_epaper_pallete, floyd_steinberg},
        image_util::bytes_to_pixmap,
    },
    storage::load_image,
    types::ImageType,
};

pub async fn make_dashboard() -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new("output/dashboard.bmp", (480, 800)).into_drawing_area();
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
    let day = format!("Today is {weekday}, {day} {month}");
    root.draw_text(&day, &text_style, (50, 50)).unwrap();
    header_section
        .draw_text(&day, &text_style, (50, 50))
        .unwrap();

    root.present()?;

    let img = load_image(ImageType::Dashboard, Local::now().date_naive())
        .await
        .unwrap();

    let mut img = bytes_to_pixmap(&img);
    convert_pixmap_to_epaper_pallete(&mut img);
    floyd_steinberg(&mut img);

    let img = img.encode_png()?;
    let mut epd = Epd7in3e::new()?;
    let fb = FrameBuffer::from_png(&img)?;
    epd.display(&fb)?;

    Ok(())
}
