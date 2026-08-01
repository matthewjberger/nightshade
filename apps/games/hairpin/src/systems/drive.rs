use crate::components::{Steer, Velocity, Yaw};
use crate::data::*;
use crate::queries::{pad_pressed, pointers_held, viewport_size, wall_push};
use crate::resources::{HairpinResources, Screen};
use nightshade::prelude::*;
use nightshade_input::Actions;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::LocalTransform;

pub fn update(
    time: Res<Time>,
    input: Res<Input>,
    actions: Res<Actions<Ask>>,
    window: Res<Window>,
    resources: Res<HairpinResources>,
    cars: Query<(&mut Velocity, &mut Yaw, &mut Steer, &mut LocalTransform)>,
) {
    if resources.screen != Screen::Playing {
        return;
    }
    let delta_time = time.delta_time;
    let (steering, throttle, braking, handbrake) = controls(&input, &actions, &window);

    cars.for_each(|_entity, (velocity, yaw, steer, transform)| {
        steer.0 += (steering - steer.0) * (STEER_RATE * delta_time).min(1.0);

        let heading = Vec3::new(yaw.0.sin(), 0.0, yaw.0.cos());
        let along = velocity.0.dot(&heading);
        let sideways = velocity.0 - heading * along;
        yaw.0 -= steer.0 * TURN_RATE * (along / (along.abs() + TURN_REFERENCE)) * delta_time;

        let along = (along + (throttle * ENGINE - braking * BRAKING) * delta_time)
            .clamp(-REVERSE_SPEED, MAXIMUM_SPEED);
        let heading = Vec3::new(yaw.0.sin(), 0.0, yaw.0.cos());
        let grip = if handbrake { DRIFT_GRIP } else { GRIP };
        velocity.0 = (heading * along + sideways * (1.0 - (grip * delta_time).min(1.0)))
            * (1.0 - (DRAG * delta_time).min(1.0));

        transform.translation += velocity.0 * delta_time;
        transform.translation.y = 0.0;

        if let Some(push) = wall_push(transform.translation, &resources.track.limits) {
            transform.translation += push;
            let normal = push.normalize();
            let into_wall = velocity.0.dot(&normal);
            if into_wall < 0.0 {
                velocity.0 -= normal * into_wall;
            }
            velocity.0 *= 1.0 - (WALL_SCRUB * delta_time).min(1.0);
        }

        transform.rotation = nalgebra_glm::quat_angle_axis(yaw.0, &Vec3::y());
    });
}

fn controls(input: &Input, actions: &Actions<Ask>, window: &Window) -> (f32, f32, f32, bool) {
    let drive = actions.axis(Ask::Drive);
    let mut steering = actions.axis(Ask::Steer);
    let mut throttle = drive.max(0.0);
    let mut braking = (-drive).max(0.0);
    let mut handbrake = actions.pressed(Ask::Handbrake);

    if TOUCH_CONTROLS {
        let viewport = viewport_size(window);
        for point in pointers_held(input) {
            steering += f32::from(pad_pressed(PAD_AREAS[PAD_STEER_RIGHT], point, viewport))
                - f32::from(pad_pressed(PAD_AREAS[PAD_STEER_LEFT], point, viewport));
            throttle += f32::from(pad_pressed(PAD_AREAS[PAD_THROTTLE], point, viewport));
            braking += f32::from(pad_pressed(PAD_AREAS[PAD_BRAKE], point, viewport));
            handbrake |= pad_pressed(PAD_AREAS[PAD_DRIFT], point, viewport);
        }
    } else if input.mouse.state.contains(MouseState::LEFT_CLICKED) {
        steering = (input.mouse.position.x / viewport_size(window).x - 0.5) * MOUSE_LOCK;
        throttle = 1.0;
    }
    if input.mouse.state.contains(MouseState::RIGHT_CLICKED) {
        braking = 1.0;
    }

    (
        steering.clamp(-1.0, 1.0),
        throttle.clamp(0.0, 1.0),
        braking.clamp(0.0, 1.0),
        handbrake,
    )
}
