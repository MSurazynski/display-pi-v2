#[derive(Debug, thiserror::Error)]
pub enum NasaError {
    #[error("request failed: {0}")]
    Request(#[from] reqwest::Error),
    #[error("response is not an image")]
    NotAnImage,
    #[error("image already exists")]
    ImageAlreadyExists,
    #[error("image not found")]
    NotFound,
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug)]
pub enum TasksError {
    Request(reqwest::Error),
}

impl From<reqwest::Error> for TasksError {
    fn from(value: reqwest::Error) -> Self {
        TasksError::Request(value)
    }
}
