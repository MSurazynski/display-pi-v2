mod api;
mod dashboard;
mod epd7in3e;
mod errors;
mod parsers;
mod render;
mod storage;
mod types;

use crate::{
    api::nasa::run_nasa_task,
    api::tasks::{fetch_all_tasks, fetch_tasks_due_today},
    dashboard::make_dashboard,
};
use tracing::info;

fn start_logger() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    start_logger();
    info!("Starting");

    // run_nasa_task().await;
    make_dashboard().await?;

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
