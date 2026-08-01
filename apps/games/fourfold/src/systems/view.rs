use crate::data;
use crate::queries;
use crate::resources::FourfoldResources;
use nightshade::prelude::*;
use nightshade_scene::prelude::LocalTransform;

pub fn place_hub_camera(resources: Res<FourfoldResources>, mut commands: Commands) {
    if resources.hub_camera == Entity::default() {
        return;
    }
    commands.set(
        resources.hub_camera,
        LocalTransform {
            translation: queries::eye_position(&resources.walker),
            rotation: queries::look_rotation(resources.walker.yaw, resources.walker.pitch),
            scale: Vec3::new(1.0, 1.0, 1.0),
        },
    );
}

pub fn place_cairn_camera(resources: Res<FourfoldResources>, mut commands: Commands) {
    if resources.cairn_camera == Entity::default() {
        return;
    }
    commands.set(
        resources.cairn_camera,
        LocalTransform {
            translation: queries::eye_position(&resources.digger),
            rotation: queries::look_rotation(resources.digger.yaw, resources.digger.pitch),
            scale: Vec3::new(1.0, 1.0, 1.0),
        },
    );
}

pub fn place_hearth_camera(resources: Res<FourfoldResources>, mut commands: Commands) {
    if resources.hearth_camera == Entity::default() {
        return;
    }
    let focus = resources.villager.focus;
    commands.set(
        resources.hearth_camera,
        LocalTransform {
            translation: focus + Vec3::new(0.0, data::HEARTH_LIFT, data::HEARTH_BACK),
            rotation: nalgebra_glm::quat_angle_axis(data::HEARTH_TILT, &Vec3::x()),
            scale: Vec3::new(1.0, 1.0, 1.0),
        },
    );
}

pub fn place_glass_camera(resources: Res<FourfoldResources>, mut commands: Commands) {
    if resources.glass_camera == Entity::default() {
        return;
    }
    commands.set(
        resources.glass_camera,
        LocalTransform {
            translation: queries::glass_eye(&resources.runner),
            rotation: Quat::identity(),
            scale: Vec3::new(1.0, 1.0, 1.0),
        },
    );
}
