use crate::systems::world::textures::{
    FLOOR_TEXTURE, OBSTACLE_TEXTURE, PLATFORM_TEXTURE, RAMP_TEXTURE, STAIRS_TEXTURE,
    prototype_material,
};
use nightshade::ecs::world::components;
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
    for (entity, tags) in &world.res::<EntityRegistry>().tags {
        if !tags.iter().any(|tag| tag == PLAYER_SPAWN_TAG) {
            continue;
        }
        if let Some(transform) =
            world.get::<nightshade::ecs::transform::components::LocalTransform>(*entity)
        {
            return Some(transform.translation);
        }
    }
    None
}

fn build_default_level(world: &mut World) {
    spawn_floor(world);
    spawn_start_zone_obstacles(world);
    spawn_sun(world);
    spawn_player_spawn_marker(world, default_spawn_position());
}

const STAIR_STEP_COUNT: usize = 8;
const STAIR_STEP_HEIGHT: f32 = 0.22;
const STAIR_STEP_DEPTH: f32 = 0.5;
const STAIR_WIDTH: f32 = 3.0;
const RAMP_LENGTH: f32 = 6.0;
const RAMP_THICKNESS: f32 = 0.3;
const RAMP_WIDTH: f32 = 3.0;

fn spawn_start_zone_obstacles(world: &mut World) {
    let white = Vec3::new(1.0, 1.0, 1.0);
    let stairs_material = prototype_material(STAIRS_TEXTURE, white, 0.85, 0.05);
    let ramp_material = prototype_material(RAMP_TEXTURE, white, 0.85, 0.05);
    let platform_material = prototype_material(PLATFORM_TEXTURE, white, 0.85, 0.05);
    let obstacle_material = prototype_material(OBSTACLE_TEXTURE, white, 0.85, 0.05);

    for step_index in 0..STAIR_STEP_COUNT {
        let height = STAIR_STEP_HEIGHT * (step_index + 1) as f32;
        spawn_static_block(
            world,
            &format!("Stair Step {}", step_index + 1),
            Vec3::new(
                0.0,
                height / 2.0,
                -4.0 - STAIR_STEP_DEPTH * step_index as f32,
            ),
            Vec3::new(STAIR_WIDTH, height, STAIR_STEP_DEPTH),
            stairs_material.clone(),
        );
    }
    let stairs_top_height = STAIR_STEP_HEIGHT * STAIR_STEP_COUNT as f32;
    spawn_static_block(
        world,
        "Stairs Platform",
        Vec3::new(
            0.0,
            stairs_top_height / 2.0,
            -4.0 - STAIR_STEP_DEPTH * STAIR_STEP_COUNT as f32 - 1.75,
        ),
        Vec3::new(4.0, stairs_top_height, 4.0),
        platform_material.clone(),
    );

    let step_test_heights = [0.15, 0.3, 0.45, 0.6, 0.75];
    for (step_index, height) in step_test_heights.into_iter().enumerate() {
        spawn_static_block(
            world,
            &format!("Step Test {}", step_index + 1),
            Vec3::new(6.0, height / 2.0, -4.0 + 2.0 * step_index as f32),
            Vec3::new(1.5, height, 1.5),
            platform_material.clone(),
        );
    }

    spawn_ramp(
        world,
        "Gentle Ramp",
        Vec3::new(-6.0, 0.0, -4.0),
        14.0_f32.to_radians(),
        ramp_material.clone(),
    );
    spawn_ramp(
        world,
        "Steep Ramp",
        Vec3::new(-10.0, 0.0, -4.0),
        28.0_f32.to_radians(),
        ramp_material,
    );

    for pillar_index in 0..4 {
        spawn_static_block(
            world,
            &format!("Pillar {}", pillar_index + 1),
            Vec3::new(-4.5 + 3.0 * pillar_index as f32, 1.25, 5.0),
            Vec3::new(0.8, 2.5, 0.8),
            obstacle_material.clone(),
        );
    }
    spawn_static_block(
        world,
        "Low Hurdle",
        Vec3::new(0.0, 0.225, 8.0),
        Vec3::new(3.0, 0.45, 0.4),
        obstacle_material.clone(),
    );
    spawn_static_block(
        world,
        "High Hurdle",
        Vec3::new(0.0, 0.45, 10.5),
        Vec3::new(3.0, 0.9, 0.4),
        obstacle_material.clone(),
    );
    spawn_static_block(
        world,
        "Wall Block",
        Vec3::new(5.0, 1.0, 8.0),
        Vec3::new(2.0, 2.0, 2.0),
        obstacle_material,
    );
}

fn spawn_static_block(
    world: &mut World,
    name: &str,
    position: Vec3,
    size: Vec3,
    material: nightshade::render::material::Material,
) -> Entity {
    let entity = spawn_static_physics_cube_with_material(world, position, size, material);
    if let Some(entity_name) = world.get_mut::<nightshade::ecs::primitives::Name>(entity) {
        entity_name.0 = name.to_string();
    }
    ensure_guid(world, entity);
    entity
}

fn spawn_ramp(
    world: &mut World,
    name: &str,
    base_position: Vec3,
    angle: f32,
    material: nightshade::render::material::Material,
) {
    let center_height = (RAMP_THICKNESS / 2.0) * angle.cos() + (RAMP_LENGTH / 2.0) * angle.sin();
    let entity = spawn_static_block(
        world,
        name,
        Vec3::new(base_position.x, center_height, base_position.z),
        Vec3::new(RAMP_WIDTH, RAMP_THICKNESS, RAMP_LENGTH),
        material,
    );
    if let Some(transform) =
        world.get_mut::<nightshade::ecs::transform::components::LocalTransform>(entity)
    {
        transform.rotation = nalgebra_glm::quat_angle_axis(angle, &Vec3::x());
    }
}

fn spawn_floor(world: &mut World) {
    let material = prototype_material(FLOOR_TEXTURE, Vec3::new(0.55, 0.6, 0.7), 0.85, 0.05);
    let entity = spawn_static_physics_cube_with_material(
        world,
        Vec3::new(0.0, -0.5, 0.0),
        Vec3::new(50.0, 1.0, 50.0),
        material,
    );
    if let Some(name) = world.get_mut::<nightshade::ecs::primitives::Name>(entity) {
        name.0 = "Floor".to_string();
    }
    world
        .res_mut::<EntityRegistry>()
        .tags
        .entry(entity)
        .or_default()
        .push(FLOOR_TAG.to_string());
    ensure_guid(world, entity);
}

fn spawn_player_spawn_marker(world: &mut World, position: Vec3) {
    let entity = world.spawn_with((
        Name("PlayerSpawn".to_string()),
        components::LocalTransform {
            translation: position,
            ..Default::default()
        },
        components::GlobalTransform::default(),
    ));
    world
        .res_mut::<EntityRegistry>()
        .tags
        .entry(entity)
        .or_default()
        .push(PLAYER_SPAWN_TAG.to_string());
    ensure_guid(world, entity);
}
