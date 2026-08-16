use crate::render::dither::convert_pixmap_to_epaper_pallete;
use crate::render::image_util::{bytes_to_pixmap, crop, rotate_to_vertical};
use reqwest::Client;
use serde::Deserialize;
use tokio::io::AsyncWriteExt;

const NASA_URL: &str = "https://api.nasa.gov/planetary/apod";

#[derive(Debug, Deserialize)]
pub struct NasaData {
    pub media_type: String,
    pub url: String,
}

pub async fn fetch_nasa_image() -> Result<(), reqwest::Error> {
    let client = Client::new();
    let now = chrono::Local::now().format("%Y-%m-%d").to_string();

    let project_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let output_filename = format!("nasa-{}.png", now);
    let output_path = project_root.join("output").join(output_filename);

    let image_exists = tokio::fs::try_exists(&output_path).await.unwrap();

    if image_exists {
        println!("Image already exisits for today");
        return Ok(());
    }

    let response = client
        .get(NASA_URL)
        .query(&[("api_key", "DEMO_KEY"), ("date", &now)])
        .send()
        .await?
        .error_for_status()?
        .json::<NasaData>()
        .await?;

    let image_bytes = client
        .get(&response.url)
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?;

    if response.media_type != "image" {
        print!("APOD for today is not an image");
        return Ok(());
    }

    let mut img = bytes_to_pixmap(&image_bytes);

    rotate_to_vertical(&mut img);
    crop(&mut img, 480, 800).unwrap();
    convert_pixmap_to_epaper_pallete(&mut img);

    let image_bytes = img.encode_png().unwrap();

    let mut file = tokio::fs::File::create_new(output_path).await.unwrap();
    file.write_all(&image_bytes).await.unwrap();

    Ok(())
}
