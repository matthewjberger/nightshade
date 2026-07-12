use crate::ecs::{ENEMY, ENGINE_ENTITY, Enemy, EngineEntity, SandboxWorld};
use nightshade::ecs::sync::sync_engine_translation;
use nightshade::ecs::world::commands::{spawn_entities, spawn_material};
use nightshade::prelude::*;
use nightshade::render::material::Material;

const ENEMY_COUNT: u32 = 5;
const ENEMY_SPEED: f32 = 1.6;
const ENEMY_HEIGHT: f32 = 0.0;
const TARGET_REACHED: f32 = 0.4;
const LEG_HIP_HEIGHT: f32 = 0.42;
const LEG_HALF_LENGTH: f32 = 0.21;
const LEG_SWING: f32 = 0.45;
const GAIT_FREQUENCY: f32 = 4.0;
const LEG_OFFSETS: [(f32, f32); 4] = [(0.18, 0.3), (-0.18, 0.3), (0.18, -0.3), (-0.18, -0.3)];

/// Patrol waypoints the enemies cycle through. Defined as `(x, z)` so
/// they can be `const`; the runtime lifts each pair to a `Vec3` at the
/// shared enemy height.
const PATROL_POINTS: &[(f32, f32)] = &[
    (16.0, 16.0),
    (-16.0, 16.0),
    (-16.0, -16.0),
    (16.0, -16.0),
    (0.0, 18.0),
    (0.0, -18.0),
    (18.0, 0.0),
    (-18.0, 0.0),
];

pub fn spawn(sandbox_world: &mut SandboxWorld, world: &mut World) {
    for index in 0..ENEMY_COUNT {
        let start_index = index as usize % PATROL_POINTS.len();
        let next_index = (start_index + 1) % PATROL_POINTS.len();
        let start = patrol_target(start_index);
        let next = patrol_target(next_index);

        let (engine_entity, legs) = spawn_visual(world, start, index);
        let game_entity = sandbox_world.spawn_entities(ENEMY | ENGINE_ENTITY, 1)[0];
        sandbox_world.set_engine_entity(game_entity, EngineEntity(engine_entity));
        sandbox_world.set_enemy(
            game_entity,
            Enemy {
                target: next,
                speed: ENEMY_SPEED,
                patrol_index: next_index as u32,
                legs,
                gait_phase: index as f32 * 1.3,
            },
        );
    }
}

pub fn update(sandbox_world: &mut SandboxWorld, world: &mut World) {
    let delta = world.resources.window.timing.delta_time;

    let snapshots: Vec<(freecs::Entity, Entity, Enemy, Vec3)> = sandbox_world
        .query_entities(ENEMY | ENGINE_ENTITY)
        .filter_map(|game_entity| {
            let engine_entity = sandbox_world.get_engine_entity(game_entity)?.0;
            let enemy = *sandbox_world.get_enemy(game_entity)?;
            let position = world.core.get_local_transform(engine_entity)?.translation;
            Some((game_entity, engine_entity, enemy, position))
        })
        .collect();

    let mut translations: Vec<(Entity, Vec3)> = Vec::with_capacity(snapshots.len());
    let mut state_updates: Vec<(freecs::Entity, Enemy)> = Vec::with_capacity(snapshots.len());

    for (game_entity, engine_entity, enemy, position) in snapshots {
        let to_target = enemy.target - position;
        let distance = to_target.norm();
        let step = (enemy.speed * delta).min(distance.max(0.0));

        let new_position = if distance > f32::EPSILON {
            position + (to_target / distance) * step
        } else {
            position
        };

        let mut new_state = enemy;
        if distance <= TARGET_REACHED {
            let next_index = (enemy.patrol_index as usize + 1) % PATROL_POINTS.len();
            new_state.patrol_index = next_index as u32;
            new_state.target = patrol_target(next_index);
        }
        new_state.gait_phase = enemy.gait_phase + delta * enemy.speed * GAIT_FREQUENCY;

        if distance > f32::EPSILON {
            let yaw = to_target.x.atan2(to_target.z);
            if let Some(transform) = world.core.get_local_transform_mut(engine_entity) {
                transform.rotation = nalgebra_glm::quat_angle_axis(yaw, &Vec3::y());
            }
        }

        for (leg_index, leg) in enemy.legs.iter().enumerate() {
            let leg_phase = if leg_index == 0 || leg_index == 3 {
                0.0
            } else {
                std::f32::consts::PI
            };
            let swing = (new_state.gait_phase + leg_phase).sin() * LEG_SWING;
            let (offset_x, offset_z) = LEG_OFFSETS[leg_index];
            if let Some(transform) = world.core.get_local_transform_mut(*leg) {
                transform.rotation = nalgebra_glm::quat_angle_axis(swing, &Vec3::x());
                transform.translation = Vec3::new(
                    offset_x,
                    LEG_HIP_HEIGHT - LEG_HALF_LENGTH * swing.cos(),
                    offset_z - LEG_HALF_LENGTH * swing.sin(),
                );
            }
        }

        translations.push((engine_entity, new_position));
        state_updates.push((game_entity, new_state));
    }

    sync_engine_translation(translations, world);
    for (game_entity, new_state) in state_updates {
        if let Some(enemy) = sandbox_world.get_enemy_mut(game_entity) {
            *enemy = new_state;
        }
    }
}

