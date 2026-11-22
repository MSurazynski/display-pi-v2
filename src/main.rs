mod api;
mod epd7in3e;
mod errors;
mod parsers;
mod render;
mod storage;
mod types;

use crate::{
    api::tasks::{fetch_all_tasks, fetch_tasks_due_today},
    epd7in3e::{Epd7in3e, FrameBuffer},
    storage::{does_nasa_image_exist, load_image},
    types::ImageType,
};
use reqwest::Client;
use tracing::{error, info, instrument};

use crate::{
    api::nasa::fetch_nasa_image_with_retries, parsers::nasa::edit_nasa_image, storage::save_image,
};

#[instrument]
async fn run_nasa_task() {
    let client = Client::new();
    let date = chrono::Local::now().format("%Y-%m-%d").to_string();
    let nasa_url = "https://science.nasa.gov/wp-json/wp/v2/apod-basic/";
    let nasa_api_key = "DEMO_KEY";

    if does_nasa_image_exist(&date).await.unwrap() {
        info!("Nasa image already exists for today");
        return;
    }

    let image_bytes = fetch_nasa_image_with_retries(&client, nasa_url, &date, nasa_api_key, 3)
        .await
        .unwrap();

    let image_bytes = edit_nasa_image(image_bytes).await.unwrap_or_else(|e| {
        tracing::error!("Failed to edit nasa image: {:?}", e);
        panic!("Failed to edit nasa image: {:?}", e)
    });
    save_image(ImageType::NasaImage, image_bytes).await.unwrap();
    info!("Nasa image saved");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    info!("Starting");

    run_nasa_task().await;

    let mut epd = Epd7in3e::new()?;
    let fb = FrameBuffer::from_png(
        &load_image(ImageType::Dashboard, chrono::Local::now().date_naive())
            .await
            .unwrap(),
    )?;
    epd.display(&fb)?;

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
