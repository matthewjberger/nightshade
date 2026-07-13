use nalgebra_glm::Vec3;

/// State carried by each wandering enemy, stored in the app's member
/// world on the same entity that carries the engine-side render
/// components, so systems read and write the entity's transform
/// directly.
#[derive(Default, Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub struct Enemy {
    pub target: Vec3,
    pub speed: f32,
    pub patrol_index: u32,
    pub legs: [nightshade::prelude::Entity; 4],
    pub gait_phase: f32,
}
