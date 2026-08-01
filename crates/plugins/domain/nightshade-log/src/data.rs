#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogRotation {
    PerSession,
    Daily,
    Never,
}

#[derive(Debug, Clone)]
pub struct LogConfig {
    pub directory: String,
    pub rotation: LogRotation,
    pub default_filter: String,
    pub timestamp_format: String,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            directory: "logs".to_string(),
            rotation: LogRotation::PerSession,
            default_filter: "info".to_string(),
            timestamp_format: "%Y-%m-%d_%H-%M-%S".to_string(),
        }
    }
}
