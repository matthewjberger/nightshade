use crate::components::{GaitPhase, Legs, PatrolIndex, Speed, Target};
use nightshade::prelude::*;
use nightshade_graphics::commands::builtins::spawn_material;
use nightshade_graphics::prelude::*;
use nightshade_render::data::material::Material;

const ENEMY_COUNT: u32 = 5;
const ENEMY_SPEED: f32 = 1.6;
const ENEMY_HEIGHT: f32 = 0.0;
const TARGET_REACHED: f32 = 0.4;
const LEG_HIP_HEIGHT: f32 = 0.42;
const LEG_HALF_LENGTH: f32 = 0.21;
const LEG_SWING: f32 = 0.45;
const GAIT_FREQUENCY: f32 = 4.0;
const LEG_OFFSETS: [(f32, f32); 4] = [(0.18, 0.3), (-0.18, 0.3), (0.18, -0.3), (-0.18, -0.3)];

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

pub fn spawn(scene: &mut crate::commands::Scene<'_, '_>) {
    for index in 0..ENEMY_COUNT {
        let start_index = index as usize % PATROL_POINTS.len();
        let next_index = (start_index + 1) % PATROL_POINTS.len();
        let start = patrol_target(start_index);
        let next = patrol_target(next_index);

        let (sheep, legs) = spawn_visual(scene, start, index);
        scene.storage.set(sheep, Target(next));
        scene.storage.set(sheep, Speed(ENEMY_SPEED));
        scene.storage.set(sheep, PatrolIndex(next_index as u32));
        scene.storage.set(sheep, Legs(legs));
        scene.storage.set(sheep, GaitPhase(index as f32 * 1.3));
    }
}

pub fn update(
    stage: crate::commands::Stage,
    mut commands: Commands,
    storage: &mut nightshade::ecs::Storage,
) {
    let scene = &mut crate::commands::Scene {
        storage,
        commands: &mut commands,
        stage,
    };
    let delta = scene.stage.time.delta_time;

    let mut body_writes: Vec<(Entity, Vec3, Option<f32>)> = Vec::new();
    let mut leg_writes: Vec<(Entity, f32, f32, f32)> = Vec::new();

    scene
        .storage
        .query::<(
            &mut Target,
            &Speed,
            &mut PatrolIndex,
            &Legs,
            &mut GaitPhase,
            &nightshade_scene::contract::LocalTransform,
        )>()
        .for_each(
            |sheep, (target, speed, patrol_index, legs, gait_phase, transform)| {
                let position = transform.translation;
                let to_target = target.0 - position;
                let distance = to_target.norm();
                let step = (speed.0 * delta).min(distance.max(0.0));

                let new_position = if distance > f32::EPSILON {
                    position + (to_target / distance) * step
                } else {
                    position
                };

                if distance <= TARGET_REACHED {
                    let next_index = (patrol_index.0 as usize + 1) % PATROL_POINTS.len();
                    patrol_index.0 = next_index as u32;
                    target.0 = patrol_target(next_index);
                }
                gait_phase.0 += delta * speed.0 * GAIT_FREQUENCY;

                let yaw = if distance > f32::EPSILON {
                    Some(to_target.x.atan2(to_target.z))
                } else {
                    None
                };
                body_writes.push((sheep, new_position, yaw));

                for (leg_index, leg) in legs.0.iter().enumerate() {
                    let leg_phase = if leg_index == 0 || leg_index == 3 {
                        0.0
                    } else {
                        std::f32::consts::PI
                    };
                    let swing = (gait_phase.0 + leg_phase).sin() * LEG_SWING;
                    let (offset_x, offset_z) = LEG_OFFSETS[leg_index];
                    leg_writes.push((*leg, swing, offset_x, offset_z));
                }
            },
        );

    for (sheep, new_position, yaw) in body_writes {
        if let Some(transform) = scene
            .storage
            .get_mut::<nightshade_scene::contract::LocalTransform>(sheep)
        {
            transform.translation = new_position;
            if let Some(yaw) = yaw {
                transform.rotation = nalgebra_glm::quat_angle_axis(yaw, &Vec3::y());
            }
        }
    }
    for (leg, swing, offset_x, offset_z) in leg_writes {
        if let Some(transform) = scene
            .storage
            .get_mut::<nightshade_scene::contract::LocalTransform>(leg)
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

pub fn despawn(scene: &mut crate::commands::Scene<'_, '_>) {
    let sheep: Vec<Entity> = scene.storage.query_entities::<(Legs,)>().collect();
    let crate::commands::Scene { storage, stage, .. } = scene;
    let mut caches = nightshade_graphics::commands::builtins::EntityCaches {
        entities: stage.entities,
        meshes: stage.meshes,
        mesh_state: stage.mesh_state,
        textures: stage.textures,
        materials: stage.materials,
        hierarchy: stage.hierarchy,
    };
    for entity in sheep {
        nightshade_graphics::commands::builtins::despawn_recursive_now(
            storage,
            &mut caches,
            entity,
        );
    }
}

fn patrol_target(index: usize) -> Vec3 {
    let (x, z) = PATROL_POINTS[index % PATROL_POINTS.len()];
    Vec3::new(x, ENEMY_HEIGHT, z)
}

fn spawn_visual(
    scene: &mut crate::commands::Scene<'_, '_>,
    position: Vec3,
    index: u32,
) -> (Entity, [Entity; 4]) {
    let root = scene.storage.spawn((
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
        scene,
        root,
        format!("sheep_body_{}", index),
        Vec3::new(0.0, 0.62, 0.0),
        Vec3::new(0.55, 0.5, 0.85),
        wool.clone(),
    );
    spawn_sheep_part(
        scene,
        root,
        format!("sheep_head_{}", index),
        Vec3::new(0.0, 0.78, 0.55),
        Vec3::new(0.26, 0.28, 0.3),
        hide.clone(),
    );
    spawn_sheep_part(
        scene,
        root,
        format!("sheep_tuft_{}", index),
        Vec3::new(0.0, 0.96, 0.5),
        Vec3::new(0.2, 0.12, 0.18),
        wool,
    );

    let mut legs = [Entity::default(); 4];
    for (leg_index, (offset_x, offset_z)) in LEG_OFFSETS.iter().enumerate() {
        legs[leg_index] = spawn_sheep_part(
            scene,
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
    scene: &mut crate::commands::Scene<'_, '_>,
    root: Entity,
    name: String,
    translation: Vec3,
    scale: Vec3,
    material: Material,
) -> Entity {
    let entity = scene.storage.spawn((
        Name(name.clone()),
        LocalTransform {
            translation,
            scale,
            ..Default::default()
        },
        GlobalTransform::default(),
        RenderMesh::new("Cube"),
        Visibility::default(),
        CastsShadow,
        ChildOf(root),
    ));
    spawn_material(scene.storage, scene.stage.materials, entity, name, material);
    entity
}
