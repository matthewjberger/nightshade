use crate::components::{Body, Cell, Steer, Velocity, Yaw};
use crate::data::ask::Ask;
use crate::data::motion::{KNOCK_SCRUB, TURN_REFERENCE};
use crate::data::settings::Settings;
use crate::queries::collide::{hull_at, hull_touches, props_near};
use crate::queries::grid::{cell_of, surface_height};
use crate::resources::{City, Knock, Knocks, Player};
use nightshade::prelude::*;
use nightshade_config::contract::Config;
use nightshade_input::Actions;
use nightshade_platform::prelude::Time;
use nightshade_scene::prelude::LocalTransform;

type Wheels<'world, 'item> = nightshade::ecs::system_param::Query<
    'world,
    (
        &'item mut Velocity,
        &'item mut Yaw,
        &'item mut Steer,
        &'item mut LocalTransform,
        &'item mut Cell,
        Option<&'item Body>,
    ),
>;

pub fn update(
    time: Res<Time>,
    actions: Res<Actions<Ask>>,
    config: Res<Config<Settings>>,
    player: Res<Player>,
    city: Res<City>,
    mut knocks: ResMut<Knocks>,
    mut cars: Wheels<'_, '_>,
) {
    let Some(car) = player.car else {
        return;
    };
    if player.flying {
        return;
    }
    let delta_time = time.delta_time;
    let steering = actions.axis(Ask::Steer).clamp(-1.0, 1.0);
    let drive = actions.axis(Ask::Drive).clamp(-1.0, 1.0);
    let throttle = drive.max(0.0);
    let braking = (-drive).max(0.0);
    let handbrake = actions.pressed(Ask::Handbrake);
    let engine = config.number(Settings::Engine);
    let brakes = config.number(Settings::Braking);
    let top_speed = config.number(Settings::TopSpeed);
    let reverse = config.number(Settings::Reverse);
    let drag = config.number(Settings::Drag);
    let turn_rate = config.number(Settings::TurnRate);
    let steer_rate = config.number(Settings::SteerRate);
    let knock_speed = config.number(Settings::KnockSpeed);
    let grip = if handbrake {
        config.number(Settings::DriftGrip)
    } else {
        config.number(Settings::Grip)
    };

    let mut moved = None;
    cars.for_entity(car, |(velocity, yaw, steer, transform, cell, body)| {
        steer.0 += (steering - steer.0) * (steer_rate * delta_time).min(1.0);

        let heading = Vec3::new(yaw.0.sin(), 0.0, yaw.0.cos());
        let along = velocity.0.dot(&heading);
        let sideways = velocity.0 - heading * along;
        yaw.0 -= steer.0 * turn_rate * (along / (along.abs() + TURN_REFERENCE)) * delta_time;

        let along = (along + (throttle * engine - braking * brakes) * delta_time)
            .clamp(-reverse, top_speed);
        let heading = Vec3::new(yaw.0.sin(), 0.0, yaw.0.cos());
        velocity.0 = (heading * along + sideways * (1.0 - (grip * delta_time).min(1.0)))
            * (1.0 - (drag * delta_time).min(1.0));

        transform.translation += velocity.0 * delta_time;
        transform.translation.y = surface_height(transform.translation);
        transform.rotation = nalgebra_glm::quat_angle_axis(yaw.0, &Vec3::y());
        cell.0 = cell_of(transform.translation);
        moved = body.map(|body| (transform.translation, velocity.0, yaw.0, body.0));
    });
    let Some((position, velocity, yaw, body)) = moved else {
        return;
    };
    let hull = hull_at(&body, position, yaw);
    let velocity = Vec3::new(velocity.x, 0.0, velocity.z);
    let speed = velocity.norm();
    let mut knocked = 0;
    for (key, index, lot) in props_near(&city, position) {
        if speed > knock_speed && hull_touches(&hull, &lot) {
            knocks.pending.push(Knock {
                key,
                index,
                direction: velocity / speed,
            });
            knocked += 1;
        }
    }
    if knocked > 0 {
        cars.for_entity(car, |(velocity, _, _, _, _, _)| {
            velocity.0 *= KNOCK_SCRUB.powi(knocked);
        });
    }
}
