use crate::errors::image_error::ImageError;
use crate::render::dither::{convert_pixmap_to_epaper_pallete, floyd_steinberg};
use crate::render::image_util::{bytes_to_pixmap, crop, rotate_to_vertical};
use bytes::Bytes;

pub async fn edit_nasa_image(image_bytes: Bytes) -> Result<Vec<u8>, ImageError> {
    let mut img = bytes_to_pixmap(&image_bytes);

    rotate_to_vertical(&mut img);
    crop(&mut img, 480, 800).ok_or(ImageError::Crop)?;
    convert_pixmap_to_epaper_pallete(&mut img);
    floyd_steinberg(&mut img);

    let image_bytes = img.encode_png()?;

    Ok(image_bytes)
}
