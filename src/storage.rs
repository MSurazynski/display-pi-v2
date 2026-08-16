use crate::errors::{api_errors::NasaError, image_error::ImageError};
use crate::types::ImageType;
use tokio::io::AsyncWriteExt;

pub async fn does_nasa_image_exist(date: &str) -> Result<bool, NasaError> {
    let project_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let output_filename = format!("nasa-{}.png", date);
    let output_path = project_root.join("output").join(output_filename);

    Ok(tokio::fs::try_exists(&output_path).await?)
}

pub async fn save_image(image_type: ImageType, image_bytes: Vec<u8>) -> Result<(), ImageError> {
    let current_date = chrono::Local::now().format("%Y-%m-%d").to_string();

    let project_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));

    let name = match image_type {
        ImageType::NasaImage => "nasa".to_string(),
    };

    let output_filename = format!("{}-{}.png", &name, current_date);
    let output_path = project_root.join("output").join(output_filename);

    let mut file = tokio::fs::File::create_new(output_path).await?;
    file.write_all(&image_bytes).await?;

    Ok(())
}
