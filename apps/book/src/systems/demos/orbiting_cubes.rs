use nalgebra_glm::Vec3;
use nightshade::ecs::{Entity, Storage};
use nightshade_render::data::material::Material;
use nightshade_render::data::procedural_meshes::create_cube_mesh;
use nightshade_scene::contract::LocalTransform;

use crate::commands::scene;
use crate::components::{OrbitHeight, OrbitPhase, OrbitRadius, OrbitRate, SpinAxis, SpinRate};
use crate::data::demo::DemoControl;
use crate::resources::DemoScene;

const MESH: &str = "book/orbiting_cubes/cube";
const MATERIAL: &str = "book/orbiting_cubes/cube";

const ADD: &str = "orbiting_cubes/add";
const REMOVE: &str = "orbiting_cubes/remove";
const SPEED: &str = "orbiting_cubes/speed";

const STARTING_COUNT: usize = 5;
const MOST_CUBES: usize = 10;
const RADIUS: f32 = 1.7;
const ORBIT_SPEED: f32 = 0.7;
const SPIN_SPEED: f32 = 1.6;
const CUBE_SCALE: f32 = 0.55;

pub const CONTROLS: &[DemoControl] = &[
    DemoControl::Button {
        label: "Add cube",
        action: ADD,
    },
    DemoControl::Button {
        label: "Remove",
        action: REMOVE,
    },
    DemoControl::Slider {
        label: "Speed",
        action: SPEED,
        lowest: 0.0,
        highest: 2.0,
        start: ORBIT_SPEED,
    },
];

pub fn mount(storage: &mut Storage, scene: &mut DemoScene<'_>) {
    let settings = &mut *scene.settings;
    settings.atmosphere = nightshade_render::data::config::Atmosphere::None;
    settings.bloom_enabled = false;

    scene::register_mesh(scene, MESH, create_cube_mesh());
    scene::register_material(
        scene,
        MATERIAL,
        Material {
            base_color: [0.86, 0.52, 0.30, 1.0],
            roughness: 0.35,
            metallic: 0.1,
            ..Default::default()
        },
    );

    let mut cubes = Vec::with_capacity(STARTING_COUNT);
    for _ in 0..STARTING_COUNT {
        cubes.push(add_cube(storage, scene));
    }
    space_evenly(storage, &cubes);

    scene::spawn_owned_sun(storage, scene, Vec3::new(-0.45, -0.72, -0.53), 2.6);
    scene::spawn_owned_sun(storage, scene, Vec3::new(0.6, -0.2, 0.77), 0.8);
    scene::spawn_owned_sun(storage, scene, Vec3::new(0.2, 0.75, -0.63), 0.5);
    scene::spawn_owned_camera(storage, scene, Vec3::new(0.0, 0.0, 0.0), 0.4, 0.42, 5.2);
}

pub fn step(storage: &mut Storage, scene: &mut DemoScene<'_>, delta_time: f32) {
    let input = &*scene.input;
    let clicked = input.clicked.clone();
    let speed = input.value(SPEED, ORBIT_SPEED);

    let mut cubes: Vec<Entity> = storage
        .query_entities::<(OrbitPhase, SpinRate, LocalTransform)>()
        .collect();

    let mut count_changed = false;
    for action in &clicked {
        match *action {
            ADD if cubes.len() < MOST_CUBES => {
                cubes.push(add_cube(storage, scene));
                count_changed = true;
            }
            REMOVE => {
                if let Some(entity) = cubes.pop() {
                    scene::despawn_owned(storage, scene, &[entity]);
                    count_changed = true;
                }
            }
            _ => {}
        }
    }
    if count_changed {
        space_evenly(storage, &cubes);
    }

    for entity in cubes {
        let Some(orbit_phase) = storage.get::<OrbitPhase>(entity).copied() else {
            continue;
        };
        let Some(radius) = storage.get::<OrbitRadius>(entity).copied() else {
            continue;
        };
        let Some(height) = storage.get::<OrbitHeight>(entity).copied() else {
            continue;
        };
        let Some(axis) = storage.get::<SpinAxis>(entity).copied() else {
            continue;
        };
        let Some(rate) = storage.get::<SpinRate>(entity).copied() else {
            continue;
        };
        let phase = orbit_phase.0 + speed * delta_time;
        let rotation = nalgebra_glm::quat_angle_axis(rate.0 * delta_time, &axis.0);

        storage.set(entity, OrbitPhase(phase));
        storage.set(entity, OrbitRate(speed));
        if let Some(transform) = storage.get_mut::<LocalTransform>(entity) {
            transform.translation =
                Vec3::new(phase.cos() * radius.0, height.0, phase.sin() * radius.0);
            transform.rotation = rotation * transform.rotation;
        }
    }
}

fn add_cube(storage: &mut Storage, scene: &mut DemoScene<'_>) -> Entity {
    let cube = scene::spawn_owned_mesh(
        storage,
        scene,
        MESH,
        MATERIAL,
        LocalTransform {
            scale: Vec3::new(CUBE_SCALE, CUBE_SCALE, CUBE_SCALE),
            ..Default::default()
        },
    );
    storage.set(cube, OrbitRadius(RADIUS));
    storage.set(cube, OrbitHeight(0.0));
    storage.set(cube, OrbitRate(ORBIT_SPEED));
    storage.set(cube, OrbitPhase(0.0));
    storage.set(cube, SpinAxis(Vec3::new(0.3, 1.0, 0.2).normalize()));
    storage.set(cube, SpinRate(SPIN_SPEED));
    cube
}

fn space_evenly(storage: &mut Storage, cubes: &[Entity]) {
    let Some(first) = cubes.first() else {
        return;
    };
    let base = storage
        .get::<OrbitPhase>(*first)
        .map_or(0.0, |phase| phase.0);
    let step = std::f32::consts::TAU / cubes.len() as f32;

    for (index, entity) in cubes.iter().enumerate() {
        storage.set(*entity, OrbitPhase(base + index as f32 * step));
    }
}
