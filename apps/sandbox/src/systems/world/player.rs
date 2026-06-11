use crate::ecs::SandboxWorld;
use crate::systems::world::level::{default_spawn_position, find_player_spawn};
use nightshade::ecs::physics::commands::spawn_first_person_player;
use nightshade::prelude::*;

pub const PLAYER_TAG: &str = "player";

const FLY_GROUND_CLEARANCE: f32 = 1.0;
const FLOOR_SURFACE_HEIGHT: f32 = 0.0;

pub fn clamp_fly_camera(sandbox_world: &SandboxWorld, world: &mut World) {
    if !sandbox_world.resources.player.fly {
        return;
    }
    let Some(fly_camera) = sandbox_world.resources.player.fly_camera else {
        return;
    };
    let Some(translation) = world
        .core
        .get_local_transform(fly_camera)
        .map(|transform| transform.translation)
    else {
        return;
    };
    let ground = if world.resources.terrain.enabled {
        world
            .resources
            .terrain_render
            .sample_height(translation.x, translation.z)
            .unwrap_or(translation.y - FLY_GROUND_CLEARANCE)
    } else {
        FLOOR_SURFACE_HEIGHT
    };
    let minimum_y = ground + FLY_GROUND_CLEARANCE;
    let mut resolved = translation;
    resolved.y = resolved.y.max(minimum_y);
    resolved = crate::systems::world::rocks::push_out_sphere(sandbox_world, resolved, 0.45);
    if (resolved - translation).norm_squared() > 0.0 {
        if let Some(transform) = world.core.get_local_transform_mut(fly_camera) {
            transform.translation = resolved;
        }
        mark_local_transform_dirty(world, fly_camera);
    }
}

pub fn set_fly(sandbox_world: &mut SandboxWorld, world: &mut World, enabled: bool) {
    sandbox_world.resources.player.fly = enabled;
    if enabled {
        let pose = sandbox_world
            .resources
            .player
            .camera_entity
            .and_then(|camera| world.core.get_global_transform(camera))
            .map(|transform| {
                (
                    transform.translation(),
                    nalgebra_glm::to_quat(&nalgebra_glm::mat3_to_mat4(
                        &nalgebra_glm::mat4_to_mat3(&transform.0),
                    )),
                )
            });
        let Some((position, rotation)) = pose else {
            return;
        };
        let fly_camera =
            nightshade::ecs::camera::commands::spawn_camera(world, position, "Fly Camera".into());
        if let Some(transform) = world.core.get_local_transform_mut(fly_camera) {
            transform.rotation = rotation;
        }
        mark_local_transform_dirty(world, fly_camera);
        sandbox_world.resources.player.fly_camera = Some(fly_camera);
        world.resources.active_camera = Some(fly_camera);
    } else {
        world.resources.active_camera = sandbox_world.resources.player.camera_entity;
        if let Some(fly_camera) = sandbox_world.resources.player.fly_camera.take() {
            world.despawn_entities(&[fly_camera]);
        }
    }
}

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
