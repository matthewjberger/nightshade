//! Logging: the file-based log configuration, the tracing subscriber
//! installation, and [`LogPlugin`], which owns both. Composing the plugin
//! is what installs the subscribers; a program without it emits nothing.

use crate::app::{App, Plugin};

/// Controls how log files are rotated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogRotation {
    /// New log file each application launch (timestamp in filename).
    PerSession,
    /// One log file per day.
    Daily,
    /// Single log file, appended to on each launch.
    Never,
}

/// Configuration for file-based logging (desktop only, requires `tracing` feature).
#[derive(Debug, Clone)]
pub struct LogConfig {
    /// Directory for log files (relative to working directory).
    pub directory: String,
    /// Log rotation strategy.
    pub rotation: LogRotation,
    /// Default `RUST_LOG` filter when the environment variable is not set.
    pub default_filter: String,
    /// chrono format string for per-session log timestamps.
    /// Default: `"%Y-%m-%d_%H-%M-%S"` produces `2026-03-04_14-30-00`.
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

#[cfg(all(not(target_arch = "wasm32"), feature = "tracing"))]
pub fn log_file_name(title: &str, config: &LogConfig) -> String {
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

/// Installs the tracing subscribers and returns the appender guards, which
/// must live until the program exits so buffered lines flush. On native
/// with the `tracing` feature this is console plus a rolling log file, with
/// Tracy and chrome-trace layers when those features are on. Idempotent: a
/// second call leaves the first subscriber in place and returns no guards.
#[cfg(all(not(target_arch = "wasm32"), feature = "tracing"))]
pub fn initialize_logging(title: &str, config: &LogConfig) -> Vec<Box<dyn std::any::Any>> {
    use tracing_subscriber::EnvFilter;
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;

    let file_name = log_file_name(title, config);
    let file_appender = match config.rotation {
        LogRotation::Daily => tracing_appender::rolling::daily(&config.directory, &file_name),
        _ => tracing_appender::rolling::never(&config.directory, &file_name),
    };
    let (non_blocking, file_guard) = tracing_appender::non_blocking(file_appender);

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&config.default_filter));

    let registry = tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer())
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(non_blocking)
                .with_ansi(false),
        );

    let mut guards: Vec<Box<dyn std::any::Any>> = vec![Box::new(file_guard)];

    #[cfg(all(feature = "tracy", not(feature = "chrome")))]
    let initialized = registry
        .with(tracing_tracy::TracyLayer::default())
        .try_init()
        .is_ok();

    #[cfg(all(feature = "chrome", not(feature = "tracy")))]
    let initialized = {
        let (chrome_layer, chrome_guard) = tracing_chrome::ChromeLayerBuilder::new()
            .file("logs/trace.json")
            .build();
        guards.push(Box::new(chrome_guard));
        registry.with(chrome_layer).try_init().is_ok()
    };

    #[cfg(all(feature = "tracy", feature = "chrome"))]
    let initialized = {
        let (chrome_layer, chrome_guard) = tracing_chrome::ChromeLayerBuilder::new()
            .file("logs/trace.json")
            .build();
        guards.push(Box::new(chrome_guard));
        registry
            .with(tracing_tracy::TracyLayer::default())
            .with(chrome_layer)
            .try_init()
            .is_ok()
    };

    #[cfg(not(any(feature = "tracy", feature = "chrome")))]
    let initialized = registry.try_init().is_ok();

    if !initialized {
        guards.clear();
    }
    guards
}

/// The subscriber installation when file logging is unavailable: a plain
/// console subscriber on native without the `tracing` feature, the browser
/// console (plus the panic hook) on the web. Same idempotence contract as
/// the file form.
#[cfg(not(all(not(target_arch = "wasm32"), feature = "tracing")))]
pub fn initialize_logging(_title: &str, _config: &LogConfig) -> Vec<Box<dyn std::any::Any>> {
    #[cfg(not(target_arch = "wasm32"))]
    drop(tracing_subscriber::fmt().try_init());

    #[cfg(target_arch = "wasm32")]
    {
        console_error_panic_hook::set_once();
        let max_level = if cfg!(debug_assertions) {
            tracing::level_filters::LevelFilter::INFO
        } else {
            tracing::level_filters::LevelFilter::WARN
        };
        drop(
            tracing_subscriber::fmt()
                .with_max_level(max_level)
                .with_writer(tracing_web::MakeConsoleWriter)
                .without_time()
                .try_init(),
        );
    }
    Vec::new()
}

/// Installs logging at composition: the subscribers go up when the plugin
/// builds, and the appender guards ride the [`App`] so files flush until
/// the program exits. The log file is named after `title` when set, else
/// the window title as composed so far.
#[derive(Default)]
pub struct LogPlugin {
    pub title: Option<String>,
    pub config: LogConfig,
}

impl Plugin for LogPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(all(not(target_arch = "wasm32"), feature = "tracing"))]
        {
            app.world.resources.window.log_config = self.config.clone();
        }
        let title = self
            .title
            .clone()
            .unwrap_or_else(|| app.world.resources.window.title.clone());
        let guards = initialize_logging(&title, &self.config);
        app.log_guards.extend(guards);
    }
}
