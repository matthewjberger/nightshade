use nalgebra_glm::Vec3;
use nightshade::ecs::{Entity, Storage};
use nightshade_render::data::geometry::{Mesh, Vertex};
use nightshade_render::data::material::Material;
use nightshade_scene::contract::LocalTransform;

use crate::commands::scene;
use crate::components::{SpinAxis, SpinRate};
use crate::data::demo::DemoControl;
use crate::resources::DemoScene;

const MESH: &str = "book/spinning_triangle/blade";
const MATERIAL: &str = "book/spinning_triangle/blade";

const SPEED: &str = "spinning_triangle/speed";
const REVERSE: &str = "spinning_triangle/reverse";
const RESET: &str = "spinning_triangle/reset";

const START_SPEED: f32 = 0.9;

pub const CONTROLS: &[DemoControl] = &[
    DemoControl::Slider {
        label: "Speed",
        action: SPEED,
        lowest: 0.0,
        highest: 4.0,
        start: START_SPEED,
    },
    DemoControl::Toggle {
        label: "Reverse",
        action: REVERSE,
        start: false,
    },
    DemoControl::Button {
        label: "Reset",
        action: RESET,
    },
];

pub fn mount(storage: &mut Storage, scene: &mut DemoScene<'_>) {
    let settings = &mut *scene.settings;
    settings.atmosphere = nightshade_render::data::config::Atmosphere::None;
    settings.bloom_enabled = false;

    scene::register_mesh(scene, MESH, blade_mesh());
    scene::register_material(
        scene,
        MATERIAL,
        Material {
            base_color: [0.35, 0.62, 0.94, 1.0],
            roughness: 0.45,
            metallic: 0.0,
            double_sided: true,
            ..Default::default()
        },
    );

    let blade = scene::spawn_owned_mesh(storage, scene, MESH, MATERIAL, LocalTransform::default());
    storage.set(blade, SpinAxis(Vec3::y()));
    storage.set(blade, SpinRate(START_SPEED));

    scene::spawn_owned_sun(storage, scene, Vec3::new(-0.4, -0.55, -0.73), 2.6);
    scene::spawn_owned_sun(storage, scene, Vec3::new(0.65, -0.15, 0.75), 0.9);
    scene::spawn_owned_camera(storage, scene, Vec3::new(0.0, 0.0, 0.0), 0.0, 0.18, 3.4);
}

pub fn step(storage: &mut Storage, scene: &mut DemoScene<'_>, delta_time: f32) {
    let input = &*scene.input;
    let reset = input.clicked.contains(&RESET);
    let speed = input.value(SPEED, START_SPEED);
    let direction = if input.switched_on(REVERSE, false) {
        -1.0
    } else {
        1.0
    };

    let spinners: Vec<Entity> = storage
        .query_entities::<(SpinRate, LocalTransform)>()
        .collect();
    for entity in spinners {
        let Some(axis) = storage.get::<SpinAxis>(entity).copied() else {
            continue;
        };
        let rate = speed * direction;
        storage.set(entity, SpinRate(rate));

        let rotation = nalgebra_glm::quat_angle_axis(rate * delta_time, &axis.0);
        if let Some(transform) = storage.get_mut::<LocalTransform>(entity) {
            transform.rotation = if reset {
                nalgebra_glm::quat_identity()
            } else {
                rotation * transform.rotation
            };
        }
    }
}

fn blade_mesh() -> Mesh {
    let radius = 1.15_f32;
    let corners = [
        Vec3::new(0.0, radius, 0.0),
        Vec3::new(-radius * 0.866, -radius * 0.5, 0.0),
        Vec3::new(radius * 0.866, -radius * 0.5, 0.0),
    ];
    let front = Vec3::z();
    let back = -Vec3::z();

    let mut vertices = Vec::with_capacity(6);
    for corner in corners {
        vertices.push(Vertex::new(corner, front));
    }
    for corner in corners {
        vertices.push(Vertex::new(corner, back));
    }

    Mesh::new(vertices, vec![0, 1, 2, 3, 5, 4])
}
