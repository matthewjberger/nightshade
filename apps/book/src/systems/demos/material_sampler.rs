use nalgebra_glm::Vec3;
use nightshade::ecs::{Entity, Storage};
use nightshade_render::commands::material_registry::material_registry_mutate;
use nightshade_render::data::material::Material;
use nightshade_render::data::procedural_meshes::create_sphere_mesh;
use nightshade_scene::contract::LocalTransform;

use crate::commands::scene;
use crate::components::{SpinAxis, SpinRate};
use crate::data::demo::DemoControl;
use crate::resources::DemoScene;

const MESH: &str = "book/material_sampler/ball";
const MATERIALS: [&str; 5] = [
    "book/material_sampler/ball_0",
    "book/material_sampler/ball_1",
    "book/material_sampler/ball_2",
    "book/material_sampler/ball_3",
    "book/material_sampler/ball_4",
];

const ROUGHNESS: &str = "material_sampler/roughness";
const METAL: &str = "material_sampler/metal";
const TURN: &str = "material_sampler/turn";

const START_ROUGHNESS: f32 = 0.35;
const START_METAL: f32 = 0.0;

pub const CONTROLS: &[DemoControl] = &[
    DemoControl::Slider {
        label: "Rough",
        action: ROUGHNESS,
        lowest: 0.02,
        highest: 1.0,
        start: START_ROUGHNESS,
    },
    DemoControl::Slider {
        label: "Metal",
        action: METAL,
        lowest: 0.0,
        highest: 1.0,
        start: START_METAL,
    },
    DemoControl::Toggle {
        label: "Turn",
        action: TURN,
        start: true,
    },
];

pub fn mount(storage: &mut Storage, scene: &mut DemoScene<'_>) {
    let settings = &mut *scene.settings;
    settings.atmosphere = nightshade_render::data::config::Atmosphere::Sky;
    settings.bloom_enabled = false;

    scene::register_mesh(scene, MESH, create_sphere_mesh(0.52, 44));

    for (index, name) in MATERIALS.iter().enumerate() {
        let shade = index as f32 / (MATERIALS.len() - 1) as f32;
        scene::register_material(
            scene,
            name,
            Material {
                base_color: [
                    0.72 - shade * 0.28,
                    0.55 + shade * 0.18,
                    0.36 + shade * 0.34,
                    1.0,
                ],
                roughness: START_ROUGHNESS,
                metallic: START_METAL,
                ..Default::default()
            },
        );
    }

    let spread = 1.25;
    let first = -spread * (MATERIALS.len() as f32 - 1.0) * 0.5;
    for (index, name) in MATERIALS.iter().enumerate() {
        let ball = scene::spawn_owned_mesh(
            storage,
            scene,
            MESH,
            name,
            LocalTransform {
                translation: Vec3::new(first + index as f32 * spread, 0.0, 0.0),
                ..Default::default()
            },
        );
        storage.set(ball, SpinAxis(Vec3::y()));
        storage.set(ball, SpinRate(0.35));
    }

    scene::spawn_owned_sun(storage, scene, Vec3::new(-0.4, -0.7, -0.58), 3.2);
    scene::spawn_owned_sun(storage, scene, Vec3::new(0.62, -0.25, 0.74), 1.0);
    scene::spawn_owned_camera(storage, scene, Vec3::new(0.0, 0.0, 0.0), 0.0, 0.16, 5.6);
}

pub fn step(storage: &mut Storage, scene: &mut DemoScene<'_>, delta_time: f32) {
    let input = &*scene.input;
    let roughness = input.value(ROUGHNESS, START_ROUGHNESS);
    let metal = input.value(METAL, START_METAL);
    let turning = input.switched_on(TURN, true);

    let registry = &mut *scene.materials;
    for name in MATERIALS {
        material_registry_mutate(registry, name, |material| {
            material.roughness = roughness;
            material.metallic = metal;
        });
    }

    if !turning {
        return;
    }

    let balls: Vec<Entity> = storage
        .query_entities::<(SpinRate, LocalTransform)>()
        .collect();
    for entity in balls {
        let Some(axis) = storage.get::<SpinAxis>(entity).copied() else {
            continue;
        };
        let Some(rate) = storage.get::<SpinRate>(entity).copied() else {
            continue;
        };
        let rotation = nalgebra_glm::quat_angle_axis(rate.0 * delta_time, &axis.0);
        if let Some(transform) = storage.get_mut::<LocalTransform>(entity) {
            transform.rotation = rotation * transform.rotation;
        }
    }
}
