mod api;
mod errors;
mod parsers;
mod render;
mod storage;
mod types;
use std::time::Duration;

use crate::{storage::does_nasa_image_exist, types::ImageType};
use chrono::{Local, Timelike};
use render::dashboard::render_dashboard_svg;
use reqwest::Client;
use tokio::time::sleep;

use crate::{
    api::nasa::fetch_nasa_image_with_retries, parsers::nasa::edit_nasa_image, storage::save_image,
};

const RUN_HOURS: &[u32] = &[6, 18];

fn get_duration_till_next_run() -> Duration {
    let now = Local::now();
    let current_hour = now.hour();

    let next = RUN_HOURS.iter().find(|&&h| h > current_hour).copied();

    let target = match next {
        Some(h) => now.date_naive().and_hms_opt(h, 0, 0).unwrap(),
        None => {
            let tommorow = now.date_naive().succ_opt().unwrap();
            tommorow.and_hms_opt(RUN_HOURS[0], 0, 0).unwrap()
        }
    };

    let target = target.and_local_timezone(Local).unwrap();
    (target - now).to_std().unwrap_or(Duration::from_secs(0))
}

async fn run_nasa_task() {
    let client = Client::new();
    let date = chrono::Local::now().format("%Y-%m-%d").to_string();
    let nasa_url = "https://api.nasa.gov/planetary/apod";
    let nasa_api_key = "DEMO_KEY";

    if does_nasa_image_exist(&date).await.unwrap() {
        print!("Nasa image already exists for today");
        return;
    }

    let image_bytes = fetch_nasa_image_with_retries(&client, nasa_url, &date, nasa_api_key, 3)
        .await
        .unwrap();

    let image_bytes = edit_nasa_image(image_bytes).await.unwrap_or_else(|e| {
        panic!("failed to edit nasa image: {:?}", e);
    });
    save_image(ImageType::NasaImage, image_bytes).await.unwrap();
}

#[tokio::main]
async fn main() {
    // let raw_weather_data = fetch_wather().await.unwrap();
    // let weather_data = parse_weather_data(raw_weather_data).unwrap();

    // render_dashboard_svg();

    print!("Started");

    loop {
        let wait_time = get_duration_till_next_run();
        println!("Next run in {:?} hours", wait_time / 3600);

        tokio::select! {
            _ = sleep(wait_time) => {
                run_nasa_task().await;
            }
            _ = tokio::signal::ctrl_c() => {
                println!("Shutdown signal received, exiting");
                break;
            }
        }
    }
}
