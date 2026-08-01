use nalgebra_glm::{Vec2, Vec3};
use nightshade_ecs::Storage;

pub type HeightAnswer = fn(&Storage, Vec2) -> Option<f32>;
pub type SolidAnswer = fn(&Storage, Vec3) -> bool;

#[derive(Default)]
pub struct Ground {
    pub height_at: Option<HeightAnswer>,
    pub solid_at: Option<SolidAnswer>,
}
