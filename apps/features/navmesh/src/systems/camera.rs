use crate::components::Facing;
use crate::resources::{Behavior, Interface, Viewpoint};
use nightshade::prelude::*;

pub fn gate_orbit_camera(
    behavior: Res<Behavior>,
    viewpoint: Res<Viewpoint>,
    mut orbits: Query<(&mut nightshade_camera_controller::contract::PanOrbitCamera,)>,
) {
    let follow = behavior.follow;
    let Some(camera) = viewpoint.0 else {
        return;
    };
    orbits.for_entity(camera, |(orbit,)| {
        if orbit.enabled == follow {
            orbit.enabled = !follow;
        }
    });
}

pub fn update_follow_camera(
    mut behavior: ResMut<Behavior>,
    viewpoint: Res<Viewpoint>,
    panel: Res<Interface>,
    storage: &mut nightshade::ecs::Storage,
) {
    if !behavior.follow {
        return;
    }

    let Some(followed) = behavior.followed else {
        return;
    };

    if !storage.is_alive(followed) {
        behavior.followed = None;
        behavior.follow = false;
        if let Some(data) =
            storage.get_mut::<nightshade_ui::components::UiCheckboxData>(panel.follow_mode)
        {
            data.value = false;
        }
        return;
    }

    let Some(camera_entity) = viewpoint.0 else {
        return;
    };

    let facing = storage
        .get::<Facing>(followed)
        .map(|value| value.0)
        .unwrap_or(0.0);
    let fox_position = storage
        .get::<nightshade_scene::contract::LocalTransform>(followed)
        .map(|transform| transform.translation)
        .unwrap_or(Vec3::zeros());

    let camera_offset = nalgebra_glm::vec3(0.0, 8.0, 12.0);
    let rotated_offset = nalgebra_glm::rotate_vec3(&camera_offset, facing, &Vec3::y());
    let camera_position = fox_position + rotated_offset;
    let look_target = fox_position + nalgebra_glm::vec3(0.0, 1.0, 0.0);

    if let Some(transform) =
        storage.get_mut::<nightshade_scene::contract::LocalTransform>(camera_entity)
    {
        transform.translation = camera_position;
        let direction = nalgebra_glm::normalize(&(look_target - camera_position));
        let up = Vec3::y();
        let right = nalgebra_glm::normalize(&nalgebra_glm::cross(&direction, &up));
        let corrected_up = nalgebra_glm::cross(&right, &direction);
        let rotation_matrix = nalgebra_glm::mat3(
            right.x,
            corrected_up.x,
            -direction.x,
            right.y,
            corrected_up.y,
            -direction.y,
            right.z,
            corrected_up.z,
            -direction.z,
        );
        transform.rotation = nalgebra_glm::mat3_to_quat(&rotation_matrix);
    }
}
