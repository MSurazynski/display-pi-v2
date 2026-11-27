mod api;
mod dashboard;
mod epd7in3e;
mod errors;
mod parsers;
mod render;
mod storage;
mod types;

use crate::{
    api::{
        nasa::run_nasa_task,
        tasks::{fetch_all_tasks, fetch_tasks_due_today},
    },
    dashboard::make_dashboard,
    epd7in3e::{Epd7in3e, FrameBuffer},
    render::{
        dither::{convert_pixmap_to_epaper_pallete, floyd_steinberg},
        image_util::image_to_pixmap,
    },
    storage::{load_image, save_image},
    types::ImageType,
};
use image::{EncodableLayout, RgbaImage, imageops::FilterType};
use tokio::io::AsyncWriteExt;
use tracing::info;

fn start_logger() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();
}

async fn foo() {
    let image_bytes = load_image(ImageType::NasaImage, chrono::Local::now().date_naive())
        .await
        .unwrap();

    let mut img = image::load_from_memory(&image_bytes).unwrap();

    if img.width() > img.height() {
        img = img.rotate90();
    }

    let original_width = img.width() as f32;
    let original_height = img.height() as f32;

    let biggest_height_mul = img.height() as f32 / 800.0_f32;
    let biggest_width_mul = img.width() as f32 / 480.0_f32;

    let mut multiplier: f32 = 0_f32;
    let mut crop_x: f32 = 0_f32;
    let mut crop_y: f32 = 0_f32;
    let mut new_width: f32 = 0_f32;
    let mut new_height: f32 = 0_f32;

    if biggest_width_mul.ge(&biggest_height_mul) {
        multiplier = biggest_height_mul;

        new_height = 800_f32 * multiplier;
        new_width = 480_f32 * multiplier;

        crop_x = (original_width / 2_f32) - (new_width / 2_f32);
    } else {
        multiplier = biggest_width_mul;

        new_height = 800_f32 * multiplier;
        new_width = 480_f32 * multiplier;

        crop_y = (original_height / 2_f32) - (new_height / 2_f32);
    }

    let img = image::imageops::crop(
        &mut img,
        crop_x as u32,
        crop_y as u32,
        new_width as u32,
        new_height as u32,
    );

    let img: image::DynamicImage = img.to_image().into();

    let img = img.resize(480, 800, FilterType::Gaussian);
    let mut img = image_to_pixmap(img);

    convert_pixmap_to_epaper_pallete(&mut img);
    floyd_steinberg(&mut img);

    let image_bytes = img.encode_png().unwrap();

    // let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
    //     .join("output")
    //     .join("test.png");
    // let mut file = tokio::fs::File::create(path.clone()).await.unwrap();
    // file.write_all(&image_bytes).await.unwrap();

    let mut epd = Epd7in3e::new().unwrap();
    let fb = FrameBuffer::from_png(&image_bytes).unwrap();
    epd.display(&fb).unwrap();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    start_logger();
    info!("Starting");

    // run_nasa_task().await;
    foo().await;
    // make_dashboard().await?;

    info!("Finished");
    Ok(())

    // let raw_weather_data = fetch_wather().await.unwrap();
    // let weather_data = parse_weather_data(raw_weather_data).unwrap();
    // render_dashboard_svg();
    // let todoist_url = "https://api.todoist.com/api/v1/tasks";
    // let api_key = "82fe982a94f66c098447ac908a0939e505c24dea";
    // fetch_tasks_due_today(&client, todoist_url, api_key)
    //     .await
    //     .unwrap();
}
