use crate::resources::{Toast, ToastSettings};

pub fn showing(toast: &Toast) -> bool {
    toast.remaining > 0.0
}

pub fn opacity(toast: &Toast, settings: &ToastSettings) -> f32 {
    if settings.fading <= 0.0 {
        return 1.0;
    }
    (toast.remaining / settings.fading).clamp(0.0, 1.0)
}
