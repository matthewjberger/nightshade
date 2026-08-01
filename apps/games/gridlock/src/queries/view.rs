use crate::data::scale::{AHEAD_LOAD, VIEW_BEHIND};
use crate::resources::Player;
use nightshade::ecs::Storage;
use nightshade::prelude::*;
use nightshade_camera::prelude::ActiveCamera;
use nightshade_scene::prelude::LocalTransform;

pub fn eye(active_camera: &ActiveCamera, storage: &Storage) -> Vec3 {
    active_camera
        .0
        .and_then(|camera| storage.get::<LocalTransform>(camera))
        .map(|transform| transform.translation)
        .unwrap_or_default()
}

pub fn forward(active_camera: &ActiveCamera, storage: &Storage) -> Vec3 {
    active_camera
        .0
        .and_then(|camera| storage.get::<LocalTransform>(camera))
        .map(|transform| {
            nalgebra_glm::quat_rotate_vec3(&transform.rotation, &Vec3::new(0.0, 0.0, -1.0))
        })
        .unwrap_or_else(|| Vec3::new(0.0, 0.0, 1.0))
}

pub fn view_weight(offset: Vec3, forward: Vec3) -> f32 {
    let flat = nalgebra_glm::Vec2::new(offset.x, offset.z);
    let facing = nalgebra_glm::Vec2::new(forward.x, forward.z);
    let reach = flat.norm() * facing.norm();
    if flat.dot(&facing) >= AHEAD_LOAD * reach {
        1.0
    } else {
        VIEW_BEHIND as f32
    }
}

pub fn anchor(player: &Player, storage: &Storage) -> Option<Vec3> {
    let entity = if player.flying {
        player.fly_camera
    } else {
        player.car.or(player.body)
    }?;
    storage
        .get::<LocalTransform>(entity)
        .map(|transform| transform.translation)
}

pub fn look_angles(rotation: &nalgebra_glm::Quat) -> (f32, f32) {
    let forward = nalgebra_glm::quat_rotate_vec3(rotation, &Vec3::new(0.0, 0.0, -1.0));
    let yaw = (-forward.x).atan2(-forward.z);
    let pitch = forward.y.clamp(-1.0, 1.0).asin();
    (yaw, pitch)
}

pub fn look_rotation(yaw: f32, pitch: f32) -> nalgebra_glm::Quat {
    nalgebra_glm::quat_angle_axis(yaw, &Vec3::y())
        * nalgebra_glm::quat_angle_axis(pitch, &Vec3::x())
}

pub fn is_night(hour: f32, from: f32, until: f32) -> bool {
    hour >= from || hour < until
}
