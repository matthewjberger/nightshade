use nalgebra_glm::Vec3;
use nightshade::ecs::{Entity, Storage};
use nightshade_render::data::material::Material;
use nightshade_render::data::procedural_meshes::{create_cube_mesh, create_plane_mesh};
use nightshade_scene::contract::LocalTransform;

use crate::commands::scene;
use crate::data::demo::DemoControl;
use crate::resources::DemoScene;

const CUBE_MESH: &str = "book/transform_parts/cube";
const CUBE_MATERIAL: &str = "book/transform_parts/cube";
const FLOOR_MESH: &str = "book/transform_parts/floor";
const FLOOR_MATERIAL: &str = "book/transform_parts/floor";

const ACROSS: &str = "transform_parts/across";
const TURN: &str = "transform_parts/turn";
const SIZE: &str = "transform_parts/size";

const START_ACROSS: f32 = 0.0;
const START_TURN: f32 = 0.0;
const START_SIZE: f32 = 1.0;

#[derive(Clone, Copy, Debug, Default)]
pub struct Handled;

pub const CONTROLS: &[DemoControl] = &[
    DemoControl::Slider {
        label: "Across",
        action: ACROSS,
        lowest: -2.0,
        highest: 2.0,
        start: START_ACROSS,
    },
    DemoControl::Slider {
        label: "Turn",
        action: TURN,
        lowest: 0.0,
        highest: 360.0,
        start: START_TURN,
    },
    DemoControl::Slider {
        label: "Size",
        action: SIZE,
        lowest: 0.25,
        highest: 2.0,
        start: START_SIZE,
    },
];

pub fn mount(storage: &mut Storage, scene: &mut DemoScene<'_>) {
    let settings = &mut *scene.settings;
    settings.atmosphere = nightshade_render::data::config::Atmosphere::None;
    settings.bloom_enabled = false;

    scene::register_mesh(scene, CUBE_MESH, create_cube_mesh());
    scene::register_material(
        scene,
        CUBE_MATERIAL,
        Material {
            base_color: [0.45, 0.66, 0.92, 1.0],
            roughness: 0.42,
            metallic: 0.0,
            ..Default::default()
        },
    );

    scene::register_mesh(scene, FLOOR_MESH, create_plane_mesh(7.0));
    scene::register_material(
        scene,
        FLOOR_MATERIAL,
        Material {
            base_color: [0.30, 0.31, 0.34, 1.0],
            roughness: 0.9,
            metallic: 0.0,
            ..Default::default()
        },
    );

    scene::spawn_owned_mesh(
        storage,
        scene,
        FLOOR_MESH,
        FLOOR_MATERIAL,
        LocalTransform {
            translation: Vec3::new(0.0, -0.8, 0.0),
            ..Default::default()
        },
    );

    let cube = scene::spawn_owned_mesh(
        storage,
        scene,
        CUBE_MESH,
        CUBE_MATERIAL,
        LocalTransform::default(),
    );
    storage.set(cube, Handled);

    scene::spawn_owned_sun(storage, scene, Vec3::new(-0.45, -0.7, -0.55), 3.0);
    scene::spawn_owned_sun(storage, scene, Vec3::new(0.6, -0.2, 0.7), 0.7);
    scene::spawn_owned_camera(storage, scene, Vec3::new(0.0, -0.1, 0.0), 0.5, 0.35, 5.4);
}

pub fn step(storage: &mut Storage, scene: &mut DemoScene<'_>, _delta_time: f32) {
    let input = &*scene.input;
    let across = input.value(ACROSS, START_ACROSS);
    let turn = input.value(TURN, START_TURN).to_radians();
    let size = input.value(SIZE, START_SIZE);

    let handled: Vec<Entity> = storage
        .query_entities::<(Handled, LocalTransform)>()
        .collect();
    for entity in handled {
        let Some(transform) = storage.get_mut::<LocalTransform>(entity) else {
            continue;
        };
        transform.translation = Vec3::new(across, 0.0, 0.0);
        transform.rotation = nalgebra_glm::quat_angle_axis(turn, &Vec3::y());
        transform.scale = Vec3::new(size, size, size);
    }
}
