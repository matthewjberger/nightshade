use crate::ecs::SandboxWorld;
use crate::systems::world::textures::{FLOOR_TEXTURE, prototype_material};
use nightshade::prelude::*;

const GROUND_SIZE: f32 = 4000.0;
const TEXTURE_PERIOD: f32 = 12.5;
const GROUND_CENTER_HEIGHT: f32 = -0.51;
const PARKED_HEIGHT: f32 = -10_000.0;

/// The ground slab is one giant static physics cube that follows the
/// player, snapped to the floor texture's repeat period so the motion
/// is imperceptible. Its top sits a hair below the level floor so the
/// two never z-fight where they overlap. The slab is parked far below
/// the world instead of despawned when the mode turns off.
pub fn set_enabled(sandbox_world: &mut SandboxWorld, world: &mut World, enabled: bool) {
    if sandbox_world.resources.infinite_world.enabled == enabled {
        return;
    }
    sandbox_world.resources.infinite_world.enabled = enabled;
    let ground = ensure_ground(sandbox_world, world);
    let height = if enabled {
        GROUND_CENTER_HEIGHT
    } else {
        PARKED_HEIGHT
    };
    if let Some(transform) = world.core.get_local_transform_mut(ground) {
        transform.translation.y = height;
    }
    mark_local_transform_dirty(world, ground);
    if enabled {
        recenter(sandbox_world, world);
    }
}

pub fn update(sandbox_world: &SandboxWorld, world: &mut World) {
    if !sandbox_world.resources.infinite_world.enabled {
        return;
    }
    recenter(sandbox_world, world);
}

fn recenter(sandbox_world: &SandboxWorld, world: &mut World) {
    let Some(ground) = sandbox_world.resources.infinite_world.ground else {
        return;
    };
    let Some(player_entity) = sandbox_world.resources.player.player_entity else {
        return;
    };
    let Some(player_translation) = world
        .core
        .get_local_transform(player_entity)
        .map(|transform| transform.translation)
    else {
        return;
    };
    let snapped_x = (player_translation.x / TEXTURE_PERIOD).round() * TEXTURE_PERIOD;
    let snapped_z = (player_translation.z / TEXTURE_PERIOD).round() * TEXTURE_PERIOD;
    let mut moved = false;
    if let Some(transform) = world.core.get_local_transform_mut(ground)
        && (transform.translation.x != snapped_x || transform.translation.z != snapped_z)
    {
        transform.translation.x = snapped_x;
        transform.translation.z = snapped_z;
        moved = true;
    }
    if moved {
        mark_local_transform_dirty(world, ground);
    }
}

fn ensure_ground(sandbox_world: &mut SandboxWorld, world: &mut World) -> Entity {
    if let Some(ground) = sandbox_world.resources.infinite_world.ground {
        return ground;
    }
    let mut material = prototype_material(FLOOR_TEXTURE, Vec3::new(0.55, 0.6, 0.7), 0.85, 0.05);
    let repeats = GROUND_SIZE / TEXTURE_PERIOD;
    material.base_texture_transform.scale = [repeats, repeats];
    let entity = spawn_static_physics_cube_with_material(
        world,
        Vec3::new(0.0, PARKED_HEIGHT, 0.0),
        Vec3::new(GROUND_SIZE, 1.0, GROUND_SIZE),
        material,
    );
    if let Some(name) = world.core.get_name_mut(entity) {
        name.0 = "InfiniteGround".to_string();
    }
    sandbox_world.resources.infinite_world.ground = Some(entity);
    entity
}
