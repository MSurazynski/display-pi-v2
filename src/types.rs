#[derive(Debug, Clone, Copy)]
pub enum ImageType {
    NasaImageRaw,
    NasaImageProcessed,
    DashboardRaw,
    DashboardProcessed,
}

impl ImageType {
    pub fn as_str(self) -> &'static str {
        match self {
            ImageType::NasaImageRaw => "nasa-image-raw",
            ImageType::NasaImageProcessed => "nasa-image-processed",
            ImageType::DashboardRaw => "dashboard-image-raw",
            ImageType::DashboardProcessed => "dashboard-image-processed",
        }
    }
}