/// Tears down every enemy: drops the engine-side render entity for
/// each one, then despawns the sandbox-side bookkeeping entity. Pairs
/// with [`spawn`] so screen transitions can wipe the world.
pub fn despawn(sandbox_world: &mut SandboxWorld, world: &mut World) {
    let engine_entities: Vec<Entity> = sandbox_world
        .query_entities(ENEMY | ENGINE_ENTITY)
        .filter_map(|game_entity| sandbox_world.get_engine_entity(game_entity).map(|e| e.0))
        .collect();
    for engine_entity in engine_entities {
        despawn_recursive_immediate(world, engine_entity);
    }
    let game_entities: Vec<freecs::Entity> = sandbox_world.query_entities(ENEMY).collect();
    if !game_entities.is_empty() {
        sandbox_world.despawn_entities(&game_entities);
    }
}

fn patrol_target(index: usize) -> Vec3 {
    let (x, z) = PATROL_POINTS[index % PATROL_POINTS.len()];
    Vec3::new(x, ENEMY_HEIGHT, z)
}

fn spawn_visual(world: &mut World, position: Vec3, index: u32) -> (Entity, [Entity; 4]) {
    let root = spawn_entities(world, NAME | LOCAL_TRANSFORM | GLOBAL_TRANSFORM, 1)[0];
    if let Some(name) = world.core.get_name_mut(root) {
        name.0 = format!("Sheep {}", index);
    }
    if let Some(transform) = world.core.get_local_transform_mut(root) {
        transform.translation = position;
    }

    let wool = Material {
        base_color: [0.93, 0.91, 0.87, 1.0],
        roughness: 0.95,
        metallic: 0.0,
        ..Default::default()
    };
    let hide = Material {
        base_color: [0.13, 0.11, 0.1, 1.0],
        roughness: 0.8,
        metallic: 0.0,
        ..Default::default()
    };

    spawn_sheep_part(
        world,
        root,
        format!("sheep_body_{}", index),
        Vec3::new(0.0, 0.62, 0.0),
        Vec3::new(0.55, 0.5, 0.85),
        wool.clone(),
    );
    spawn_sheep_part(
        world,
        root,
        format!("sheep_head_{}", index),
        Vec3::new(0.0, 0.78, 0.55),
        Vec3::new(0.26, 0.28, 0.3),
        hide.clone(),
    );
    spawn_sheep_part(
        world,
        root,
        format!("sheep_tuft_{}", index),
        Vec3::new(0.0, 0.96, 0.5),
        Vec3::new(0.2, 0.12, 0.18),
        wool,
    );

    let mut legs = [Entity::default(); 4];
    for (leg_index, (offset_x, offset_z)) in LEG_OFFSETS.iter().enumerate() {
        legs[leg_index] = spawn_sheep_part(
            world,
            root,
            format!("sheep_leg_{}_{}", index, leg_index),
            Vec3::new(*offset_x, LEG_HIP_HEIGHT - LEG_HALF_LENGTH, *offset_z),
            Vec3::new(0.1, LEG_HALF_LENGTH * 2.0, 0.1),
            hide.clone(),
        );
    }

    (root, legs)
}

fn spawn_sheep_part(
    world: &mut World,
    root: Entity,
    name: String,
    translation: Vec3,
    scale: Vec3,
    material: Material,
) -> Entity {
    let entity = spawn_entities(
        world,
        NAME | LOCAL_TRANSFORM
            | GLOBAL_TRANSFORM
            | RENDER_MESH
            | MATERIAL_REF
            | VISIBILITY
            | PARENT,
        1,
    )[0];
    if let Some(part_name) = world.core.get_name_mut(entity) {
        part_name.0 = name.clone();
    }
    if let Some(transform) = world.core.get_local_transform_mut(entity) {
        transform.translation = translation;
        transform.scale = scale;
    }
    world.core.set_parent(entity, Parent(Some(root)));
    if let Some(mesh) = world.core.get_render_mesh_mut(entity) {
        mesh.name = "Cube".to_string();
    }
    spawn_material(world, entity, name, material);
    entity
}
