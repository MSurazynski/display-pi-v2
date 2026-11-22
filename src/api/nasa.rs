use crate::errors::api_errors::NasaError;

use bytes::Bytes;
use reqwest::Client;
use serde::Deserialize;
use tokio_retry::{
    Retry,
    strategy::{ExponentialBackoff, jitter},
};

#[derive(Debug, Deserialize)]
pub struct NasaData {
    pub date: String,
    pub media_type: String,
    pub hdurl: Option<String>,
}

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
        eprintln!("APOD for {} is not an image", apod.date);
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

pub async fn fetch_nasa_image_with_retries(
    client: &Client,
    base_url: &str,
    date: &str,
    api_key: &str,
    retries: usize,
) -> Result<Bytes, NasaError> {
    let retry_strategy = ExponentialBackoff::from_millis(10)
        .map(jitter)
        .take(retries);

    let bytes = Retry::start(retry_strategy, || {
        fetch_nasa_image(client, base_url, date, api_key)
    })
    .await?;

    Ok(bytes)
}
