use crate::systems::world::textures::{FLOOR_TEXTURE, prototype_material};
use nightshade::ecs::world::components;
use nightshade::ecs::world::{GLOBAL_TRANSFORM, LOCAL_TRANSFORM, LOCAL_TRANSFORM_DIRTY, NAME};
use nightshade::prelude::*;

pub const PLAYER_SPAWN_TAG: &str = "player_spawn";
pub const FLOOR_TAG: &str = "level_floor";

#[cfg(not(target_arch = "wasm32"))]
pub const LEVEL_PATH: &str = "apps/sandbox/sandbox.nsmap";

pub fn default_spawn_position() -> Vec3 {
    Vec3::new(0.0, 1.2, 0.0)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn load_or_build_level(world: &mut World) {
    use std::path::Path;

    let path = Path::new(LEVEL_PATH);
    if path.exists() {
        match load_scene(path) {
            Ok(scene) => {
                if let Err(error) = spawn_scene(world, &scene, None) {
                    tracing::error!("failed to spawn scene from {}: {}", LEVEL_PATH, error);
                    build_default_level(world);
                    save_current_level(world);
                }
            }
            Err(error) => {
                tracing::error!("failed to load {}: {}; rebuilding", LEVEL_PATH, error);
                build_default_level(world);
                save_current_level(world);
            }
        }
    } else {
        tracing::info!("no level file at {}, building default", LEVEL_PATH);
        build_default_level(world);
        save_current_level(world);
    }
}

#[cfg(target_arch = "wasm32")]
pub fn load_or_build_level(world: &mut World) {
    build_default_level(world);
}

#[cfg(not(target_arch = "wasm32"))]
pub fn save_current_level(world: &World) {
    use std::path::Path;

    let mut scene = world_to_scene(world, "Sandbox");
    if let Err(error) = save_scene(&mut scene, Path::new(LEVEL_PATH)) {
        tracing::error!("failed to save {}: {}", LEVEL_PATH, error);
    } else {
        tracing::info!("wrote level to {}", LEVEL_PATH);
    }
}

pub fn find_player_spawn(world: &World) -> Option<Vec3> {
    for (entity, tags) in &world.resources.entities.tags {
        if !tags.iter().any(|tag| tag == PLAYER_SPAWN_TAG) {
            continue;
        }
        if let Some(transform) = world.core.get_local_transform(*entity) {
            return Some(transform.translation);
        }
    }
    None
}

fn build_default_level(world: &mut World) {
    spawn_floor(world);
    spawn_sun(world);
    spawn_player_spawn_marker(world, default_spawn_position());
}

fn spawn_floor(world: &mut World) {
    let material = prototype_material(FLOOR_TEXTURE, Vec3::new(0.55, 0.6, 0.7), 0.85, 0.05);
    let entity = spawn_static_physics_cube_with_material(
        world,
        Vec3::new(0.0, -0.5, 0.0),
        Vec3::new(50.0, 1.0, 50.0),
        material,
    );
    if let Some(name) = world.core.get_name_mut(entity) {
        name.0 = "Floor".to_string();
    }
    world
        .resources
        .entities
        .tags
        .entry(entity)
        .or_default()
        .push(FLOOR_TAG.to_string());
    ensure_guid(world, entity);
}

fn spawn_player_spawn_marker(world: &mut World, position: Vec3) {
    let entity = spawn_entities(
        world,
        NAME | LOCAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY | GLOBAL_TRANSFORM,
        1,
    )[0];
    if let Some(name) = world.core.get_name_mut(entity) {
        name.0 = "PlayerSpawn".to_string();
    }
    world.core.set_local_transform(
        entity,
        components::LocalTransform {
            translation: position,
            rotation: Quat::identity(),
            scale: Vec3::new(1.0, 1.0, 1.0),
        },
    );
    world
        .core
        .set_local_transform_dirty(entity, components::LocalTransformDirty);
    world
        .core
        .set_global_transform(entity, components::GlobalTransform::default());
    world
        .resources
        .entities
        .tags
        .entry(entity)
        .or_default()
        .push(PLAYER_SPAWN_TAG.to_string());
    ensure_guid(world, entity);
}
