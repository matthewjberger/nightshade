use crate::commands::spawn_ground;
use crate::data::Settings;
use crate::resources::SdfScene;
use nightshade::prelude::*;
use nightshade_config::prelude::Config;
use nightshade_physics::prelude::{PhysicsWorld, physics_world_wake_all};
use nightshade_sdf::resources::SdfCollision;

pub fn carry(
    mut collision: ResMut<SdfCollision>,
    mut scene: ResMut<SdfScene>,
    config: Res<Config<Settings>>,
    mut physics: ResMut<PhysicsWorld>,
    mut commands: Commands,
) {
    if !collision.gone.is_empty() || !collision.ready.is_empty() {
        physics_world_wake_all(&mut physics);
    }
    let friction = config.number(Settings::GroundFriction);
    for key in collision.gone.drain(..) {
        if let Some(chunk) = scene.ground.remove(&key) {
            commands.despawn_recursive(chunk);
        }
    }
    for (key, mesh) in collision.ready.drain(..) {
        if let Some(chunk) = scene.ground.remove(&key) {
            commands.despawn_recursive(chunk);
        }
        if mesh.triangles.is_empty() {
            continue;
        }
        let chunk = spawn_ground(&mut commands, mesh, friction);
        scene.ground.insert(key, chunk);
    }
}
