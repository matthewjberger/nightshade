use nalgebra_glm::Vec3;
use nightshade::ecs::{Entity, Storage};
use nightshade_render::data::config::Atmosphere;
use nightshade_render::data::material::Material;
use nightshade_render::data::procedural_meshes::{create_plane_mesh, create_sphere_mesh};
use nightshade_scene::contract::LocalTransform;

use crate::commands::scene;
use crate::data::demo::DemoControl;
use crate::resources::DemoScene;

const BALL_MESH: &str = "book/sky_dial/ball";
const BALL_MATERIAL: &str = "book/sky_dial/ball";
const FLOOR_MESH: &str = "book/sky_dial/floor";
const FLOOR_MATERIAL: &str = "book/sky_dial/floor";

const SKY: &str = "sky_dial/sky";
const HOUR: &str = "sky_dial/hour";
const BLOOM: &str = "sky_dial/bloom";

const START_SKY: usize = 1;
const START_HOUR: f32 = 0.55;

#[derive(Clone, Copy, Debug, Default)]
pub struct Sun;

const SKIES: [Atmosphere; 6] = [
    Atmosphere::None,
    Atmosphere::Sky,
    Atmosphere::CloudySky,
    Atmosphere::Sunset,
    Atmosphere::Space,
    Atmosphere::Nebula,
];

const SKY_NAMES: [&str; 6] = ["None", "Sky", "Cloudy sky", "Sunset", "Space", "Nebula"];

pub const CONTROLS: &[DemoControl] = &[
    DemoControl::Choice {
        label: "Sky",
        action: SKY,
        options: &SKY_NAMES,
        start: START_SKY,
    },
    DemoControl::Slider {
        label: "Hour",
        action: HOUR,
        lowest: 0.05,
        highest: 0.95,
        start: START_HOUR,
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
        settings.atmosphere = SKIES[START_SKY];
        settings.bloom_enabled = true;
        settings.bloom_intensity = 0.45;
    }

    scene::register_mesh(scene, BALL_MESH, create_sphere_mesh(0.9, 44));
    scene::register_material(
        scene,
        BALL_MATERIAL,
        Material {
            base_color: [0.82, 0.82, 0.84, 1.0],
            roughness: 0.22,
            metallic: 0.65,
            ..Default::default()
        },
    );

    scene::register_mesh(scene, FLOOR_MESH, create_plane_mesh(14.0));
    scene::register_material(
        scene,
        FLOOR_MATERIAL,
        Material {
            base_color: [0.34, 0.33, 0.31, 1.0],
            roughness: 0.88,
            ..Default::default()
        },
    );

    scene::spawn_owned_mesh(
        storage,
        scene,
        FLOOR_MESH,
        FLOOR_MATERIAL,
        LocalTransform {
            translation: Vec3::new(0.0, -0.9, 0.0),
            ..Default::default()
        },
    );
    scene::spawn_owned_mesh(
        storage,
        scene,
        BALL_MESH,
        BALL_MATERIAL,
        LocalTransform::default(),
    );

    let sun = scene::spawn_owned_sun(storage, scene, sun_direction(START_HOUR), 3.2);
    storage.set(sun, Sun);

    scene::spawn_owned_camera(storage, scene, Vec3::new(0.0, -0.1, 0.0), 0.7, 0.16, 5.6);
}

pub fn step(storage: &mut Storage, scene: &mut DemoScene<'_>, _delta_time: f32) {
    let (sky, hour, bloom) = {
        let input = &*scene.input;
        (
            (input.value(SKY, START_SKY as f32).round().max(0.0) as usize).min(SKIES.len() - 1),
            input.value(HOUR, START_HOUR),
            input.switched_on(BLOOM, true),
        )
    };

    {
        let settings = &mut *scene.settings;
        settings.atmosphere = SKIES[sky];
        settings.bloom_enabled = bloom;
    }

    let direction = sun_direction(hour);
    let suns: Vec<Entity> = storage.query_entities::<(Sun, LocalTransform)>().collect();
    for entity in suns {
        if let Some(transform) = storage.get_mut::<LocalTransform>(entity) {
            transform.translation = -direction * 12.0;
            transform.rotation = scene::orbit_rotation(direction);
        }
    }
}

fn sun_direction(hour: f32) -> Vec3 {
    let angle = hour * std::f32::consts::PI;
    Vec3::new(-angle.cos(), -angle.sin().max(0.08), -0.35).normalize()
}
