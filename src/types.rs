#[derive(Debug, Clone, Copy)]
pub enum ImageType {
    NasaImage,
    Dashboard,
}

impl ImageType {
    pub fn as_str(self) -> &'static str {
        match self {
            ImageType::NasaImage => "nasa",
            ImageType::Dashboard => "dashboard",
        }
    }
}
