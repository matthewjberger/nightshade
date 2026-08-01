use crate::resources::{Toast, ToastSettings};

pub fn say(toast: &mut Toast, settings: &ToastSettings, line: impl Into<String>) {
    toast.line = line.into();
    toast.remaining = settings.showing;
}

pub fn hush(toast: &mut Toast) {
    toast.line.clear();
    toast.remaining = 0.0;
}
