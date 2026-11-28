#[derive(Debug, Clone, Copy)]
pub enum Day {
    Today,
    Tomorrow,
}

#[derive(Debug, Clone, Copy)]
pub enum Stage {
    Raw,
    Processed,
}

#[derive(Debug, Clone, Copy)]
pub enum TimeOfDay {
    Morning,
    Evening,
}

#[derive(Debug, Clone, Copy)]
pub enum ImageType {
    Nasa {
        stage: Stage,
    },
    Dashboard {
        stage: Stage,
        day: Day,
        time_of_day: TimeOfDay,
    },
}

impl Day {
    pub fn as_str(self) -> &'static str {
        match self {
            Day::Today => "today",
            Day::Tomorrow => "tomorrow",
        }
    }
}

impl TimeOfDay {
    pub fn as_str(self) -> &'static str {
        match self {
            TimeOfDay::Morning => "morning",
            TimeOfDay::Evening => "evening",
        }
    }
}

impl Stage {
    pub fn as_str(self) -> &'static str {
        match self {
            Stage::Raw => "raw",
            Stage::Processed => "processed",
        }
    }
}

impl ImageType {
    pub fn as_str(self) -> String {
        match self {
            ImageType::Nasa { stage } => format!("nasa-image-{}", stage.as_str()).to_owned(),
            ImageType::Dashboard {
                stage,
                day,
                time_of_day,
            } => {
                format!(
                    "dashboard-image-{}-{}-{}",
                    stage.as_str(),
                    day.as_str(),
                    time_of_day.as_str()
                )
            }
        }
    }
}
