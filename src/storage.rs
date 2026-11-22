use crate::errors::{api_errors::NasaError, image_error::ImageError};
use crate::types::ImageType;
use tokio::io::AsyncWriteExt;
use tracing::info;

fn image_path(image_type: ImageType, date: chrono::NaiveDate) -> std::path::PathBuf {
    let name = match image_type {
        ImageType::NasaImage => "nasa".to_string(),
        ImageType::Dashboard => "dashboard".to_string(),
    };

    let file_name = format!("{}-{}.png", name, date.format("%Y-%m-%d"));

    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("output")
        .join(file_name)
}

pub async fn does_nasa_image_exist(date: &str) -> Result<bool, NasaError> {
    let project_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let output_filename = format!("nasa-{}.png", date);
    let output_path = project_root.join("output").join(output_filename);

    Ok(tokio::fs::try_exists(&output_path).await?)
}

pub async fn save_image(image_type: ImageType, image_bytes: Vec<u8>) -> Result<(), ImageError> {
    let today = chrono::Local::now().date_naive();
    let output_path = image_path(image_type, today);

    let mut file = tokio::fs::File::create_new(output_path.clone()).await?;
    file.write_all(&image_bytes).await?;

    info!("{} saved in {:?}", image_type.as_str(), &output_path);
    Ok(())
}

pub async fn load_image(
    image_type: ImageType,
    date: chrono::NaiveDate,
) -> Result<Vec<u8>, ImageError> {
    Ok(tokio::fs::read(image_path(image_type, date)).await?)
}
