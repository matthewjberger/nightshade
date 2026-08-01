use nalgebra_glm::Vec3;
use nightshade::ecs::{Entity, Storage};
use nightshade_render::components::{Light, LightType};
use nightshade_render::data::material::Material;
use nightshade_render::data::procedural_meshes::{create_plane_mesh, create_sphere_mesh};
use nightshade_scene::contract::LocalTransform;

use crate::commands::scene;
use crate::components::{OrbitHeight, OrbitPhase, OrbitRadius, OrbitRate};
use crate::data::demo::DemoControl;
use crate::resources::DemoScene;

const BALL_MESH: &str = "book/light_rig/ball";
const BALL_MATERIAL: &str = "book/light_rig/ball";
const FLOOR_MESH: &str = "book/light_rig/floor";
const FLOOR_MATERIAL: &str = "book/light_rig/floor";
const LAMP_MESH: &str = "book/light_rig/lamp";
const LAMP_MATERIAL: &str = "book/light_rig/lamp";

const HEIGHT: &str = "light_rig/height";
const STRENGTH: &str = "light_rig/strength";
const MOVING: &str = "light_rig/moving";

const START_HEIGHT: f32 = 1.6;
const START_STRENGTH: f32 = 14.0;
const LAMP_RADIUS: f32 = 2.2;

pub const CONTROLS: &[DemoControl] = &[
    DemoControl::Slider {
        label: "Height",
        action: HEIGHT,
        lowest: 0.3,
        highest: 3.5,
        start: START_HEIGHT,
    },
    DemoControl::Slider {
        label: "Strength",
        action: STRENGTH,
        lowest: 1.0,
        highest: 40.0,
        start: START_STRENGTH,
    },
    DemoControl::Toggle {
        label: "Moving",
        action: MOVING,
        start: true,
    },
];

pub fn mount(storage: &mut Storage, scene: &mut DemoScene<'_>) {
    let settings = &mut *scene.settings;
    settings.atmosphere = nightshade_render::data::config::Atmosphere::None;
    settings.bloom_enabled = false;

    scene::register_mesh(scene, BALL_MESH, create_sphere_mesh(0.8, 40));
    scene::register_material(
        scene,
        BALL_MATERIAL,
        Material {
            base_color: [0.82, 0.80, 0.76, 1.0],
            roughness: 0.42,
            metallic: 0.0,
            ..Default::default()
        },
    );

    scene::register_mesh(scene, FLOOR_MESH, create_plane_mesh(9.0));
    scene::register_material(
        scene,
        FLOOR_MATERIAL,
        Material {
            base_color: [0.42, 0.43, 0.46, 1.0],
            roughness: 0.85,
            metallic: 0.0,
            ..Default::default()
        },
    );

    scene::register_mesh(scene, LAMP_MESH, create_sphere_mesh(0.09, 16));
    scene::register_material(
        scene,
        LAMP_MATERIAL,
        Material {
            base_color: [1.0, 0.95, 0.85, 1.0],
            emissive_factor: [3.0, 2.8, 2.4],
            roughness: 1.0,
            metallic: 0.0,
            ..Default::default()
        },
    );

    scene::spawn_owned_mesh(
        storage,
        scene,
        BALL_MESH,
        BALL_MATERIAL,
        LocalTransform {
            translation: Vec3::new(0.0, 0.8, 0.0),
            ..Default::default()
        },
    );
    scene::spawn_owned_mesh(
        storage,
        scene,
        FLOOR_MESH,
        FLOOR_MATERIAL,
        LocalTransform::default(),
    );

    let lamp = scene::spawn_owned_mesh(
        storage,
        scene,
        LAMP_MESH,
        LAMP_MATERIAL,
        LocalTransform {
            translation: Vec3::new(LAMP_RADIUS, START_HEIGHT, 0.0),
            ..Default::default()
        },
    );
    storage.set(lamp, OrbitRadius(LAMP_RADIUS));
    storage.set(lamp, OrbitHeight(START_HEIGHT));
    storage.set(lamp, OrbitRate(0.55));
    storage.set(lamp, OrbitPhase(0.0));
    storage.set(
        lamp,
        Light {
            light_type: LightType::Point,
            color: Vec3::new(1.0, 0.94, 0.86),
            intensity: START_STRENGTH,
            range: 24.0,
            cast_shadows: true,
            ..Default::default()
        },
    );

    scene::spawn_owned_sun(storage, scene, Vec3::new(-0.3, -0.9, -0.3), 0.55);
    scene::spawn_owned_camera(storage, scene, Vec3::new(0.0, 0.7, 0.0), 0.6, 0.30, 6.4);
}

pub fn step(storage: &mut Storage, scene: &mut DemoScene<'_>, delta_time: f32) {
    let input = &*scene.input;
    let height = input.value(HEIGHT, START_HEIGHT);
    let strength = input.value(STRENGTH, START_STRENGTH);
    let moving = input.switched_on(MOVING, true);

    let lamps: Vec<Entity> = storage.query_entities::<(OrbitPhase, Light)>().collect();
    for entity in lamps {
        let Some(orbit_phase) = storage.get::<OrbitPhase>(entity).copied() else {
            continue;
        };
        let rate = storage
            .get::<OrbitRate>(entity)
            .copied()
            .unwrap_or_default();
        let radius = storage
            .get::<OrbitRadius>(entity)
            .copied()
            .unwrap_or_default();
        let phase = if moving {
            orbit_phase.0 + rate.0 * delta_time
        } else {
            orbit_phase.0
        };

        storage.set(entity, OrbitPhase(phase));
        storage.set(entity, OrbitHeight(height));
        if let Some(light) = storage.get_mut::<Light>(entity) {
            light.intensity = strength;
        }
        if let Some(transform) = storage.get_mut::<LocalTransform>(entity) {
            transform.translation =
                Vec3::new(phase.cos() * radius.0, height, phase.sin() * radius.0);
        }
    }
}
