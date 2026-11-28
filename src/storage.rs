use crate::errors::{api_errors::NasaError, image_error::ImageError};
use crate::types::ImageType::{self};
use crate::types::Stage::Raw;
use tokio::io::AsyncWriteExt;
use tracing::info;

pub fn image_path(image_type: ImageType, date: chrono::NaiveDate) -> std::path::PathBuf {
    let file_name = format!("{}-{}.png", image_type.as_str(), date.format("%Y-%m-%d"));

    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("output")
        .join(file_name)
}

pub async fn does_raw_nasa_image_exist(date: chrono::NaiveDate) -> Result<bool, NasaError> {
    let path = image_path(ImageType::Nasa { stage: Raw }, date);
    Ok(tokio::fs::try_exists(&path).await?)
}

pub async fn save_image_to_storage(
    image_type: ImageType,
    image_bytes: Vec<u8>,
) -> Result<(), ImageError> {
    let today = chrono::Local::now().date_naive();
    let output_path = image_path(image_type, today);

    info!(
        "Started saving {} in {:?}",
        image_type.as_str(),
        &output_path
    );

    let mut file = tokio::fs::File::create(output_path.clone()).await?;
    file.write_all(&image_bytes).await?;

    info!("{} saved in {:?}", image_type.as_str(), &output_path);
    Ok(())
}

pub async fn load_image_from_storage(
    image_type: ImageType,
    date: chrono::NaiveDate,
) -> Result<Vec<u8>, ImageError> {
    Ok(tokio::fs::read(image_path(image_type, date)).await?)
}
