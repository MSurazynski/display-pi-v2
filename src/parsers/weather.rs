use crate::api::weather::RawWeatherResponse;
use chrono::{NaiveDate, NaiveDateTime, ParseError, ParseResult};
use itertools::izip;
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct WeatherData {
    pub hourly: HashMap<NaiveDateTime, HourlyForecast>,
    pub daily: HashMap<NaiveDate, DailyForecast>,
}

#[derive(Debug, Default)]
pub struct HourlyForecast {
    pub temperature: f32,
    pub precipitation: f32,
    pub precipitation_probability: f32,
}

#[derive(Debug, Default)]
pub struct DailyForecast {
    pub weather_code: i32,
    pub temperature_max: f32,
    pub temperature_min: f32,
}

pub fn parse_weather_data(raw_data: RawWeatherResponse) -> Result<WeatherData, ParseError> {
    let mut parsed_data: WeatherData = WeatherData::default();

    // Parse hourly weather

    for (time, temperature, precipitation, precipitation_probability) in izip!(
        raw_data.hourly.time,
        raw_data.hourly.temperature_2m,
        raw_data.hourly.precipitation,
        raw_data.hourly.precipitation_probability,
    ) {
        parsed_data.hourly.insert(
            NaiveDateTime::parse_from_str(&time, "%Y-%m-%dT%H:%M")?,
            HourlyForecast {
                temperature,
                precipitation,
                precipitation_probability,
            },
        );
    }

    for (time, weather_code, temperature_max, temperature_min) in izip!(
        raw_data.daily.time,
        raw_data.daily.weather_code,
        raw_data.daily.temperature_2m_max,
        raw_data.daily.temperature_2m_min,
    ) {
        parsed_data.daily.insert(
            NaiveDate::parse_from_str(&time, "%Y-%m-%d")?,
            DailyForecast {
                weather_code,
                temperature_max,
                temperature_min,
            },
        );
    }

    Ok(parsed_data)
}
