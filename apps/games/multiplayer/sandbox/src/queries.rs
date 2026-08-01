pub mod aim;
pub mod models;
pub mod step;

use crate::data::CHARACTERS;
use crate::data::assets::MODELS;
use crate::data::wire::PeerId;
use nalgebra_glm::Vec3;

pub fn model(name: &str) -> usize {
    MODELS
        .iter()
        .position(|file| file.name == name)
        .unwrap_or_else(|| panic!("the sandbox has no model named {name}"))
}

pub fn character_of(seat: usize) -> u8 {
    (seat % CHARACTERS.len()) as u8
}

pub fn spawn_of(seat: usize) -> Vec3 {
    let angle = seat as f32 * std::f32::consts::TAU / 8.0;
    Vec3::new(
        angle.sin() * crate::theme::SPAWN_RING,
        1.5,
        angle.cos() * crate::theme::SPAWN_RING,
    )
}

pub fn label(peer: PeerId) -> String {
    format!("player {peer}")
}
