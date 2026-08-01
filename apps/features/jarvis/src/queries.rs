use nightshade::prelude::*;
use nightshade_graphics::prelude::*;
use nightshade_openxr::contract::{XrHand, XrHandState, XrPose, XrRig};

pub fn origin_pose(storage: &nightshade::ecs::Storage, rig: &XrRig) -> XrPose {
    storage
        .get::<LocalTransform>(rig.origin)
        .map(|transform| XrPose {
            position: transform.translation,
            orientation: transform.rotation,
        })
        .unwrap_or_default()
}

pub fn to_world(origin: &XrPose, point: Vec3) -> Vec3 {
    origin.position + nalgebra_glm::quat_rotate_vec3(&origin.orientation, &point)
}

pub fn to_stage(origin: &XrPose, point: Vec3) -> Vec3 {
    nalgebra_glm::quat_rotate_vec3(
        &nalgebra_glm::quat_inverse(&origin.orientation),
        &(point - origin.position),
    )
}

pub fn direction_to_world(origin: &XrPose, direction: Vec3) -> Vec3 {
    nalgebra_glm::quat_rotate_vec3(&origin.orientation, &direction)
}

pub fn direction_to_stage(origin: &XrPose, direction: Vec3) -> Vec3 {
    nalgebra_glm::quat_rotate_vec3(&nalgebra_glm::quat_inverse(&origin.orientation), &direction)
}

pub fn hand_pose(storage: &nightshade::ecs::Storage, rig: &XrRig, hand: XrHand) -> Option<XrPose> {
    storage
        .get::<LocalTransform>(rig.hand(hand))
        .map(|transform| XrPose {
            position: transform.translation,
            orientation: transform.rotation,
        })
}

pub fn palm_direction(hand: XrHand, orientation: &Quat) -> Vec3 {
    let axis = match hand {
        XrHand::Left => Vec3::x(),
        XrHand::Right => -Vec3::x(),
    };
    nalgebra_glm::quat_rotate_vec3(orientation, &axis)
}

pub fn palm_axis(hand: XrHand) -> Vec3 {
    match hand {
        XrHand::Left => Vec3::x(),
        XrHand::Right => -Vec3::x(),
    }
}

pub fn heading_degrees(orientation: &Quat) -> f32 {
    let forward = nalgebra_glm::quat_rotate_vec3(orientation, &-Vec3::z());
    let degrees = forward.x.atan2(-forward.z).to_degrees();
    degrees.rem_euclid(360.0)
}

pub fn ease(value: f32) -> f32 {
    let clamped = value.clamp(0.0, 1.0);
    clamped * clamped * (3.0 - 2.0 * clamped)
}

pub fn aim_points_down(state: &XrHandState) -> bool {
    state.aim.is_some_and(|aim| {
        let direction = nalgebra_glm::quat_rotate_vec3(&aim.orientation, &-Vec3::z());
        direction.dot(&-Vec3::y()) > crate::theme::ARM_DOWN_COSINE
    })
}

pub fn drone_alive(storage: &nightshade::ecs::Storage, entity: Entity) -> bool {
    storage
        .get::<crate::components::Respawn>(entity)
        .is_some_and(|respawn| respawn.0 <= 0.0)
}
