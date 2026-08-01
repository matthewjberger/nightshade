use crate::components::Extent;
use crate::contract::{Dimension, Glyph};
use crate::data;
use crate::resources::{ArchState, Runner, Walker};
use nightshade::prelude::{Quat, Query, Vec3};
use nightshade_scene::prelude::LocalTransform;

#[derive(Clone, Copy, Debug)]
pub struct Aabb {
    pub min: Vec3,
    pub max: Vec3,
}

pub fn slab_box(center: Vec3, size: Vec3) -> Aabb {
    let half = size * 0.5;
    Aabb {
        min: center - half,
        max: center + half,
    }
}

pub fn body_box(position: Vec3) -> Aabb {
    let radius = data::BODY_RADIUS;
    Aabb {
        min: Vec3::new(position.x - radius, position.y, position.z - radius),
        max: Vec3::new(
            position.x + radius,
            position.y + data::BODY_HEIGHT,
            position.z + radius,
        ),
    }
}

pub fn overlaps(left: &Aabb, right: &Aabb) -> bool {
    left.min.x < right.max.x
        && left.max.x > right.min.x
        && left.min.y < right.max.y
        && left.max.y > right.min.y
        && left.min.z < right.max.z
        && left.max.z > right.min.z
}

pub fn slices_for(velocity: Vec3, delta_time: f32) -> u32 {
    let travel = velocity.norm() * delta_time;
    let wanted = (travel / data::STEP_LIMIT).ceil().max(1.0);
    (wanted as u32).clamp(1, data::MAX_SLICES)
}

pub fn slide(
    position: Vec3,
    velocity: Vec3,
    solids: &[Aabb],
    delta_time: f32,
) -> (Vec3, Vec3, bool) {
    let slices = slices_for(velocity, delta_time);
    let slice = delta_time / slices as f32;
    let mut moved = position;
    let mut speed = velocity;
    let mut grounded = false;
    for _ in 0..slices {
        for axis in [0_usize, 2, 1] {
            let travel = speed[axis] * slice;
            if travel == 0.0 {
                continue;
            }
            moved[axis] += travel;
            if let Some(push) = correction(moved, solids, axis, travel) {
                moved[axis] += push;
                if axis == 1 && travel < 0.0 {
                    grounded = true;
                }
                speed[axis] = 0.0;
            }
        }
    }
    (moved, speed, grounded)
}

fn correction(position: Vec3, solids: &[Aabb], axis: usize, travel: f32) -> Option<f32> {
    let body = body_box(position);
    let mut push = 0.0_f32;
    for block in solids {
        if !overlaps(&body, block) {
            continue;
        }
        let step = if travel > 0.0 {
            block.min[axis] - body.max[axis]
        } else {
            block.max[axis] - body.min[axis]
        };
        if step.abs() > push.abs() {
            push = step;
        }
    }
    if push == 0.0 { None } else { Some(push) }
}

pub fn gather_solids(
    blocks: Query<(&Dimension, &Extent, &LocalTransform)>,
    wanted: Dimension,
) -> Vec<Aabb> {
    let mut solids = Vec::new();
    blocks.for_each(|_entity, (dimension, extent, transform)| {
        if *dimension == wanted {
            solids.push(slab_box(transform.translation, extent.0));
        }
    });
    solids
}

pub fn look_rotation(yaw: f32, pitch: f32) -> Quat {
    nalgebra_glm::quat_angle_axis(yaw, &Vec3::y())
        * nalgebra_glm::quat_angle_axis(pitch, &Vec3::x())
}

pub fn eye_position(walker: &Walker) -> Vec3 {
    walker.position + Vec3::new(0.0, data::EYE_HEIGHT, 0.0)
}

pub fn forward_of_pose(yaw: f32, pitch: f32) -> Vec3 {
    nalgebra_glm::quat_rotate_vec3(&look_rotation(yaw, pitch), &Vec3::new(0.0, 0.0, -1.0))
}

pub fn forward_of(yaw: f32) -> Vec3 {
    Vec3::new(-yaw.sin(), 0.0, -yaw.cos())
}

pub fn right_of(yaw: f32) -> Vec3 {
    Vec3::new(yaw.cos(), 0.0, -yaw.sin())
}

pub fn glass_eye(runner: &Runner) -> Vec3 {
    Vec3::new(
        runner.focus.x,
        runner.focus.y + data::GLASS_CAMERA_LIFT,
        data::GLASS_CAMERA_Z,
    )
}

pub fn within_dials(walker: &Walker) -> bool {
    let middle = data::dial_center(1);
    let flat = Vec3::new(
        walker.position.x - middle.x,
        0.0,
        walker.position.z - middle.z,
    );
    flat.norm() <= data::DIAL_REACH
}

pub fn in_arch(walker: &Walker, gate: &data::Gate) -> bool {
    let along = Vec3::new(gate.normal.x.abs(), 0.0, gate.normal.z.abs());
    let across = Vec3::new(along.z, 0.0, along.x);
    let size = across * data::ARCH_WIDTH + along * 2.0 + Vec3::new(0.0, data::ARCH_HEIGHT, 0.0);
    let center = Vec3::new(
        gate.center.x + gate.normal.x * -data::PORTAL_INSET,
        data::ARCH_HEIGHT * 0.5,
        gate.center.z + gate.normal.z * -data::PORTAL_INSET,
    );
    overlaps(&body_box(walker.position), &slab_box(center, size))
}

pub fn at_gate(runner: &Runner) -> bool {
    if !runner.armed {
        return false;
    }
    let gate = slab_box(data::GLASS_GATE, data::GLASS_GATE_SIZE);
    overlaps(&body_box(runner.position), &gate)
}

pub fn at_spot(position: Vec3, target: Vec3, reach: f32) -> bool {
    let flat = Vec3::new(position.x - target.x, 0.0, position.z - target.z);
    flat.norm() <= reach
}

pub fn tone_index(dimension: Dimension) -> usize {
    data::TONES
        .iter()
        .position(|tone| tone.home == dimension)
        .unwrap_or(0)
}

pub fn wanted_glyph(dial: usize) -> Glyph {
    data::TONES
        .iter()
        .find(|tone| tone.marks == dial + 1)
        .map(|tone| tone.glyph)
        .unwrap_or_default()
}

pub fn dials_match(arch: &ArchState) -> bool {
    (0..data::DIAL_COUNT).all(|dial| arch.dials[dial] == wanted_glyph(dial))
}

pub fn dial_reading(arch: &ArchState) -> String {
    arch.dials
        .iter()
        .map(|glyph| glyph.name())
        .collect::<Vec<&str>>()
        .join(" ")
}
