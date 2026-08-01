use crate::contract::WorldSide;
use crate::data;
use crate::resources::{MashupResources, Phase, PortalFrame};
use nightshade::prelude::{Entity, Mat4, Quat, Vec3};

pub fn remember_frame(
    resources: &mut MashupResources,
    side: WorldSide,
    model: Mat4,
    eye: Entity,
    fade: f32,
) {
    let rim = match side {
        WorldSide::Meadow => data::MEADOW_RIM,
        WorldSide::Quarry => data::QUARRY_RIM,
    };
    resources.frames[side.index()] = Some(PortalFrame {
        model,
        eye,
        rim,
        fade,
    });
}

pub fn begin_dive(
    resources: &mut MashupResources,
    from_eye: Vec3,
    from_look: Quat,
    to_eye: Vec3,
    to_look: Quat,
    toward: WorldSide,
) {
    resources.crossing.phase = Phase::Diving;
    resources.crossing.elapsed = 0.0;
    resources.crossing.from_eye = from_eye;
    resources.crossing.from_look = from_look;
    resources.crossing.to_eye = to_eye;
    resources.crossing.to_look = to_look;
    resources.crossing.toward = toward;
}

pub fn begin_settle(
    resources: &mut MashupResources,
    from_eye: Vec3,
    from_look: Quat,
    wanted_eye: Vec3,
) {
    resources.crossing.phase = Phase::Settling;
    resources.crossing.elapsed = 0.0;
    resources.crossing.from_eye = from_eye;
    resources.crossing.from_look = from_look;
    resources.crossing.hold = data::CROSS_HOLD;
    resources.crossing.settle_time = ((wanted_eye - from_eye).norm() / data::SETTLE_SPEED)
        .clamp(data::SETTLE_QUICKEST, data::SETTLE_SLOWEST);
}

pub fn finish_crossing(resources: &mut MashupResources) {
    resources.crossing.phase = Phase::Idle;
    resources.crossing.elapsed = 0.0;
}

pub fn arrive_in_meadow(resources: &mut MashupResources, eye: Vec3, look: Quat, gate_normal: Vec3) {
    resources.here = WorldSide::Meadow;
    let flat = nalgebra_glm::quat_rotate_vec3(&look, &Vec3::new(0.0, 0.0, -1.0));
    let heading = Vec3::new(flat.x, 0.0, flat.z);
    let feet = Vec3::new(
        eye.x
            .clamp(-data::MEADOW_HALF + 1.0, data::MEADOW_HALF - 1.0),
        data::MEADOW_GROUND,
        eye.z
            .clamp(-data::MEADOW_HALF + 1.0, data::MEADOW_HALF - 1.0),
    );
    resources.meadow.feet = feet;
    resources.meadow.focus = feet;
    if heading.norm() > 0.1 {
        resources.meadow.facing = (-heading.x).atan2(-heading.z);
    }
    let quarter = std::f32::consts::FRAC_PI_2;
    let mut best = 0;
    let mut highest = f32::MIN;
    for index in 0..4 {
        let yaw = index as f32 * quarter;
        let reach = yaw.sin() * gate_normal.x + yaw.cos() * gate_normal.z;
        if reach > highest {
            highest = reach;
            best = index;
        }
    }
    resources.meadow.yaw_index = best;
    resources.meadow.yaw = best as f32 * quarter;
}

pub fn arrive_in_quarry(resources: &mut MashupResources, eye: Vec3, look: Quat, ground: f32) {
    resources.here = WorldSide::Quarry;
    let flat = nalgebra_glm::quat_rotate_vec3(&look, &Vec3::new(0.0, 0.0, -1.0));
    let heading = Vec3::new(flat.x, 0.0, flat.z);
    resources.quarry.position = Vec3::new(eye.x, ground, eye.z);
    resources.quarry.velocity = Vec3::zeros();
    resources.quarry.pitch = 0.0;
    if heading.norm() > 0.1 {
        resources.quarry.yaw = (-heading.x).atan2(-heading.z);
    }
}
