use crate::components::Speed;
use crate::data::{
    BASE_VIEW_DEGREES, EYE_OFFSET, VIEW_EASE_RATE, VIEW_SPEED_RANGE, VIEW_STEP_DEGREES,
    WIDE_VIEW_DEGREES,
};
use crate::resources::TetherResources;
use nightshade::prelude::*;
use nightshade_camera::prelude::{Camera, Projection};
use nightshade_platform::prelude::Time;
use nightshade_scene::prelude::LocalTransform;

pub fn follow_the_runner(
    time: Res<Time>,
    resources: Res<TetherResources>,
    mut commands: Commands,
    mut bodies: Query<(&LocalTransform, Option<&Speed>)>,
) {
    let Some(runner) = resources.scene.runner else {
        return;
    };
    let Some(camera) = resources.scene.camera else {
        return;
    };

    let mut body = None;
    let mut speed = 0.0;
    bodies.for_entity(runner, |(transform, runner_speed)| {
        body = Some(transform.translation);
        speed = runner_speed.map(|speed| speed.0).unwrap_or(0.0);
    });
    let Some(body) = body else {
        return;
    };

    let mut eye = None;
    bodies.for_entity(camera, |(transform, _)| eye = Some(*transform));
    let Some(mut eye) = eye else {
        return;
    };
    eye.translation = body + Vec3::new(0.0, EYE_OFFSET, 0.0);
    commands.set(camera, eye);

    let reach = (speed / VIEW_SPEED_RANGE).clamp(0.0, 1.0);
    let wanted = BASE_VIEW_DEGREES + (WIDE_VIEW_DEGREES - BASE_VIEW_DEGREES) * reach;
    let blend = (time.delta_time * VIEW_EASE_RATE).clamp(0.0, 1.0);

    commands.queue(move |storage, _| {
        let Some(entity_camera) = storage.get_mut::<Camera>(camera) else {
            return;
        };
        let Projection::Perspective(ref mut perspective) = entity_camera.projection else {
            return;
        };
        let held = perspective.y_fov_rad.to_degrees();
        let eased = held + (wanted - held) * blend;
        if (eased - held).abs() >= VIEW_STEP_DEGREES {
            perspective.y_fov_rad = eased.to_radians();
        }
    });
}
