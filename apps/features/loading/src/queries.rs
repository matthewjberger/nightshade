use crate::data::{MODELS, PICTURES, SOUNDS, TOTAL};
use crate::resources::Progress;
use crate::theme;

pub fn landed(progress: &Progress) -> usize {
    progress.models_landed + progress.pictures_landed + progress.sounds_landed
}

pub fn fill_width(progress: &Progress) -> f32 {
    let share = landed(progress) as f32 / TOTAL as f32;
    ((theme::BAR_WIDTH - theme::BAR_INSET * 2.0) * share.clamp(0.0, 1.0)).max(1.0)
}

pub fn count_text(progress: &Progress) -> String {
    format!("{} of {TOTAL}", landed(progress))
}

pub fn still_reading(progress: &Progress) -> String {
    format!(
        "models {} of {}   pictures {} of {}   sounds {} of {}",
        progress.models_landed,
        MODELS.len(),
        progress.pictures_landed,
        PICTURES,
        progress.sounds_landed,
        SOUNDS,
    )
}
