use nalgebra_glm::{Vec2, Vec3};
use nightshade::ecs::{Entity, Storage};
use nightshade_render::data::material::Material;
use nightshade_render::data::procedural_meshes::{
    create_cone_mesh, create_cube_mesh, create_cylinder_mesh, create_plane_mesh,
    create_sphere_mesh, create_torus_mesh,
};
use nightshade_scene::contract::{GlobalTransform, LocalTransform};

use crate::commands::scene;
use crate::components::{PickRadius, Picked, Slot};
use crate::data::demo::DemoControl;
use crate::resources::DemoScene;

const MATERIAL_PLAIN: &str = "book/picking_table/plain";
const MATERIAL_PICKED: &str = "book/picking_table/picked";
const MATERIAL_FLOOR: &str = "book/picking_table/floor";
const FLOOR_MESH: &str = "book/picking_table/floor";

const SHAPES: [(&str, f32); 5] = [
    ("book/picking_table/cube", 0.62),
    ("book/picking_table/ball", 0.55),
    ("book/picking_table/cone", 0.70),
    ("book/picking_table/pipe", 0.62),
    ("book/picking_table/ring", 0.70),
];

const CLEAR: &str = "picking_table/clear";
const SPREAD: &str = "picking_table/spread";

const START_SPREAD: f32 = 1.35;
const RISE: f32 = 0.35;

pub const CONTROLS: &[DemoControl] = &[
    DemoControl::Slider {
        label: "Spread",
        action: SPREAD,
        lowest: 0.9,
        highest: 2.0,
        start: START_SPREAD,
    },
    DemoControl::Button {
        label: "Clear",
        action: CLEAR,
    },
];

pub fn mount(storage: &mut Storage, scene: &mut DemoScene<'_>) {
    let settings = &mut *scene.settings;
    settings.atmosphere = nightshade_render::data::config::Atmosphere::None;
    settings.bloom_enabled = false;

    scene::register_material(
        scene,
        MATERIAL_PLAIN,
        Material {
            base_color: [0.60, 0.63, 0.70, 1.0],
            roughness: 0.5,
            metallic: 0.05,
            ..Default::default()
        },
    );
    scene::register_material(
        scene,
        MATERIAL_PICKED,
        Material {
            base_color: [0.94, 0.62, 0.24, 1.0],
            roughness: 0.35,
            metallic: 0.1,
            ..Default::default()
        },
    );

    scene::register_material(
        scene,
        MATERIAL_FLOOR,
        Material {
            base_color: [0.52, 0.53, 0.56, 1.0],
            roughness: 0.9,
            metallic: 0.0,
            ..Default::default()
        },
    );
    scene::register_mesh(scene, FLOOR_MESH, create_plane_mesh(14.0));
    scene::spawn_owned_mesh(
        storage,
        scene,
        FLOOR_MESH,
        MATERIAL_FLOOR,
        LocalTransform {
            translation: Vec3::new(0.0, -0.55, 0.0),
            ..Default::default()
        },
    );

    scene::register_mesh(scene, SHAPES[0].0, create_cube_mesh());
    scene::register_mesh(scene, SHAPES[1].0, create_sphere_mesh(0.5, 36));
    scene::register_mesh(scene, SHAPES[2].0, create_cone_mesh(0.5, 1.0, 28));
    scene::register_mesh(scene, SHAPES[3].0, create_cylinder_mesh(0.4, 1.0, 28));
    scene::register_mesh(scene, SHAPES[4].0, create_torus_mesh(0.45, 0.18, 28, 18));

    let first = -START_SPREAD * (SHAPES.len() as f32 - 1.0) * 0.5;
    for (index, (mesh, radius)) in SHAPES.iter().enumerate() {
        let shape = scene::spawn_owned_mesh(
            storage,
            scene,
            mesh,
            MATERIAL_PLAIN,
            LocalTransform {
                translation: Vec3::new(first + index as f32 * START_SPREAD, 0.0, 0.0),
                ..Default::default()
            },
        );
        storage.set(shape, PickRadius(*radius));
        storage.set(shape, Slot(index));
        storage.set(shape, Picked(false));
    }

    let key = scene::spawn_owned_sun(storage, scene, Vec3::new(-0.42, -0.70, -0.58), 3.0);
    if let Some(light) = storage.get_mut::<nightshade_render::components::Light>(key) {
        light.cast_shadows = true;
    }
    scene::spawn_owned_sun(storage, scene, Vec3::new(0.6, -0.22, 0.76), 0.9);
    scene::spawn_owned_sun(storage, scene, Vec3::new(0.15, 0.8, -0.58), 0.45);
    scene::spawn_owned_camera(storage, scene, Vec3::new(0.0, 0.0, 0.0), 0.0, 0.30, 6.0);
}

