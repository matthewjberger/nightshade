#[cfg(target_os = "android")]
static ANDROID_APP: std::sync::OnceLock<android_activity::AndroidApp> = std::sync::OnceLock::new();

#[cfg(target_os = "android")]
pub fn set_android_app(app: android_activity::AndroidApp) {
    drop(ANDROID_APP.set(app));
}

#[cfg(target_os = "android")]
pub fn android_app() -> Option<android_activity::AndroidApp> {
    ANDROID_APP.get().cloned()
}

#[cfg(target_os = "android")]
pub use android_activity;

#[macro_export]
macro_rules! android_main {
    ($launch:path) => {
        #[cfg(target_os = "android")]
        #[unsafe(no_mangle)]
        fn android_main(app: $crate::android::android_activity::AndroidApp) {
            $crate::android::set_android_app(app);
            if let Err(error) = $launch() {
                $crate::android::tracing::error!("{error}");
            }
            std::process::exit(0);
        }
    };
}

#[cfg(target_os = "android")]
pub use tracing;
