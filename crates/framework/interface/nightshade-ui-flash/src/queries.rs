use crate::resources::{Flash, FlashHandles};
use nalgebra_glm::Vec4;
use nightshade::ecs::Entity;

pub fn built(handles: &FlashHandles) -> bool {
    handles.panel != Entity::default()
}

pub fn wash(flash: &Flash) -> Vec4 {
    let mut total = Vec4::new(0.0, 0.0, 0.0, 0.0);
    for held in &flash.washes {
        total += held.color * held.amount;
    }
    Vec4::new(
        total.x.min(1.0),
        total.y.min(1.0),
        total.z.min(1.0),
        total.w.min(1.0),
    )
}

pub fn fading(flash: &Flash) -> bool {
    !flash.washes.is_empty()
}

pub fn strength(flash: &Flash) -> f32 {
    flash
        .washes
        .iter()
        .map(|held| held.amount)
        .fold(0.0_f32, f32::max)
}
