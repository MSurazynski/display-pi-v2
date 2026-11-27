use tracing::{info, instrument};

use crate::{
    epd7in3e::{Epd7in3e, FrameBuffer},
    storage::load_image_from_storage,
    types::ImageType,
};

#[instrument(skip_all)]
pub async fn display_image_from_storage(image_type: ImageType) {
    info!("Starting to display {}", image_type.as_str());

    let mut epd = Epd7in3e::new().unwrap();
    let fb = FrameBuffer::from_png(
        &load_image_from_storage(image_type, chrono::Local::now().date_naive())
            .await
            .unwrap(),
    )
    .unwrap();
    epd.display(&fb).unwrap();

    info!("Finished displaying {}", image_type.as_str());
}

#[instrument(skip_all)]
pub async fn display_image_from_memory(image_type: ImageType, image_bytes: &[u8]) {
    info!("Starting to display {}", image_type.as_str());

    let is_png = image::guess_format(&image_bytes)
        .map(|f| f == image::ImageFormat::Png)
        .unwrap_or(false);

    assert!(is_png);

    let mut epd = Epd7in3e::new().unwrap();
    let fb = FrameBuffer::from_png(image_bytes).unwrap();
    epd.display(&fb).unwrap();

    info!("Finished displaying {}", image_type.as_str());
}
