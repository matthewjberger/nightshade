use crate::ecs::SandboxResources;
use crate::systems::world::level::{default_spawn_position, find_player_spawn};
use nightshade::plugins::physics::commands::spawn_first_person_player;
use nightshade::prelude::*;
use nightshade::render::terrain_config::TerrainSettings;

pub const PLAYER_TAG: &str = "player";

const FLY_GROUND_CLEARANCE: f32 = 1.0;
const FLOOR_SURFACE_HEIGHT: f32 = 0.0;

pub fn clamp_fly_camera(sandbox_world: &SandboxResources, world: &mut World) {
    if !sandbox_world.player.fly {
        return;
    }
    let Some(fly_camera) = sandbox_world.player.fly_camera else {
        return;
    };
    let Some(translation) = world
        .get::<nightshade::ecs::transform::components::LocalTransform>(fly_camera)
        .map(|transform| transform.translation)
    else {
        return;
    };
    let ground = if world.plugin_resource::<TerrainSettings>().enabled {
        world
            .expect_resource::<TerrainRenderState>()
            .sample_height(translation.x, translation.z)
            .unwrap_or(translation.y - FLY_GROUND_CLEARANCE)
    } else {
        FLOOR_SURFACE_HEIGHT
    };
    let minimum_y = ground + FLY_GROUND_CLEARANCE;
    let mut resolved = translation;
    resolved.y = resolved.y.max(minimum_y);
    resolved = crate::systems::world::rocks::push_out_sphere(sandbox_world, resolved, 0.45);
    if (resolved - translation).norm_squared() > 0.0
        && let Some(transform) =
            world.get_mut::<nightshade::ecs::transform::components::LocalTransform>(fly_camera)
    {
        transform.translation = resolved;
    }
}

pub fn set_fly(sandbox_world: &mut SandboxResources, world: &mut World, enabled: bool) {
    sandbox_world.player.fly = enabled;
    let gameplay_camera = if sandbox_world.character.third_person {
        sandbox_world.character.camera
    } else {
        sandbox_world.player.camera_entity
    };
    if enabled {
        let pose = gameplay_camera
            .and_then(|camera| {
                world.get::<nightshade::ecs::transform::components::GlobalTransform>(camera)
            })
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
        if let Some(transform) =
            world.get_mut::<nightshade::ecs::transform::components::LocalTransform>(fly_camera)
        {
            transform.rotation = rotation;
        }
        sandbox_world.player.fly_camera = Some(fly_camera);
        world.expect_resource_mut::<ActiveCamera>().0 = Some(fly_camera);
        if let Some(player_entity) = sandbox_world.player.player_entity {
            world.remove_tag_type::<TerrainColliderAnchor>(player_entity);
        }
        world.add_tag_type::<TerrainColliderAnchor>(fly_camera);
    } else {
        world.expect_resource_mut::<ActiveCamera>().0 = gameplay_camera;
        if let Some(fly_camera) = sandbox_world.player.fly_camera.take() {
            world.remove_tag_type::<TerrainColliderAnchor>(fly_camera);
            despawn_entities_with_cache_cleanup(world, &[fly_camera]);
        }
        if let Some(player_entity) = sandbox_world.player.player_entity {
            world.add_tag_type::<TerrainColliderAnchor>(player_entity);
        }
    }
}

pub fn spawn(sandbox_world: &mut SandboxResources, world: &mut World) {
    let position = find_player_spawn(world).unwrap_or_else(default_spawn_position);
    let (player_entity, camera_entity) = spawn_first_person_player(world, position);
    world
        .resources
        .entities
        .tags
        .entry(player_entity)
        .or_default()
        .push(PLAYER_TAG.to_string());
    world.add_tag_type::<TerrainColliderAnchor>(player_entity);
    sandbox_world.player.player_entity = Some(player_entity);
    sandbox_world.player.camera_entity = Some(camera_entity);
    world.expect_resource_mut::<ActiveCamera>().0 = Some(camera_entity);
}

/// Teleport the player back to the spawn marker. Mirrors the game
/// app's `combat::reset_level` verbatim: rewrite local translation,
/// zero controller velocity, rewrite the physics-interpolation
/// snapshots, mark dirty.
pub fn reset(sandbox_world: &mut SandboxResources, world: &mut World) {
    let Some(player_entity) = sandbox_world.player.player_entity else {
        return;
    };
    let spawn = find_player_spawn(world).unwrap_or_else(default_spawn_position);

    if let Some(transform) =
        world.get_mut::<nightshade::ecs::transform::components::LocalTransform>(player_entity)
    {
        transform.translation = spawn;
    }
    if let Some(controller) = world
        .get_mut::<nightshade::plugins::physics::components::CharacterControllerComponent>(
        player_entity,
    ) {
        controller.velocity = Vec3::zeros();
    }
    if let Some(interpolation) = world
        .get_mut::<nightshade::plugins::physics::components::PhysicsInterpolation>(player_entity)
    {
        interpolation.previous_translation = spawn;
        interpolation.current_translation = spawn;
    }

    crate::systems::world::character::reset(sandbox_world, world);
}
