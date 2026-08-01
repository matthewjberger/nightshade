use nalgebra_glm::Vec3;
use nightshade::ecs::{ChildOf, Entity, Storage};
use nightshade_render::data::material::Material;
use nightshade_render::data::procedural_meshes::{create_cube_mesh, create_sphere_mesh};
use nightshade_scene::contract::LocalTransform;

use crate::commands::scene;
use crate::data::demo::DemoControl;
use crate::resources::DemoScene;

const TRUNK_MESH: &str = "book/hierarchy/trunk";
const TRUNK_MATERIAL: &str = "book/hierarchy/trunk";
const LIMB_MESH: &str = "book/hierarchy/limb";
const LIMB_MATERIAL: &str = "book/hierarchy/limb";
const LOOSE_MATERIAL: &str = "book/hierarchy/loose";

const TURN: &str = "hierarchy/turn";
const LIFT: &str = "hierarchy/lift";
const CUT: &str = "hierarchy/cut";

const START_TURN: f32 = 0.6;
const START_LIFT: f32 = 0.0;
const LIMBS: usize = 4;

#[derive(Clone, Copy, Debug, Default)]
pub struct Trunk;

#[derive(Clone, Copy, Debug, Default)]
pub struct Limb {
    pub at: Vec3,
    pub loose: bool,
}

pub const CONTROLS: &[DemoControl] = &[
    DemoControl::Slider {
        label: "Turn",
        action: TURN,
        lowest: 0.0,
        highest: 2.0,
        start: START_TURN,
    },
    DemoControl::Slider {
        label: "Lift",
        action: LIFT,
        lowest: -0.8,
        highest: 0.8,
        start: START_LIFT,
    },
    DemoControl::Toggle {
        label: "Cut one loose",
        action: CUT,
        start: false,
    },
];

pub fn mount(storage: &mut Storage, scene: &mut DemoScene<'_>) {
    let settings = &mut *scene.settings;
    settings.atmosphere = nightshade_render::data::config::Atmosphere::None;
    settings.bloom_enabled = false;

    scene::register_mesh(scene, TRUNK_MESH, create_cube_mesh());
    scene::register_material(
        scene,
        TRUNK_MATERIAL,
        Material {
            base_color: [0.42, 0.46, 0.55, 1.0],
            roughness: 0.5,
            ..Default::default()
        },
    );

    scene::register_mesh(scene, LIMB_MESH, create_sphere_mesh(0.28, 28));
    scene::register_material(
        scene,
        LIMB_MATERIAL,
        Material {
            base_color: [0.36, 0.68, 0.95, 1.0],
            roughness: 0.35,
            ..Default::default()
        },
    );
    scene::register_material(
        scene,
        LOOSE_MATERIAL,
        Material {
            base_color: [0.94, 0.52, 0.36, 1.0],
            roughness: 0.35,
            ..Default::default()
        },
    );

    let trunk = scene::spawn_owned_mesh(
        storage,
        scene,
        TRUNK_MESH,
        TRUNK_MATERIAL,
        LocalTransform {
            scale: Vec3::new(0.7, 0.7, 0.7),
            ..Default::default()
        },
    );
    storage.set(trunk, Trunk);

    for index in 0..LIMBS {
        let angle = index as f32 * std::f32::consts::TAU / LIMBS as f32;
        let at = Vec3::new(angle.cos() * 1.6, 0.0, angle.sin() * 1.6);
        let material = if index == 0 {
            LOOSE_MATERIAL
        } else {
            LIMB_MATERIAL
        };
        let limb = scene::spawn_owned_mesh(
            storage,
            scene,
            LIMB_MESH,
            material,
            LocalTransform {
                translation: at,
                ..Default::default()
            },
        );
        storage.set(
            limb,
            Limb {
                at,
                loose: index == 0,
            },
        );
        storage.set(limb, ChildOf(trunk));
    }

    scene::spawn_owned_sun(storage, scene, Vec3::new(-0.4, -0.65, -0.65), 3.0);
    scene::spawn_owned_sun(storage, scene, Vec3::new(0.7, -0.2, 0.6), 0.8);
    scene::spawn_owned_camera(storage, scene, Vec3::new(0.0, 0.0, 0.0), 0.6, 0.42, 6.4);
}

pub fn step(storage: &mut Storage, scene: &mut DemoScene<'_>, delta_time: f32) {
    let input = &*scene.input;
    let turn = input.value(TURN, START_TURN);
    let lift = input.value(LIFT, START_LIFT);
    let cut = input.switched_on(CUT, false);

    let trunks: Vec<Entity> = storage
        .query_entities::<(Trunk, LocalTransform)>()
        .collect();
    for entity in trunks {
        let Some(transform) = storage.get_mut::<LocalTransform>(entity) else {
            continue;
        };
        let step = nalgebra_glm::quat_angle_axis(turn * delta_time, &Vec3::y());
        transform.rotation = step * transform.rotation;
        transform.translation = Vec3::new(0.0, lift, 0.0);
    }

    let limbs: Vec<Entity> = storage.query_entities::<(Limb,)>().collect();
    for entity in limbs {
        let Some(limb) = storage.get::<Limb>(entity).copied() else {
            continue;
        };
        if !limb.loose {
            continue;
        }
        let attached = storage.get::<ChildOf>(entity).is_some();
        if cut && attached {
            storage.remove_components::<(ChildOf,)>(entity);
            if let Some(transform) = storage.get_mut::<LocalTransform>(entity) {
                transform.translation = limb.at;
            }
        }
    }
}
