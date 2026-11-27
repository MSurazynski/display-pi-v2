use reqwest::Client;
use tracing::{info, instrument};

use crate::{
    api::nasa::fetch_nasa_image_with_retries,
    process::nasa::process,
    render::display::display_image_from_memory,
    storage::{does_raw_nasa_image_exist, load_image_from_storage, save_image_to_storage},
    types::ImageType::{NasaImageProcessed, NasaImageRaw},
};

#[instrument]
pub async fn run_nasa_task() {
    let client = Client::new();
    let nasa_url = "https://science.nasa.gov/wp-json/wp/v2/apod-basic/";
    let date = chrono::Local::now().date_naive();
    let nasa_api_key = "DEMO_KEY";

    let mut image_bytes = if does_raw_nasa_image_exist(chrono::Local::now().date_naive())
        .await
        .unwrap()
    {
        info!("Nasa image already exists for today");
        load_image_from_storage(NasaImageRaw, date).await.unwrap()
    } else {
        let image_bytes =
            fetch_nasa_image_with_retries(&client, nasa_url, &date.to_string(), nasa_api_key, 3)
                .await
                .unwrap();
        save_image_to_storage(NasaImageRaw, image_bytes.clone())
            .await
            .unwrap();
        image_bytes
    };

    process(&mut image_bytes).await;
    save_image_to_storage(NasaImageProcessed, image_bytes.clone())
        .await
        .unwrap();

    display_image_from_memory(NasaImageProcessed, &image_bytes).await;
}
