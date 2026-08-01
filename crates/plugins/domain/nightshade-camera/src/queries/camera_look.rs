use nalgebra_glm::{Quat, Vec3};

pub fn camera_look_rotation(eye: Vec3, target: Vec3, roll_degrees: f32) -> Quat {
    let direction = target - eye;
    if direction.magnitude_squared() < f32::EPSILON {
        return Quat::identity();
    }
    let direction = direction.normalize();
    let yaw = (-direction.x).atan2(-direction.z);
    let pitch = (-direction.y).clamp(-1.0, 1.0).asin();
    let yaw_rotation = nalgebra_glm::quat_angle_axis(yaw, &Vec3::y());
    let pitch_rotation = nalgebra_glm::quat_angle_axis(-pitch, &Vec3::x());
    let look = yaw_rotation * pitch_rotation;
    if roll_degrees.abs() < f32::EPSILON {
        return look;
    }
    let roll = nalgebra_glm::quat_angle_axis(roll_degrees.to_radians(), &direction);
    roll * look
}
