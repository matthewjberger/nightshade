use crate::data::{self, Hit};
use crate::resources::{PlayerState, Solid};
use nightshade::prelude::{Quat, Vec3};
use nightshade_portal::contract::PortalPlane;

pub fn player_rotation(player: &PlayerState) -> Quat {
    nalgebra_glm::quat_angle_axis(player.yaw, &Vec3::y())
        * nalgebra_glm::quat_angle_axis(player.pitch, &Vec3::x())
}

pub fn player_forward(player: &PlayerState) -> Vec3 {
    let pitch = player.pitch.cos();
    Vec3::new(
        -player.yaw.sin() * pitch,
        player.pitch.sin(),
        -player.yaw.cos() * pitch,
    )
}

pub fn eye_position(player: &PlayerState) -> Vec3 {
    player.position + Vec3::new(0.0, data::EYE_HEIGHT, 0.0)
}

pub fn solid_from(center: [f32; 3], size: [f32; 3]) -> Solid {
    let center = Vec3::new(center[0], center[1], center[2]);
    let half = Vec3::new(size[0], size[1], size[2]) * 0.5;
    Solid {
        min: center - half,
        max: center + half,
    }
}

pub fn cast(origin: Vec3, direction: Vec3, solids: &[Solid]) -> Option<Hit> {
    let mut best: Option<Hit> = None;
    for solid in solids {
        let mut entry = 0.0_f32;
        let mut exit = data::REACH;
        let mut axis = 0_usize;
        let mut facing = 1.0_f32;
        let mut reached = true;
        for index in 0..3 {
            if direction[index].abs() < 1.0e-6 {
                if origin[index] < solid.min[index] || origin[index] > solid.max[index] {
                    reached = false;
                    break;
                }
                continue;
            }
            let inverse = 1.0 / direction[index];
            let (near_face, far_face, side) = if inverse >= 0.0 {
                (solid.min[index], solid.max[index], -1.0)
            } else {
                (solid.max[index], solid.min[index], 1.0)
            };
            let near = (near_face - origin[index]) * inverse;
            let far = (far_face - origin[index]) * inverse;
            if near > entry {
                entry = near;
                axis = index;
                facing = side;
            }
            exit = exit.min(far);
            if entry > exit {
                reached = false;
                break;
            }
        }
        if !reached || entry <= 1.0e-4 {
            continue;
        }
        if best.as_ref().is_some_and(|held| held.distance <= entry) {
            continue;
        }
        let mut normal = Vec3::zeros();
        normal[axis] = facing;
        best = Some(Hit {
            point: origin + direction * entry,
            normal,
            distance: entry,
        });
    }
    best
}

pub fn body_reach(normal: Vec3) -> f32 {
    normal.x.abs() * data::PLAYER_HALF_WIDTH
        + normal.y.abs() * data::PLAYER_HEIGHT * 0.5
        + normal.z.abs() * data::PLAYER_HALF_WIDTH
}

pub fn body_center(player: &PlayerState) -> Vec3 {
    player.position + Vec3::new(0.0, data::PLAYER_HEIGHT * 0.5, 0.0)
}

pub fn landing_is_clear(position: Vec3, solids: &[Solid]) -> bool {
    let (min, max) = player_bounds(position);
    !solids.iter().any(|solid| overlaps(min, max, solid))
}

pub fn touching_portal(player: &PlayerState, portal: &PortalPlane) -> bool {
    let reach = body_reach(portal.normal.normalize());
    nightshade_portal::queries::touching_portal(
        body_center(player),
        reach,
        data::TRAVEL_MARGIN,
        portal,
    )
}

pub fn slices_for(velocity: Vec3, delta_time: f32) -> u32 {
    let travel = velocity.norm() * delta_time;
    let wanted = (travel / data::STEP_LIMIT).ceil().max(1.0);
    (wanted as u32).min(data::MAX_SLICES)
}

pub fn player_bounds(position: Vec3) -> (Vec3, Vec3) {
    let half = Vec3::new(data::PLAYER_HALF_WIDTH, 0.0, data::PLAYER_HALF_WIDTH);
    (
        position - half,
        position + half + Vec3::new(0.0, data::PLAYER_HEIGHT, 0.0),
    )
}

pub fn overlaps(min: Vec3, max: Vec3, solid: &Solid) -> bool {
    min.x < solid.max.x
        && max.x > solid.min.x
        && min.y < solid.max.y
        && max.y > solid.min.y
        && min.z < solid.max.z
        && max.z > solid.min.z
}

pub fn shortest_push(position: Vec3, axis: usize, solids: &[Solid]) -> Option<f32> {
    let (min, max) = player_bounds(position);
    let mut chosen: Option<f32> = None;
    for solid in solids {
        if !overlaps(min, max, solid) {
            continue;
        }
        let forward = solid.max[axis] - min[axis];
        let backward = solid.min[axis] - max[axis];
        let correction = if forward.abs() < backward.abs() {
            forward
        } else {
            backward
        };
        if chosen.is_none_or(|held: f32| correction.abs() < held.abs()) {
            chosen = Some(correction);
        }
    }
    chosen
}
