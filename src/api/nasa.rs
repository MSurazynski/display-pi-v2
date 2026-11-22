use crate::{
    epd7in3e::{Epd7in3e, FrameBuffer},
    errors::api_errors::NasaError,
    parsers::nasa::edit_nasa_image,
    storage::{does_nasa_image_exist, load_image, save_image},
    types::ImageType,
};

use bytes::Bytes;
use reqwest::Client;
use serde::Deserialize;
use tokio_retry::{
    Retry,
    strategy::{ExponentialBackoff, jitter},
};
use tracing::{error, info, instrument, warn};

#[derive(Debug, Deserialize)]
pub struct NasaData {
    pub date: String,
    pub media_type: String,
    pub hdurl: Option<String>,
}

#[instrument]
pub async fn run_nasa_task() {
    let client = Client::new();
    let date = chrono::Local::now().format("%Y-%m-%d").to_string();
    let nasa_url = "https://science.nasa.gov/wp-json/wp/v2/apod-basic/";
    let nasa_api_key = "DEMO_KEY";

    if does_nasa_image_exist(&date).await.unwrap() {
        info!("Nasa image already exists for today");
        display_nasa_image().await;
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

    display_nasa_image().await;
}

#[instrument]
async fn display_nasa_image() {
    info!("Starting to display NASA image");

    let mut epd = Epd7in3e::new().unwrap();
    let fb = FrameBuffer::from_png(
        &load_image(ImageType::NasaImage, chrono::Local::now().date_naive())
            .await
            .unwrap(),
    )
    .unwrap();
    epd.display(&fb).unwrap();

    info!("Finished displaying NASA image");
}

#[instrument(skip_all)]
async fn fetch_nasa_image(
    client: &Client,
    base_url: &str,
    date: &str,
    api_key: &str,
) -> Result<Bytes, NasaError> {
    let response = client
        .get(base_url)
        .query(&[("api_key", api_key)])
        .send()
        .await?
        .error_for_status()?
        .json::<Vec<NasaData>>()
        .await?;

    let apod = response
        .iter()
        .find(|d| d.date == date)
        .or_else(|| response.iter().max_by(|a, b| a.date.cmp(&b.date)))
        .ok_or(NasaError::NotFound)?;

    if apod.media_type != "image" {
        warn!("APOD for {} is not an image", apod.date);
        return Err(NasaError::NotAnImage);
    }

    let image_url = apod.hdurl.as_deref().ok_or(NasaError::NotAnImage)?;

    let image_bytes = client
        .get(image_url)
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?;

    Ok(image_bytes)
}

#[instrument(skip_all)]
async fn fetch_nasa_image_with_retries(
    client: &Client,
    base_url: &str,
    date: &str,
    api_key: &str,
    retries: usize,
) -> Result<Bytes, NasaError> {
    info!("Starting to fetch NASA image");
    let retry_strategy = ExponentialBackoff::from_millis(10)
        .map(jitter)
        .take(retries);

    let mut attempt = 0;
    let bytes = Retry::start(retry_strategy, || {
        attempt += 1;
        let attempt = attempt;
        async move {
            info!("Fetching NASA image, attempt = {}", attempt);
            fetch_nasa_image(client, base_url, date, api_key)
                .await
                .inspect_err(|e| error!(attempt, error = %e, "Fetch failed"))
        }
    })
    .await?;

    info!("Finished fetching NASA image");

    Ok(bytes)
}
