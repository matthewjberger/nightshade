use nalgebra_glm::Vec3;
use nightshade::ecs::Storage;
use nightshade_render::commands::material_registry::material_registry_mutate;
use nightshade_render::data::material::Material;
use nightshade_render::data::procedural_meshes::create_sphere_mesh;
use nightshade_scene::contract::LocalTransform;

use crate::commands::scene;
use crate::data::demo::DemoControl;
use crate::resources::DemoScene;

const MESH: &str = "book/emissive_row/ball";

const GLOW: &str = "emissive_row/glow";
const BLOOM: &str = "emissive_row/bloom";
const EXPOSURE: &str = "emissive_row/exposure";

const START_GLOW: f32 = 1.6;
const START_EXPOSURE: f32 = 1.0;
const BALLS: usize = 5;

pub const CONTROLS: &[DemoControl] = &[
    DemoControl::Slider {
        label: "Glow",
        action: GLOW,
        lowest: 0.0,
        highest: 6.0,
        start: START_GLOW,
    },
    DemoControl::Slider {
        label: "Exposure",
        action: EXPOSURE,
        lowest: 0.3,
        highest: 2.4,
        start: START_EXPOSURE,
    },
    DemoControl::Toggle {
        label: "Bloom",
        action: BLOOM,
        start: true,
    },
];

pub fn mount(storage: &mut Storage, scene: &mut DemoScene<'_>) {
    {
        let settings = &mut *scene.settings;
        settings.atmosphere = nightshade_render::data::config::Atmosphere::None;
        settings.bloom_enabled = true;
        settings.bloom_intensity = 0.6;
        settings.color_grading.auto_exposure = false;
        settings.color_grading.exposure = START_EXPOSURE;
    }

    scene::register_mesh(scene, MESH, create_sphere_mesh(0.42, 40));

    for index in 0..BALLS {
        let share = index as f32 / (BALLS - 1) as f32;
        scene::register_material(
            scene,
            &material_name(index),
            Material {
                base_color: [0.16, 0.20, 0.26, 1.0],
                emissive_factor: glow_of(share, START_GLOW),
                roughness: 0.30,
                ..Default::default()
            },
        );
        scene::spawn_owned_mesh(
            storage,
            scene,
            MESH,
            &material_name(index),
            LocalTransform {
                translation: Vec3::new((share - 0.5) * 4.4, 0.0, 0.0),
                ..Default::default()
            },
        );
    }

    scene::spawn_owned_sun(storage, scene, Vec3::new(-0.35, -0.6, -0.72), 0.9);
    scene::spawn_owned_camera(storage, scene, Vec3::new(0.0, 0.0, 0.0), 0.0, 0.10, 6.2);
}

pub fn step(_storage: &mut Storage, scene: &mut DemoScene<'_>, _delta_time: f32) {
    let (glow, bloom, exposure) = {
        let input = &*scene.input;
        (
            input.value(GLOW, START_GLOW),
            input.switched_on(BLOOM, true),
            input.value(EXPOSURE, START_EXPOSURE),
        )
    };

    for index in 0..BALLS {
        let share = index as f32 / (BALLS - 1) as f32;
        let name = material_name(index);
        let registry = &mut *scene.materials;
        material_registry_mutate(registry, &name, |material| {
            material.emissive_factor = glow_of(share, glow);
        });
    }

    let settings = &mut *scene.settings;
    settings.bloom_enabled = bloom;
    settings.color_grading.exposure = exposure;
}

fn material_name(index: usize) -> String {
    format!("book/emissive_row/ball_{index}")
}

fn glow_of(share: f32, glow: f32) -> [f32; 3] {
    let strength = share * glow;
    [strength, strength * 0.62, strength * 0.30]
}
