use nalgebra_glm::Vec3;
pub use nightshade::ecs::sync::EngineEntity;

/// State carried by each wandering enemy entity in the sandbox world.
/// The matching render entity in the engine world is referenced via
/// the [`EngineEntity`] component; systems read the engine entity's
/// transform for the current position and push the new translation
/// back through the engine sync helpers.
#[derive(Default, Clone, Copy, Debug)]
pub struct Enemy {
    pub target: Vec3,
    pub speed: f32,
    pub patrol_index: u32,
    pub legs: [nightshade::prelude::Entity; 4],
    pub gait_phase: f32,
}
