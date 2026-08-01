#[derive(Default)]
pub struct Toast {
    pub line: String,
    pub remaining: f32,
}

pub struct ToastSettings {
    pub showing: f32,
    pub fading: f32,
}

impl Default for ToastSettings {
    fn default() -> Self {
        Self {
            showing: 2.5,
            fading: 0.5,
        }
    }
}
