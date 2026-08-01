use nalgebra_glm::{Quat, Vec3, quat_angle_axis};

pub fn facing(direction: Vec3) -> Quat {
    let aim = if direction.magnitude() > f32::EPSILON {
        direction.normalize()
    } else {
        -Vec3::z()
    };
    let yaw = (-aim.x).atan2(-aim.z);
    let pitch = aim.y.asin();
    quat_angle_axis(yaw, &Vec3::y()) * quat_angle_axis(pitch, &Vec3::x())
}

pub fn mix(low: [f32; 4], high: [f32; 4], share: f32) -> [f32; 4] {
    [
        low[0] + (high[0] - low[0]) * share,
        low[1] + (high[1] - low[1]) * share,
        low[2] + (high[2] - low[2]) * share,
        1.0,
    ]
}
