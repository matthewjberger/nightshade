use crate::components::{Arc, Avatar, Fit, Look, Pace, Ring};
use crate::data::{HERD, HERD_MEMBERS, HERD_SPEED, Prop, course};
use crate::queries::model;
use crate::theme;
use nalgebra_glm::{Vec3, quat_angle_axis};
use nightshade::prelude::*;
use nightshade_physics::commands::spawn_static_physics_cube_with_material;
use nightshade_physics::prelude::{ColliderComponent, RigidBodyComponent};
use nightshade_render::data::material::Material;
use nightshade_scene::prelude::{GlobalTransform, LocalTransform, Name};

pub fn build(commands: &mut Commands) {
    ground(commands);
    for prop in course() {
        place(commands, &prop);
    }
    herd(commands);
}

fn ground(commands: &mut Commands) {
    let material = Material {
        base_color: theme::GROUND_COLOR,
        metallic: 0.0,
        roughness: 0.95,
        ..Default::default()
    };
    let entity = spawn_static_physics_cube_with_material(
        commands,
        Vec3::new(0.0, -0.5, 0.0),
        Vec3::new(theme::GROUND_EXTENT, 1.0, theme::GROUND_EXTENT),
        material,
    );
    commands.queue(move |storage, _resources| {
        if let Some(name) = storage.get_mut::<Name>(entity) {
            name.0 = "Ground".to_string();
        }
    });
}

fn place(commands: &mut Commands, prop: &Prop) {
    commands.spawn((
        Name(prop.model.to_string()),
        LocalTransform {
            translation: prop.position,
            rotation: quat_angle_axis(prop.yaw, &Vec3::y()),
            scale: Vec3::new(1.0, 1.0, 1.0),
        },
        GlobalTransform::default(),
        Look(model(prop.model)),
        Fit(prop.scale),
        Avatar(None),
    ));
    let Some(block) = &prop.block else {
        return;
    };
    let center = prop.position + Vec3::new(0.0, block.lift, 0.0);
    let half = block.half;
    let spin = quat_angle_axis(prop.yaw, &Vec3::y()) * quat_angle_axis(block.pitch, &Vec3::x());
    let collider = commands.spawn((
        Name(format!("{} block", prop.model)),
        LocalTransform {
            translation: center,
            rotation: spin,
            scale: Vec3::new(1.0, 1.0, 1.0),
        },
        GlobalTransform::default(),
    ));
    commands.queue(move |storage, _resources| {
        storage.set(
            collider,
            RigidBodyComponent::new_static()
                .with_translation(center.x, center.y, center.z)
                .with_rotation(spin.i, spin.j, spin.k, spin.w),
        );
        storage.set(
            collider,
            ColliderComponent::new_cuboid(half.x, half.y, half.z)
                .with_friction(0.9)
                .with_restitution(0.0),
        );
    });
}

fn herd(commands: &mut Commands) {
    for (index, (name, radius, scale)) in HERD.iter().enumerate() {
        for member in 0..HERD_MEMBERS {
            let arc = (index * HERD_MEMBERS + member) as f32 * std::f32::consts::TAU
                / (HERD.len() * HERD_MEMBERS) as f32;
            commands.spawn((
                Name((*name).to_string()),
                LocalTransform {
                    translation: Vec3::new(arc.sin() * radius, 0.0, arc.cos() * radius),
                    rotation: nalgebra_glm::Quat::identity(),
                    scale: Vec3::new(1.0, 1.0, 1.0),
                },
                GlobalTransform::default(),
                Look(model(name)),
                Fit(*scale * theme::ANIMAL_SCALE),
                Avatar(None),
                Ring(*radius),
                Arc(arc),
                Pace(HERD_SPEED / radius),
            ));
        }
    }
}
