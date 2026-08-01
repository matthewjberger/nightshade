use crate::components::{Stride, Velocity, Yaw};
use crate::data::ask::Ask;
use crate::data::motion::PERSON_RADIUS;
use crate::data::settings::Settings;
use crate::queries::collide::{lot_push, lots_near, props_near};
use crate::queries::grid::{surface_height, yaw_of};
use crate::resources::{City, Indoors, Player};
use nightshade::prelude::*;
use nightshade_camera_controller::contract::ThirdPersonCamera;
use nightshade_config::contract::Config;
use nightshade_input::Actions;
use nightshade_platform::prelude::Time;
use nightshade_scene::prelude::LocalTransform;

pub fn run(
    time: Res<Time>,
    actions: Res<Actions<Ask>>,
    config: Res<Config<Settings>>,
    mut player: ResMut<Player>,
    city: Res<City>,
    indoors: Res<Indoors>,
    mut cameras: Query<(&ThirdPersonCamera,)>,
    mut bodies: Query<(&mut Velocity, &mut Yaw, &mut Stride, &mut LocalTransform)>,
) {
    let Some(body) = player.body else {
        return;
    };
    if player.car.is_some() || player.flying {
        return;
    }
    let Some(camera) = player.walk_camera else {
        return;
    };
    let mut camera_yaw = 0.0;
    cameras.for_entity(camera, |(camera,)| camera_yaw = camera.yaw);

    let direction = actions.axis2(Ask::Walk);
    let sprinting = actions.pressed(Ask::Sprint);
    player.sprinting = sprinting && direction.norm() > 0.1;

    let forward = Vec3::new(-camera_yaw.sin(), 0.0, -camera_yaw.cos());
    let right = Vec3::new(camera_yaw.cos(), 0.0, -camera_yaw.sin());
    let pace = if sprinting {
        config.number(Settings::SprintSpeed)
    } else {
        config.number(Settings::WalkSpeed)
    };
    let acceleration = config.number(Settings::Acceleration);
    let wanted = (forward * direction.y + right * direction.x) * pace;
    let delta_time = time.delta_time;
    let indoor_floor = indoors.visit.map(|visit| visit.exit.y);

    let mut position = None;
    bodies.for_entity(body, |(velocity, yaw, stride, transform)| {
        velocity.0 += (wanted - velocity.0) * (acceleration * delta_time).min(1.0);
        let step = velocity.0 * delta_time;
        transform.translation += step;
        transform.translation.y =
            indoor_floor.unwrap_or_else(|| surface_height(transform.translation));
        stride.0 += step.norm();
        if velocity.0.norm() > 0.2 {
            yaw.0 = yaw_of(velocity.0);
        }
        transform.rotation = nalgebra_glm::quat_angle_axis(yaw.0, &Vec3::y());
        position = Some(transform.translation);
    });
    let Some(position) = position else {
        return;
    };
    let lots = if indoors.visit.is_some() {
        indoors.lots.clone()
    } else {
        let mut lots = lots_near(&city, position);
        lots.extend(
            props_near(&city, position)
                .into_iter()
                .map(|(_, _, lot)| lot),
        );
        lots
    };
    let push = lot_push(position, PERSON_RADIUS, &lots);
    if push.norm() > 0.0 {
        bodies.for_entity(body, |(velocity, _, _, transform)| {
            transform.translation += push;
            let normal = push.normalize();
            let into = velocity.0.dot(&normal);
            if into < 0.0 {
                velocity.0 -= normal * into;
            }
        });
    }
}
