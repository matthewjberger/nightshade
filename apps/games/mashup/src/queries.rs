use crate::data;
use crate::resources::{MeadowState, QuarryState};
use nightshade::prelude::{Mat4, Quat, Vec2, Vec3};
use nightshade_camera::queries::camera_look::camera_look_rotation;
use nightshade_portal::prelude::PortalPlane;
use nightshade_portal::queries::portal_frame;

pub fn eye_offset(yaw: f32) -> Vec3 {
    Vec3::new(
        yaw.sin() * data::CAMERA_REACH,
        data::CAMERA_HEIGHT,
        yaw.cos() * data::CAMERA_REACH,
    )
}

pub fn meadow_eye(meadow: &MeadowState) -> (Vec3, Quat) {
    let looked_at = meadow.focus + Vec3::new(0.0, 1.0, 0.0);
    let eye = looked_at + eye_offset(meadow.yaw);
    (eye, camera_look_rotation(eye, looked_at, 0.0))
}

pub fn quarry_rotation(quarry: &QuarryState) -> Quat {
    nalgebra_glm::quat_angle_axis(quarry.yaw, &Vec3::y())
        * nalgebra_glm::quat_angle_axis(quarry.pitch, &Vec3::x())
}

pub fn quarry_eye(quarry: &QuarryState) -> (Vec3, Quat) {
    (
        quarry.position + Vec3::new(0.0, data::EYE_HEIGHT, 0.0),
        quarry_rotation(quarry),
    )
}

pub fn pose_matrix(eye: Vec3, look: Quat) -> Mat4 {
    let mut matrix = nalgebra_glm::quat_to_mat4(&look);
    matrix[(0, 3)] = eye.x;
    matrix[(1, 3)] = eye.y;
    matrix[(2, 3)] = eye.z;
    matrix
}

pub fn column_of(x: f32, z: f32) -> (i32, i32) {
    (
        (x - data::QUARRY_ORIGIN.x).floor() as i32,
        (z - data::QUARRY_ORIGIN.z).floor() as i32,
    )
}

pub fn column_top(x: f32, z: f32) -> f32 {
    let (column_x, column_z) = column_of(x, z);
    if column_x.abs() > data::QUARRY_SPAN || column_z.abs() > data::QUARRY_SPAN {
        return 0.0;
    }
    data::column_height(column_x, column_z) as f32
}

pub fn standing_height(position: Vec3) -> f32 {
    let radius = data::QUARRY_RADIUS;
    let mut highest = f32::MIN;
    for offset in [
        Vec2::new(-radius, -radius),
        Vec2::new(radius, -radius),
        Vec2::new(-radius, radius),
        Vec2::new(radius, radius),
    ] {
        highest = highest.max(column_top(position.x + offset.x, position.z + offset.y));
    }
    highest
}

pub fn blocked(from: Vec3, to: Vec3) -> bool {
    standing_height(to) > from.y + data::QUARRY_STEP
}

pub fn matrix_pose(matrix: &Mat4) -> (Vec3, Quat) {
    let placement = nightshade_scene::contract::LocalTransform::from_matrix(matrix);
    (placement.translation, placement.rotation)
}

pub fn reached_portal(point: Vec3, heading: Vec3, portal: &PortalPlane) -> bool {
    let normal = portal.normal.normalize();
    let standoff = normal.dot(&(point - portal.center));
    if !(0.0..=data::PORTAL_REACH).contains(&standoff) {
        return false;
    }
    if heading.norm() > 0.1 && heading.normalize().dot(&normal) > -0.2 {
        return false;
    }
    let frame = portal_frame(portal);
    let right = Vec3::new(frame[(0, 0)], frame[(1, 0)], frame[(2, 0)]);
    let up = Vec3::new(frame[(0, 1)], frame[(1, 1)], frame[(2, 1)]);
    let flat = point - normal * standoff - portal.center;
    let across = right.dot(&flat) / (portal.width * 0.5);
    let along = up.dot(&flat) / (portal.height * 0.5);
    across * across + along * along <= 1.4
}

pub fn gate_point(resources: &crate::resources::MashupResources) -> (Vec3, Vec3) {
    match resources.here {
        crate::contract::WorldSide::Meadow => {
            let facing = resources.meadow.facing;
            (
                resources.meadow.feet + Vec3::new(0.0, 1.0, 0.0),
                Vec3::new(-facing.sin(), 0.0, -facing.cos()),
            )
        }
        crate::contract::WorldSide::Quarry => {
            let (eye, look) = quarry_eye(&resources.quarry);
            (
                eye,
                nalgebra_glm::quat_rotate_vec3(&look, &Vec3::new(0.0, 0.0, -1.0)),
            )
        }
    }
}

pub fn dive_target(portal: &PortalPlane) -> (Vec3, Quat) {
    let normal = portal.normal.normalize();
    let eye = portal.center - normal * data::DIVE_STANDOFF;
    let looked_at = portal.center - normal * 4.0;
    (eye, camera_look_rotation(eye, looked_at, 0.0))
}

pub fn world_field_of_view(side: crate::contract::WorldSide) -> f32 {
    match side {
        crate::contract::WorldSide::Meadow => data::MEADOW_FIELD_OF_VIEW,
        crate::contract::WorldSide::Quarry => data::FIELD_OF_VIEW,
    }
}

pub fn slices_for(velocity: Vec3, delta_time: f32) -> u32 {
    let travel = velocity.norm() * delta_time;
    let wanted = (travel / data::STEP_LIMIT).ceil().max(1.0);
    (wanted as u32).min(data::MAX_SLICES)
}

pub fn eased(share: f32) -> f32 {
    let held = share.clamp(0.0, 1.0);
    held * held * (3.0 - 2.0 * held)
}
