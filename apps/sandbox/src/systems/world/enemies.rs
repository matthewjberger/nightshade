use crate::ecs::{ENEMY, ENGINE_ENTITY, Enemy, EngineEntity, SandboxWorld};
use nightshade::ecs::material::components::Material;
use nightshade::ecs::sync::sync_engine_translation;
use nightshade::ecs::world::commands::{spawn_cube_at, spawn_material};
use nightshade::prelude::*;

const ENEMY_COUNT: u32 = 5;
const ENEMY_SPEED: f32 = 1.6;
const ENEMY_SCALE: f32 = 0.6;
const ENEMY_HEIGHT: f32 = 0.3;
const TARGET_REACHED: f32 = 0.4;

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

        let engine_entity = spawn_visual(world, start, index);
        let game_entity = sandbox_world.spawn_entities(ENEMY | ENGINE_ENTITY, 1)[0];
        sandbox_world.set_engine_entity(game_entity, EngineEntity(engine_entity));
        sandbox_world.set_enemy(
            game_entity,
            Enemy {
                target: next,
                speed: ENEMY_SPEED,
                patrol_index: next_index as u32,
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

fn spawn_visual(world: &mut World, position: Vec3, index: u32) -> Entity {
    let entity = spawn_cube_at(world, position);
    if let Some(transform) = world.core.get_local_transform_mut(entity) {
        transform.scale = Vec3::new(ENEMY_SCALE, ENEMY_SCALE, ENEMY_SCALE);
    }
    if let Some(name) = world.core.get_name_mut(entity) {
        name.0 = format!("Enemy {}", index);
    }
    let hue = index as f32 / ENEMY_COUNT.max(1) as f32;
    let material = Material {
        base_color: [0.85, 0.35 + 0.4 * hue, 0.35, 1.0],
        roughness: 0.55,
        metallic: 0.05,
        emissive_factor: [0.20, 0.05, 0.05],
        ..Default::default()
    };
    spawn_material(world, entity, format!("sandbox_enemy_{}", index), material);
    entity
}
