use chrono::Local;
use image::{DynamicImage, RgbImage};
use plotters::{
    backend::{BitMapBackend, DrawingBackend},
    coord::Shift,
    drawing::{DrawingArea, IntoDrawingArea},
    element::PathElement,
    style::{BLACK, Color, FontStyle, GREEN, IntoTextStyle, RED, RGBColor},
};

use crate::{
    render::{
        display::display_image_from_memory,
        dither::{convert_pixmap_to_epaper_pallete, floyd_steinberg},
        image_util::image_to_pixmap,
    },
    storage::save_image_to_storage,
    types::{Day::Today, ImageType::Dashboard, Stage::Processed, TimeOfDay::Morning},
};

struct Section<'a> {
    content_area: DrawingArea<BitMapBackend<'a>, Shift>,
    height: u32,
    width: u32,
}

impl<'a> Section<'a> {
    fn new(
        height: u32,
        padding_top: u32,
        padding_bottom: u32,
        padding_left: u32,
        padding_right: u32,
        free_space: &mut DrawingArea<BitMapBackend<'a>, Shift>,
    ) -> Section<'a> {
        let width = 480 - padding_left - padding_right;
        let (content_area, b) = free_space.split_vertically(height + padding_top + padding_bottom);
        let content_area = content_area.shrink((padding_left, padding_top), (width, height));

        let section: Section = Section {
            content_area,
            height,
            width,
        };
        *free_space = b;
        section
    }

    fn draw_line_borders(&self) {
        let style_content = GREEN.stroke_width(1);

        let line_content_top =
            PathElement::new(vec![(0, 0), (self.width as i32, 0)], style_content);
        let line_content_bottom = PathElement::new(
            vec![
                (0, self.height as i32),
                (self.width as i32, self.height as i32),
            ],
            style_content,
        );
        let line_content_left =
            PathElement::new(vec![(0, 0), (0, self.height as i32)], style_content);
        let line_content_right = PathElement::new(
            vec![
                (self.width as i32, 0),
                (self.width as i32, self.height as i32),
            ],
            style_content,
        );

        self.content_area.draw(&line_content_top).unwrap();
        self.content_area.draw(&line_content_bottom).unwrap();
        self.content_area.draw(&line_content_left).unwrap();
        self.content_area.draw(&line_content_right).unwrap();
    }

    fn add_header_text(&self) {
        let now = Local::now();
        let weekday = now.format("%A");
        let month = now.format("%B");
        let day = now.format("%d");

        let text_style = ("Yanone Kaffeesatz", 40, FontStyle::Bold)
            .with_color(BLACK)
            .into_text_style(&self.content_area);
        let day = format!("{weekday}, {day} {month}");
        self.content_area
            .draw_text(&day, &text_style, (0, 10))
            .unwrap();
    }
}

pub async fn run_dashboard_task() {
    let mut buf = vec![0u8; (480 * 800 * 3) as usize];

    {
        let mut root = BitMapBackend::with_buffer(&mut buf, (480, 800)).into_drawing_area();

        root.fill(&RGBColor(251, 239, 194)).unwrap();

        let day_section = Section::new(50, 50, 15, 50, 50, &mut root);
        let weather_section = Section::new(200, 15, 15, 50, 50, &mut root);
        let tasks_section = Section::new(200, 15, 15, 50, 50, &mut root);
        let garbage_section = Section::new(160, 15, 50, 50, 50, &mut root);

        day_section.draw_line_borders();
        weather_section.draw_line_borders();
        tasks_section.draw_line_borders();
        garbage_section.draw_line_borders();

        day_section.add_header_text();
    }

    let img = RgbImage::from_raw(480, 800, buf).expect("buffer has wrong size");

    let mut img = image_to_pixmap(DynamicImage::ImageRgb8(img));
    convert_pixmap_to_epaper_pallete(&mut img);
    floyd_steinberg(&mut img);

    let image_bytes = img.encode_png().unwrap();
    save_image_to_storage(
        Dashboard {
            stage: Processed,
            day: Today,
            time_of_day: Morning,
        },
        image_bytes.clone(),
    )
    .await
    .unwrap();

    display_image_from_memory(
        Dashboard {
            stage: Processed,
            day: Today,
            time_of_day: Morning,
        },
        &image_bytes,
    )
    .await;
}
