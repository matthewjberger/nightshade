use crate::theme;

pub fn swing_angle(progress: f32, flip: bool) -> f32 {
    let eased = progress * progress * (3.0 - 2.0 * progress);
    let raised = theme::SWING_RAISED - theme::SWING_ARC * eased;
    if flip { -raised } else { raised }
}
