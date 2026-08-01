use nalgebra_glm::Vec3;
use nightshade::ecs::{Entity, Storage};
use nightshade_render::data::material::Material;
use nightshade_render::data::procedural_meshes::{create_plane_mesh, create_torus_mesh};
use nightshade_scene::contract::LocalTransform;

use crate::commands::scene;
use crate::data::demo::DemoControl;
use crate::resources::DemoScene;

const RING_MESH: &str = "book/plugin_stack/ring";
const RING_MATERIAL: &str = "book/plugin_stack/ring";
const FLOOR_MESH: &str = "book/plugin_stack/floor";
const FLOOR_MATERIAL: &str = "book/plugin_stack/floor";

const TURNING: &str = "plugin_stack/turning";
const BOBBING: &str = "plugin_stack/bobbing";
const TILTING: &str = "plugin_stack/tilting";

#[derive(Clone, Copy, Debug, Default)]
pub struct Composed {
    pub turned: f32,
    pub bobbed: f32,
    pub tilted: f32,
}

pub const CONTROLS: &[DemoControl] = &[
    DemoControl::Toggle {
        label: "Turning",
        action: TURNING,
        start: true,
    },
    DemoControl::Toggle {
        label: "Bobbing",
        action: BOBBING,
        start: false,
    },
    DemoControl::Toggle {
        label: "Tilting",
        action: TILTING,
        start: false,
    },
];

pub fn mount(storage: &mut Storage, scene: &mut DemoScene<'_>) {
    let settings = &mut *scene.settings;
    settings.atmosphere = nightshade_render::data::config::Atmosphere::None;
    settings.bloom_enabled = false;

    scene::register_mesh(scene, RING_MESH, create_torus_mesh(1.0, 0.33, 44, 22));
    scene::register_material(
        scene,
        RING_MATERIAL,
        Material {
            base_color: [0.52, 0.72, 0.94, 1.0],
            roughness: 0.30,
            metallic: 0.15,
            ..Default::default()
        },
    );

    scene::register_mesh(scene, FLOOR_MESH, create_plane_mesh(8.0));
    scene::register_material(
        scene,
        FLOOR_MATERIAL,
        Material {
            base_color: [0.28, 0.29, 0.32, 1.0],
            roughness: 0.92,
            ..Default::default()
        },
    );

    scene::spawn_owned_mesh(
        storage,
        scene,
        FLOOR_MESH,
        FLOOR_MATERIAL,
        LocalTransform {
            translation: Vec3::new(0.0, -1.6, 0.0),
            ..Default::default()
        },
    );

    let ring = scene::spawn_owned_mesh(
        storage,
        scene,
        RING_MESH,
        RING_MATERIAL,
        LocalTransform::default(),
    );
    storage.set(ring, Composed::default());

    scene::spawn_owned_sun(storage, scene, Vec3::new(-0.45, -0.7, -0.5), 3.0);
    scene::spawn_owned_sun(storage, scene, Vec3::new(0.6, -0.15, 0.75), 0.8);
    scene::spawn_owned_camera(storage, scene, Vec3::new(0.0, -0.2, 0.0), 0.5, 0.30, 6.0);
}

pub fn step(storage: &mut Storage, scene: &mut DemoScene<'_>, delta_time: f32) {
    let input = &*scene.input;
    let turning = input.switched_on(TURNING, true);
    let bobbing = input.switched_on(BOBBING, false);
    let tilting = input.switched_on(TILTING, false);

    let rings: Vec<Entity> = storage
        .query_entities::<(Composed, LocalTransform)>()
        .collect();

    for entity in rings {
        let Some(mut composed) = storage.get::<Composed>(entity).copied() else {
            continue;
        };
        if turning {
            composed.turned += delta_time * 0.9;
        }
        if bobbing {
            composed.bobbed += delta_time * 1.8;
        }
        if tilting {
            composed.tilted += delta_time * 1.2;
        }
        storage.set(entity, composed);

        let turn = nalgebra_glm::quat_angle_axis(composed.turned, &Vec3::y());
        let tilt =
            nalgebra_glm::quat_angle_axis(composed.tilted.sin() * 0.6, &Vec3::new(1.0, 0.0, 0.3));
        if let Some(transform) = storage.get_mut::<LocalTransform>(entity) {
            transform.rotation = turn * tilt;
            transform.translation = Vec3::new(0.0, composed.bobbed.sin() * 0.55, 0.0);
        }
    }
}
