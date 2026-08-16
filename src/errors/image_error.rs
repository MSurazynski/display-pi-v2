#[derive(Debug)]
pub enum ImageError {
    Decode,
    Crop,
    Encode(png::EncodingError),
    Io(std::io::Error),
}

impl From<std::io::Error> for ImageError {
    fn from(value: std::io::Error) -> Self {
        ImageError::Io(value)
    }
}

impl From<png::EncodingError> for ImageError {
    fn from(value: png::EncodingError) -> Self {
        ImageError::Encode(value)
    }
}
