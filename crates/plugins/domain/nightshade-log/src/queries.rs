#[cfg(all(
    not(target_arch = "wasm32"),
    not(target_os = "android"),
    feature = "tracing"
))]
pub fn log_file_name(title: &str, config: &crate::data::LogConfig) -> String {
    use crate::data::LogRotation;

    let sanitized: String = title
        .chars()
        .map(|character| {
            if character.is_alphanumeric() || character == '-' || character == '_' {
                character
            } else {
                '_'
            }
        })
        .collect();
    let base_name = if sanitized.is_empty() {
        "app".to_string()
    } else {
        sanitized.to_lowercase()
    };
    match config.rotation {
        LogRotation::PerSession => {
            let timestamp = chrono::Local::now().format(&config.timestamp_format);
            format!("{base_name}_{timestamp}.log")
        }
        LogRotation::Daily | LogRotation::Never => format!("{base_name}.log"),
    }
}
