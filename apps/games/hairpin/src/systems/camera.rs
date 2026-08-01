use crate::components::{Velocity, Yaw};
use crate::data::{CAMERA_AIM, CAMERA_BACK, CAMERA_HEIGHT, CAMERA_LAG, OVERVIEW_AIM, OVERVIEW_EYE};
use crate::resources::{HairpinResources, Screen};
use nightshade::ecs::Storage;
use nightshade::prelude::*;
use nightshade_camera::queries::camera_look::camera_look_rotation;
use nightshade_platform::prelude::Time;
use nightshade_scene::prelude::LocalTransform;

pub fn follow(resources: Res<HairpinResources>, time: Res<Time>, storage: &mut Storage) {
    let (Some(car), Some(camera)) = (resources.track.car, resources.track.camera) else {
        return;
    };
    if resources.screen == Screen::Title {
        if let Some(transform) = storage.get_mut::<LocalTransform>(camera) {
            transform.translation = OVERVIEW_EYE;
            transform.rotation = camera_look_rotation(OVERVIEW_EYE, OVERVIEW_AIM, 0.0);
        }
        return;
    }
    let Some(focus) = storage
        .get::<LocalTransform>(car)
        .map(|transform| transform.translation)
    else {
        return;
    };
    let Some(velocity) = storage.get::<Velocity>(car).map(|velocity| velocity.0) else {
        return;
    };
    let Some(yaw) = storage.get::<Yaw>(car).map(|yaw| yaw.0) else {
        return;
    };

    let travel = if velocity.norm() > 1.0 {
        velocity.normalize()
    } else {
        Vec3::new(yaw.sin(), 0.0, yaw.cos())
    };
    let eye = focus - travel * CAMERA_BACK + Vec3::y() * CAMERA_HEIGHT;

    let Some(transform) = storage.get_mut::<LocalTransform>(camera) else {
        return;
    };
    transform.translation +=
        (eye - transform.translation) * (CAMERA_LAG * time.delta_time).min(1.0);
    transform.rotation =
        camera_look_rotation(transform.translation, focus + Vec3::y() * CAMERA_AIM, 0.0);
}
