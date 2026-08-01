use crate::resources::LofiSettings;

pub fn waiting(settings: &LofiSettings) -> bool {
    settings.waiting
}

pub fn scale(settings: &LofiSettings) -> f32 {
    settings.look.render_scale
}

pub fn snapping(settings: &LofiSettings) -> bool {
    settings.look.snap.is_some()
}
