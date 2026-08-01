use crate::data::LogConfig;
use nightshade::app::App;
use nightshade::plugin::Plugin;

#[cfg(all(
    not(target_arch = "wasm32"),
    not(target_os = "android"),
    feature = "tracing"
))]
pub fn initialize_logging(title: &str, config: &LogConfig) -> Vec<Box<dyn std::any::Any>> {
    use tracing_subscriber::EnvFilter;
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;

    use crate::data::LogRotation;

    let file_name = crate::queries::log_file_name(title, config);
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

#[cfg(target_os = "android")]
pub fn initialize_logging(title: &str, config: &LogConfig) -> Vec<Box<dyn std::any::Any>> {
    use tracing_subscriber::EnvFilter;
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;

    let Ok(layer) = tracing_android::layer(title) else {
        return Vec::new();
    };
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        EnvFilter::new(format!(
            "{},wgpu_core=debug,wgpu_hal=debug",
            config.default_filter
        ))
    });
    drop(
        tracing_subscriber::registry()
            .with(env_filter)
            .with(layer)
            .try_init(),
    );
    Vec::new()
}

#[cfg(all(
    not(target_os = "android"),
    not(all(not(target_arch = "wasm32"), feature = "tracing"))
))]
pub fn initialize_logging(_title: &str, config: &LogConfig) -> Vec<Box<dyn std::any::Any>> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(&config.default_filter));
        drop(
            tracing_subscriber::fmt()
                .with_env_filter(env_filter)
                .try_init(),
        );
    }

    #[cfg(target_arch = "wasm32")]
    {
        console_error_panic_hook::set_once();
        let fallback = if cfg!(debug_assertions) {
            config.default_filter.as_str()
        } else {
            "warn"
        };
        let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(fallback));
        drop(
            tracing_subscriber::fmt()
                .with_env_filter(env_filter)
                .with_writer(tracing_web::MakeConsoleWriter)
                .without_time()
                .try_init(),
        );
    }
    Vec::new()
}

#[derive(Default)]
pub struct LogPlugin {
    pub title: Option<String>,
    pub config: LogConfig,
}

impl Plugin for LogPlugin {
    fn needs(&self) -> Vec<nightshade::plugin::Need> {
        nightshade::needs![
            nightshade_platform::contract::window::Window => "PlatformPlugin",
        ]
    }
    fn build(&self, app: &mut App) {
        let title = self.title.clone().unwrap_or_else(|| {
            app.world
                .res::<nightshade_platform::contract::window::Window>()
                .title
                .clone()
        });
        let guards = initialize_logging(&title, &self.config);
        for guard in guards {
            app.keep_alive(guard);
        }
    }
}
