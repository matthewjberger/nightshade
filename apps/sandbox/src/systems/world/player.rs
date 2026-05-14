use crate::ecs::SandboxWorld;
use crate::systems::world::level::{default_spawn_position, find_player_spawn};
use nightshade::ecs::physics::commands::spawn_first_person_player;
use nightshade::prelude::*;

pub const PLAYER_TAG: &str = "player";

pub fn spawn(sandbox_world: &mut SandboxWorld, world: &mut World) {
    let position = find_player_spawn(world).unwrap_or_else(default_spawn_position);
    let (player_entity, camera_entity) = spawn_first_person_player(world, position);
    world
        .resources
        .entities
        .tags
        .entry(player_entity)
        .or_default()
        .push(PLAYER_TAG.to_string());
    sandbox_world.resources.player.player_entity = Some(player_entity);
    sandbox_world.resources.player.camera_entity = Some(camera_entity);
    world.resources.active_camera = Some(camera_entity);
}

/// Teleport the player back to the spawn marker. Mirrors the game
/// app's `combat::reset_level` verbatim: rewrite local translation,
/// zero controller velocity, rewrite the physics-interpolation
/// snapshots, mark dirty.
pub fn reset(sandbox_world: &SandboxWorld, world: &mut World) {
    let Some(player_entity) = sandbox_world.resources.player.player_entity else {
        return;
    };
    let spawn = find_player_spawn(world).unwrap_or_else(default_spawn_position);

    if let Some(transform) = world.core.get_local_transform_mut(player_entity) {
        transform.translation = spawn;
    }
    if let Some(controller) = world.core.get_character_controller_mut(player_entity) {
        controller.velocity = Vec3::zeros();
    }
    if let Some(interpolation) = world.core.get_physics_interpolation_mut(player_entity) {
        interpolation.previous_translation = spawn;
        interpolation.current_translation = spawn;
    }
    mark_local_transform_dirty(world, player_entity);
}
