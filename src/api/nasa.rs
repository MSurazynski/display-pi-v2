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
    pub media_type: String,
    pub url: String,
}

async fn fetch_nasa_image(
    client: &Client,
    base_url: &str,
    date: &str,
    api_key: &str,
) -> Result<Bytes, NasaError> {
    let response = client
        .get(base_url)
        .query(&[("api_key", api_key), ("date", date)])
        .send()
        .await?
        .error_for_status()?
        .json::<NasaData>()
        .await?;

    // Before download check if the media for today is actually an image
    if response.media_type != "image" {
        print!("APOD for today is not an image");
        return Err(NasaError::NotAnImage);
    }

    let image_bytes = client
        .get(&response.url)
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

// ----------------- TESTS -------------------

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn returns_error_on_500() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&server)
            .await;

        let client = Client::new();
        let result = fetch_nasa_image(&client, &server.uri(), "2026-01-01", "DEMO_KEY").await;

        assert!(matches!(result, Err(NasaError::Request(_))));
    }
    #[test]
    fn deserelizes_real_nasa_response() {
        let raw = include_str!("../../fixtures/nasa_reponse.json");
        let data: NasaData = serde_json::from_str(raw).unwrap();

        assert_eq!(data.media_type, "image");
        assert_eq!(
            data.url,
            "https://apod.nasa.gov/apod/image/2605/CometRigel_Karuk_960.jpg"
        );
    }

    #[tokio::test]
    async fn succeeds_on_second_attempt() {
        let server = MockServer::start().await;

        // First attempt: fail with 500. Only responds once.
        Mock::given(method("GET"))
            .and(path("/planetary/apod"))
            .respond_with(ResponseTemplate::new(500))
            .up_to_n_times(1)
            .with_priority(1) // higher priority = matched first
            .mount(&server)
            .await;

        // Later attempts: return valid JSON. Lower priority, so it only
        // matches after the 500 mock is exhausted.
        Mock::given(method("GET"))
            .and(path("/planetary/apod"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "media_type": "image",
                "url": format!("{}/image.png", server.uri())
            })))
            .with_priority(2)
            .mount(&server)
            .await;

        // The image download (second request in fetch_nasa_image).
        Mock::given(method("GET"))
            .and(path("/image.png"))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(vec![1, 2, 3]))
            .mount(&server)
            .await;

        let client = Client::new();
        let result = fetch_nasa_image_with_retries(
            &client,
            &format!("{}/planetary/apod", server.uri()),
            "2026-01-01",
            "DEMO_KEY",
            3,
        )
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().to_vec(), vec![1, 2, 3]);
    }

    #[tokio::test]
    async fn fails_after_exhausting_retries() {
        let server = MockServer::start().await;

        // Every request gets a 500 — success is never reached.
        Mock::given(method("GET"))
            .and(path("/planetary/apod"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&server)
            .await;

        let client = Client::new();
        let result = fetch_nasa_image_with_retries(
            &client,
            &format!("{}/planetary/apod", server.uri()),
            "2026-01-01",
            "DEMO_KEY",
            3,
        )
        .await;

        assert!(matches!(result, Err(NasaError::Request(_))));
    }
}
