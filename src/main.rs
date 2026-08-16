mod api;
mod parsers;
mod render;

use render::dashboard::render_dashboard_svg;

use crate::api::nasa::fetch_nasa_image;

#[tokio::main]
async fn main() {
    // let raw_weather_data = fetch_wather().await.unwrap();
    // let weather_data = parse_weather_data(raw_weather_data).unwrap();

    // render_dashboard_svg();

    fetch_nasa_image().await.unwrap();
}
