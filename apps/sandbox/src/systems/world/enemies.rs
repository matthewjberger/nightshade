use crate::ecs::{ENEMY, Enemy};
use nightshade::ecs::world::commands::spawn_material;
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

pub fn spawn(world: &mut World) {
    for index in 0..ENEMY_COUNT {
        let start_index = index as usize % PATROL_POINTS.len();
        let next_index = (start_index + 1) % PATROL_POINTS.len();
        let start = patrol_target(start_index);
        let next = patrol_target(next_index);

        let (sheep, legs) = spawn_visual(world, start, index);
        world.set(
            sheep,
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

pub fn update(world: &mut World) {
    let delta = world
        .expect_resource_mut::<nightshade::ecs::time::Time>()
        .delta_time;

    let mut body_writes: Vec<(Entity, Vec3, Option<f32>)> = Vec::new();
    let mut leg_writes: Vec<(Entity, f32, f32, f32)> = Vec::new();

    world
        .ecs
        .query_join::<(
            &mut Enemy,
            &nightshade::ecs::transform::components::LocalTransform,
        )>()
        .for_each(|sheep, (enemy, transform)| {
            let position = transform.translation;
            let to_target = enemy.target - position;
            let distance = to_target.norm();
            let step = (enemy.speed * delta).min(distance.max(0.0));

            let new_position = if distance > f32::EPSILON {
                position + (to_target / distance) * step
            } else {
                position
            };

            if distance <= TARGET_REACHED {
                let next_index = (enemy.patrol_index as usize + 1) % PATROL_POINTS.len();
                enemy.patrol_index = next_index as u32;
                enemy.target = patrol_target(next_index);
            }
            enemy.gait_phase += delta * enemy.speed * GAIT_FREQUENCY;

            let yaw = if distance > f32::EPSILON {
                Some(to_target.x.atan2(to_target.z))
            } else {
                None
            };
            body_writes.push((sheep, new_position, yaw));

            for (leg_index, leg) in enemy.legs.iter().enumerate() {
                let leg_phase = if leg_index == 0 || leg_index == 3 {
                    0.0
                } else {
                    std::f32::consts::PI
                };
                let swing = (enemy.gait_phase + leg_phase).sin() * LEG_SWING;
                let (offset_x, offset_z) = LEG_OFFSETS[leg_index];
                leg_writes.push((*leg, swing, offset_x, offset_z));
            }
        });

    for (sheep, new_position, yaw) in body_writes {
        if let Some(transform) =
            world.get_mut::<nightshade::ecs::transform::components::LocalTransform>(sheep)
        {
            transform.translation = new_position;
            if let Some(yaw) = yaw {
                transform.rotation = nalgebra_glm::quat_angle_axis(yaw, &Vec3::y());
            }
        }
    }
    for (leg, swing, offset_x, offset_z) in leg_writes {
        if let Some(transform) =
            world.get_mut::<nightshade::ecs::transform::components::LocalTransform>(leg)
        {
            transform.rotation = nalgebra_glm::quat_angle_axis(swing, &Vec3::x());
            transform.translation = Vec3::new(
                offset_x,
                LEG_HIP_HEIGHT - LEG_HALF_LENGTH * swing.cos(),
                offset_z - LEG_HALF_LENGTH * swing.sin(),
            );
        }
    }
}

/// Tears down every enemy. The render components and the enemy state
/// share one entity, so a single recursive despawn removes both. Pairs
/// with [`spawn`] so screen transitions can wipe the world.
pub fn despawn(world: &mut World) {
    let sheep: Vec<Entity> = world.ecs.worlds[GAME].query_entities(ENEMY).collect();
    for entity in sheep {
        despawn_recursive_immediate(world, entity);
    }
}

fn patrol_target(index: usize) -> Vec3 {
    let (x, z) = PATROL_POINTS[index % PATROL_POINTS.len()];
    Vec3::new(x, ENEMY_HEIGHT, z)
}

fn spawn_visual(world: &mut World, position: Vec3, index: u32) -> (Entity, [Entity; 4]) {
    let root = world.spawn_with((
        Name(format!("Sheep {}", index)),
        LocalTransform {
            translation: position,
            ..Default::default()
        },
        GlobalTransform::default(),
    ));

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
    let entity = world.spawn_with((
        Name(name.clone()),
        LocalTransform {
            translation,
            scale,
            ..Default::default()
        },
        GlobalTransform::default(),
        RenderMesh::new("Cube"),
        Visibility::default(),
        ChildOf(root),
    ));
    spawn_material(world, entity, name, material);
    entity
}
