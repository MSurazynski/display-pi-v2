use reqwest::Client;
use serde::Deserialize;

const WEATHER_URL: &str = "https://api.open-meteo.com/v1/forecast";

#[derive(Debug, Deserialize)]
pub struct RawWeatherResponse {
    pub hourly: RawHourlyWeather,
    pub daily: RawDailyWeather,
}

#[derive(Debug, Deserialize)]
pub struct RawHourlyWeather {
    pub time: Vec<String>,
    pub temperature_2m: Vec<f32>,
    pub precipitation: Vec<f32>,
    pub precipitation_probability: Vec<f32>,
}

#[derive(Debug, Deserialize)]
pub struct RawDailyWeather {
    pub time: Vec<String>,
    pub weather_code: Vec<i32>,
    pub temperature_2m_max: Vec<f32>,
    pub temperature_2m_min: Vec<f32>,
}

pub async fn fetch_wather() -> Result<RawWeatherResponse, reqwest::Error> {
    let client = Client::new();

    let response = client
        .get(WEATHER_URL)
        .query(&[
            ("latitude", "51.439270092728904"),
            ("longitude", "5.50632763399379"),
            (
                "hourly",
                "temperature_2m,weather_code,precipitation,precipitation_probability",
            ),
            (
                "daily",
                "weather_code,temperature_2m_max,temperature_2m_min",
            ),
            ("forecast_days", "2"),
        ])
        .send()
        .await?
        .error_for_status()?
        .json::<RawWeatherResponse>()
        .await?;

    Ok(response)
}
