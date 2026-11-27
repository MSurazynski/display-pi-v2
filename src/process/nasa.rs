use image::{DynamicImage, imageops::FilterType};
use tracing::{info, instrument};

use crate::render::{
    dither::{convert_pixmap_to_epaper_pallete, floyd_steinberg},
    image_util::image_to_pixmap,
};

fn rotate_image_to_vertical(img: &mut DynamicImage) {
    if img.width() > img.height() {
        *img = img.rotate90();
    }
}

fn crop_image_to_largest_multiple(img: &mut DynamicImage) {
    let original_width = img.width() as f32;
    let original_height = img.height() as f32;

    let biggest_height_mul = img.height() as f32 / 800.0_f32;
    let biggest_width_mul = img.width() as f32 / 480.0_f32;

    let mut multiplier: f32 = 0_f32;
    let mut crop_x: f32 = 0_f32;
    let mut crop_y: f32 = 0_f32;
    let mut new_width: f32 = 0_f32;
    let mut new_height: f32 = 0_f32;

    if biggest_width_mul.ge(&biggest_height_mul) {
        multiplier = biggest_height_mul;

        new_height = 800_f32 * multiplier;
        new_width = 480_f32 * multiplier;

        crop_x = (original_width / 2_f32) - (new_width / 2_f32);
    } else {
        multiplier = biggest_width_mul;

        new_height = 800_f32 * multiplier;
        new_width = 480_f32 * multiplier;

        crop_y = (original_height / 2_f32) - (new_height / 2_f32);
    }

    let sub_img = image::imageops::crop(
        img,
        crop_x as u32,
        crop_y as u32,
        new_width as u32,
        new_height as u32,
    );

    *img = sub_img.to_image().into();
}

fn resize_image_to_size(img: &mut DynamicImage) {
    *img = img.resize(480, 800, FilterType::Gaussian)
}

#[instrument(skip_all)]
pub async fn process(image_bytes: &mut Vec<u8>) {
    let mut img = image::load_from_memory(image_bytes).unwrap();

    rotate_image_to_vertical(&mut img);

    crop_image_to_largest_multiple(&mut img);
    info!("Cropped image");
    resize_image_to_size(&mut img);
    info!("Resized image");

    let mut img = image_to_pixmap(img);
    convert_pixmap_to_epaper_pallete(&mut img);
    info!("Converted image to epaper pallete");
    floyd_steinberg(&mut img);
    info!("Applied floyed Steinberg to image");

    info!("Finished processing");
    *image_bytes = img.encode_png().unwrap();
}