pub fn step(storage: &mut Storage, scene: &mut DemoScene<'_>, _delta_time: f32) {
    let input = &*scene.input;
    let spread = input.value(SPREAD, START_SPREAD);
    let clear = input.clicked.contains(&CLEAR);
    let pressed = input.pressed;
    let pointer = input.pointer;
    let stage = input.stage_size;
    let origin = input.camera_origin;
    let forward = input.camera_forward;
    let right = input.camera_right;
    let up = input.camera_up;
    let half_height = input.camera_half_height;

    let shapes: Vec<Entity> = storage
        .query_entities::<(PickRadius, LocalTransform)>()
        .collect();

    let picked = if pressed && stage.x > 1.0 && stage.y > 1.0 {
        let aspect = stage.x / stage.y;
        let direction = ray_direction(pointer, aspect, half_height, forward, right, up);
        nearest(storage, &shapes, origin, direction)
    } else {
        None
    };

    let first = -spread * (shapes.len() as f32 - 1.0) * 0.5;
    for entity in shapes {
        let Some(slot) = storage.get::<Slot>(entity).copied() else {
            continue;
        };
        let mut is_picked = storage.get::<Picked>(entity).copied().unwrap_or_default();
        if clear {
            is_picked.0 = false;
        }
        if let Some(hit) = picked
            && hit == entity
        {
            is_picked.0 = !is_picked.0;
        }
        storage.set(entity, is_picked);

        let name = if is_picked.0 {
            MATERIAL_PICKED
        } else {
            MATERIAL_PLAIN
        };
        let Some(wanted) = nightshade_render::queries::material_registry::material_registry_resolve(
            &*scene.materials,
            name,
        ) else {
            continue;
        };
        let same = storage
            .get::<nightshade_render::components::material::MaterialRef>(entity)
            .is_some_and(|held| held.id == wanted);
        if !same {
            storage.set(
                entity,
                nightshade_render::components::material::MaterialRef::new(wanted),
            );
        }

        if let Some(transform) = storage.get_mut::<LocalTransform>(entity) {
            transform.translation = Vec3::new(
                first + slot.0 as f32 * spread,
                if is_picked.0 { RISE } else { 0.0 },
                0.0,
            );
        }
    }
}

fn ray_direction(
    pointer: Vec2,
    aspect: f32,
    half_height: f32,
    forward: Vec3,
    right: Vec3,
    up: Vec3,
) -> Vec3 {
    let x = pointer.x * 2.0 - 1.0;
    let y = 1.0 - pointer.y * 2.0;
    (forward + right * (x * half_height * aspect) + up * (y * half_height)).normalize()
}

fn nearest(storage: &Storage, shapes: &[Entity], origin: Vec3, direction: Vec3) -> Option<Entity> {
    let mut best: Option<(f32, Entity)> = None;

    for entity in shapes {
        let Some(mark) = storage.get::<PickRadius>(*entity) else {
            continue;
        };
        let Some(global) = storage.get::<GlobalTransform>(*entity) else {
            continue;
        };
        let center = global.0.column(3).xyz();
        let Some(distance) = hits_sphere(origin, direction, center, mark.0) else {
            continue;
        };
        if best.is_none_or(|(nearest, _)| distance < nearest) {
            best = Some((distance, *entity));
        }
    }

    best.map(|(_, entity)| entity)
}

fn hits_sphere(origin: Vec3, direction: Vec3, center: Vec3, radius: f32) -> Option<f32> {
    let to_center = center - origin;
    let along = nalgebra_glm::dot(&to_center, &direction);
    if along < 0.0 {
        return None;
    }
    let closest = to_center - direction * along;
    let gap = nalgebra_glm::dot(&closest, &closest);
    let reach = radius * radius;
    if gap > reach {
        return None;
    }
    Some(along - (reach - gap).sqrt())
}
