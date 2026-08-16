#[derive(Debug)]
pub enum NasaError {
    Request(reqwest::Error),
    NotAnImage,
    ImageAlreadyExisits,
    Io(std::io::Error),
}

impl From<reqwest::Error> for NasaError {
    fn from(value: reqwest::Error) -> Self {
        NasaError::Request(value)
    }
}

impl From<std::io::Error> for NasaError {
    fn from(value: std::io::Error) -> Self {
        NasaError::Io(value)
    }
}
