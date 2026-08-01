use crate::resources::Ground;
use nalgebra_glm::{Vec2, Vec3};
use nightshade_ecs::Storage;

pub fn height_at(ground: &Ground, storage: &Storage, spot: Vec2) -> Option<f32> {
    let answer = ground.height_at?;
    answer(storage, spot)
}

pub fn solid_at(ground: &Ground, storage: &Storage, point: Vec3) -> bool {
    let Some(answer) = ground.solid_at else {
        return false;
    };
    answer(storage, point)
}

pub fn answers_height(ground: &Ground) -> bool {
    ground.height_at.is_some()
}

pub fn answers_solid(ground: &Ground) -> bool {
    ground.solid_at.is_some()
}
