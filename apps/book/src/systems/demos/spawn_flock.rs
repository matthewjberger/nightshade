use nalgebra_glm::Vec3;
use nightshade::ecs::{Entity, Storage};
use nightshade_render::data::material::Material;
use nightshade_render::data::procedural_meshes::create_cube_mesh;
use nightshade_scene::contract::LocalTransform;

use crate::commands::scene;
use crate::data::demo::DemoControl;
use crate::resources::DemoScene;

const MESH: &str = "book/spawn_flock/mote";
const MATERIAL: &str = "book/spawn_flock/mote";

const COUNT: &str = "spawn_flock/count";
const SPEED: &str = "spawn_flock/speed";
const SETTLE: &str = "spawn_flock/settle";

const START_COUNT: f32 = 24.0;
const START_SPEED: f32 = 1.0;
const MOST: usize = 120;

#[derive(Clone, Copy, Debug, Default)]
pub struct Mote {
    pub around: f32,
    pub up: f32,
    pub phase: f32,
}

pub const CONTROLS: &[DemoControl] = &[
    DemoControl::Slider {
        label: "Count",
        action: COUNT,
        lowest: 0.0,
        highest: MOST as f32,
        start: START_COUNT,
    },
    DemoControl::Slider {
        label: "Speed",
        action: SPEED,
        lowest: 0.0,
        highest: 3.0,
        start: START_SPEED,
    },
    DemoControl::Toggle {
        label: "Settle",
        action: SETTLE,
        start: false,
    },
];

pub fn mount(storage: &mut Storage, scene: &mut DemoScene<'_>) {
    let settings = &mut *scene.settings;
    settings.atmosphere = nightshade_render::data::config::Atmosphere::None;
    settings.bloom_enabled = true;
    settings.bloom_intensity = 0.35;

    scene::register_mesh(scene, MESH, create_cube_mesh());
    scene::register_material(
        scene,
        MATERIAL,
        Material {
            base_color: [0.40, 0.70, 0.96, 1.0],
            emissive_factor: [0.10, 0.24, 0.42],
            roughness: 0.35,
            ..Default::default()
        },
    );

    for index in 0..START_COUNT as usize {
        add_mote(storage, scene, index);
    }

    scene::spawn_owned_sun(storage, scene, Vec3::new(-0.4, -0.6, -0.7), 2.6);
    scene::spawn_owned_sun(storage, scene, Vec3::new(0.7, -0.1, 0.6), 0.8);
    scene::spawn_owned_camera(storage, scene, Vec3::new(0.0, 0.0, 0.0), 0.4, 0.28, 7.0);
}

pub fn step(storage: &mut Storage, scene: &mut DemoScene<'_>, delta_time: f32) {
    let input = &*scene.input;
    let wanted = input
        .value(COUNT, START_COUNT)
        .round()
        .clamp(0.0, MOST as f32) as usize;
    let speed = input.value(SPEED, START_SPEED);
    let settle = input.switched_on(SETTLE, false);

    let mut motes: Vec<Entity> = storage.query_entities::<(Mote, LocalTransform)>().collect();

    while motes.len() > wanted {
        if let Some(entity) = motes.pop() {
            scene::despawn_owned(storage, scene, &[entity]);
        }
    }
    while motes.len() < wanted {
        let index = motes.len();
        motes.push(add_mote(storage, scene, index));
    }

    for entity in motes {
        let Some(mote) = storage.get::<Mote>(entity).copied() else {
            continue;
        };
        let phase = mote.phase + speed * delta_time;
        if let Some(next) = storage.get_mut::<Mote>(entity) {
            next.phase = phase;
        }

        let radius = if settle {
            2.6
        } else {
            2.6 + (phase * 0.9).sin() * 0.5
        };
        let around = mote.around + if settle { 0.0 } else { phase * 0.12 };
        let up = mote.up;
        let position = Vec3::new(
            around.cos() * up.cos() * radius,
            up.sin() * radius,
            around.sin() * up.cos() * radius,
        );

        if let Some(transform) = storage.get_mut::<LocalTransform>(entity) {
            transform.translation = position;
            transform.rotation = nalgebra_glm::quat_angle_axis(phase, &Vec3::y());
        }
    }
}

fn add_mote(storage: &mut Storage, scene: &mut DemoScene<'_>, index: usize) -> Entity {
    let golden = std::f32::consts::PI * (3.0 - 5.0_f32.sqrt());
    let around = index as f32 * golden;
    let up = ((index as f32 % 17.0) / 17.0 - 0.5) * std::f32::consts::PI * 0.9;

    let mote = scene::spawn_owned_mesh(
        storage,
        scene,
        MESH,
        MATERIAL,
        LocalTransform {
            scale: Vec3::new(0.13, 0.13, 0.13),
            ..Default::default()
        },
    );
    storage.set(
        mote,
        Mote {
            around,
            up,
            phase: index as f32 * 0.35,
        },
    );
    mote
}
