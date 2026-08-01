use nalgebra_glm::Vec3;
use nightshade::prelude::Entity;

#[derive(Default, Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub struct Target(pub Vec3);

#[derive(Default, Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub struct Speed(pub f32);

#[derive(Default, Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub struct PatrolIndex(pub u32);

#[derive(Default, Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub struct Legs(pub [Entity; 4]);

#[derive(Default, Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub struct GaitPhase(pub f32);
