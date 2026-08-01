use crate::systems::world::textures::{
    FLOOR_TEXTURE, OBSTACLE_TEXTURE, PLATFORM_TEXTURE, RAMP_TEXTURE, STAIRS_TEXTURE,
    prototype_material,
};
use nightshade_graphics::prelude::*;
use nightshade_physics::prelude::*;
use nightshade_scene::contract as components;
use nightshade_scene::prelude::*;

pub const PLAYER_SPAWN_TAG: &str = "player_spawn";
pub const FLOOR_TAG: &str = "level_floor";

pub fn default_spawn_position() -> Vec3 {
    Vec3::new(0.0, 1.2, 0.0)
}

pub fn build_level(scene: &mut crate::commands::Scene<'_, '_>) {
    spawn_floor(scene);
    spawn_start_zone_obstacles(scene);
    spawn_sun(scene.commands);
    spawn_player_spawn_marker(scene, default_spawn_position());
}

pub fn find_player_spawn(scene: &crate::commands::Scene<'_, '_>) -> Option<Vec3> {
    for (entity, tags) in &scene.stage.entities.tags {
        if !tags.iter().any(|tag| tag == PLAYER_SPAWN_TAG) {
            continue;
        }
        if let Some(transform) = scene
            .storage
            .get::<nightshade_scene::contract::LocalTransform>(*entity)
        {
            return Some(transform.translation);
        }
    }
    None
}

const STAIR_STEP_COUNT: usize = 8;
const STAIR_STEP_HEIGHT: f32 = 0.22;
const STAIR_STEP_DEPTH: f32 = 0.5;
const STAIR_WIDTH: f32 = 3.0;
const RAMP_LENGTH: f32 = 6.0;
const RAMP_THICKNESS: f32 = 0.3;
const RAMP_WIDTH: f32 = 3.0;

fn spawn_start_zone_obstacles(scene: &mut crate::commands::Scene<'_, '_>) {
    let white = Vec3::new(1.0, 1.0, 1.0);
    let stairs_material = prototype_material(STAIRS_TEXTURE, white, 0.85, 0.05);
    let ramp_material = prototype_material(RAMP_TEXTURE, white, 0.85, 0.05);
    let platform_material = prototype_material(PLATFORM_TEXTURE, white, 0.85, 0.05);
    let obstacle_material = prototype_material(OBSTACLE_TEXTURE, white, 0.85, 0.05);

    for step_index in 0..STAIR_STEP_COUNT {
        let height = STAIR_STEP_HEIGHT * (step_index + 1) as f32;
        spawn_static_block(
            scene,
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
        scene,
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
            scene,
            &format!("Step Test {}", step_index + 1),
            Vec3::new(6.0, height / 2.0, -4.0 + 2.0 * step_index as f32),
            Vec3::new(1.5, height, 1.5),
            platform_material.clone(),
        );
    }

    spawn_ramp(
        scene,
        "Gentle Ramp",
        Vec3::new(-6.0, 0.0, -4.0),
        14.0_f32.to_radians(),
        ramp_material.clone(),
    );
    spawn_ramp(
        scene,
        "Steep Ramp",
        Vec3::new(-10.0, 0.0, -4.0),
        28.0_f32.to_radians(),
        ramp_material,
    );

    for pillar_index in 0..4 {
        spawn_static_block(
            scene,
            &format!("Pillar {}", pillar_index + 1),
            Vec3::new(-4.5 + 3.0 * pillar_index as f32, 1.25, 5.0),
            Vec3::new(0.8, 2.5, 0.8),
            obstacle_material.clone(),
        );
    }
    spawn_static_block(
        scene,
        "Low Hurdle",
        Vec3::new(0.0, 0.225, 8.0),
        Vec3::new(3.0, 0.45, 0.4),
        obstacle_material.clone(),
    );
    spawn_static_block(
        scene,
        "High Hurdle",
        Vec3::new(0.0, 0.45, 10.5),
        Vec3::new(3.0, 0.9, 0.4),
        obstacle_material.clone(),
    );
    spawn_static_block(
        scene,
        "Wall Block",
        Vec3::new(5.0, 1.0, 8.0),
        Vec3::new(2.0, 2.0, 2.0),
        obstacle_material,
    );
}

fn spawn_static_block(
    scene: &mut crate::commands::Scene<'_, '_>,
    name: &str,
    position: Vec3,
    size: Vec3,
    material: nightshade_render::data::material::Material,
) -> Entity {
    let entity = {
        let commands = &mut *scene.commands;
        spawn_static_physics_cube_with_material(commands, position, size, material)
    };
    let name = name.to_string();
    scene.commands.queue(move |storage, resources| {
        if let Some(entity_name) =
            storage.get_mut::<nightshade_scene::contract::primitives::Name>(entity)
        {
            entity_name.0 = name;
        }
        ensure_guid(
            storage,
            resources.res_mut::<nightshade_scene::resources::entity_registry::EntityRegistry>(),
            entity,
        );
    });
    entity
}

fn spawn_ramp(
    scene: &mut crate::commands::Scene<'_, '_>,
    name: &str,
    base_position: Vec3,
    angle: f32,
    material: nightshade_render::data::material::Material,
) {
    let center_height = (RAMP_THICKNESS / 2.0) * angle.cos() + (RAMP_LENGTH / 2.0) * angle.sin();
    let entity = spawn_static_block(
        scene,
        name,
        Vec3::new(base_position.x, center_height, base_position.z),
        Vec3::new(RAMP_WIDTH, RAMP_THICKNESS, RAMP_LENGTH),
        material,
    );
    scene.commands.queue(move |storage, _| {
        if let Some(transform) =
            storage.get_mut::<nightshade_scene::contract::LocalTransform>(entity)
        {
            transform.rotation = nalgebra_glm::quat_angle_axis(angle, &Vec3::x());
        }
    });
}

fn spawn_floor(scene: &mut crate::commands::Scene<'_, '_>) {
    let material = prototype_material(FLOOR_TEXTURE, Vec3::new(0.55, 0.6, 0.7), 0.85, 0.05);
    let entity = {
        let commands = &mut *scene.commands;
        spawn_static_physics_cube_with_material(
            commands,
            Vec3::new(0.0, -0.5, 0.0),
            Vec3::new(50.0, 1.0, 50.0),
            material,
        )
    };
    scene.commands.queue(move |storage, resources| {
        if let Some(name) = storage.get_mut::<nightshade_scene::contract::primitives::Name>(entity)
        {
            name.0 = "Floor".to_string();
        }
        ensure_guid(
            storage,
            resources.res_mut::<nightshade_scene::resources::entity_registry::EntityRegistry>(),
            entity,
        );
    });
    scene
        .stage
        .entities
        .tags
        .entry(entity)
        .or_default()
        .push(FLOOR_TAG.to_string());
}

fn spawn_player_spawn_marker(scene: &mut crate::commands::Scene<'_, '_>, position: Vec3) {
    let entity = scene.storage.spawn((
        Name("PlayerSpawn".to_string()),
        components::LocalTransform {
            translation: position,
            ..Default::default()
        },
        components::GlobalTransform::default(),
    ));
    scene
        .stage
        .entities
        .tags
        .entry(entity)
        .or_default()
        .push(PLAYER_SPAWN_TAG.to_string());
    ensure_guid(scene.storage, scene.stage.entities, entity);
}
