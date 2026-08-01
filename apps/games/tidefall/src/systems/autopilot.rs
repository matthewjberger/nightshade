use crate::components::{Autopilot, Footing, Ground, Locked, Push, Velocity};
use crate::data::autopilot::Event as PilotEvent;
use crate::data::footing::Event as FootingEvent;
use crate::data::{
    ALIGNED_ANGLE, ARRIVAL_FACTOR, ARRIVE_SPEED, AUTOPILOT_TURN, Ask, BRAKE_MARGIN, DESCENT_GAIN,
    DESCENT_MAX, DESCENT_MIN, DESCENT_RESPONSE, LIFT_OFF, MAX_STEP, Settings,
};
use crate::queries::{Bodies, altitude, collect_bodies, nearest_body};
use crate::resources::{Mode, TidefallResources};
use nightshade::prelude::*;
use nightshade_config::prelude::Config;
use nightshade_input::Actions;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::*;

pub type Pilots<'world, 'item> = Query<
    'world,
    (
        &'item mut Velocity,
        &'item mut Footing,
        &'item Ground,
        &'item mut Locked,
        &'item mut Autopilot,
        &'item mut Push,
        &'item mut LocalTransform,
    ),
>;

pub fn steer(
    time: Res<Time>,
    actions: Res<Actions<Ask>>,
    config: Res<Config<Settings>>,
    resources: Res<TidefallResources>,
    bodies: Bodies<'_, '_>,
    mut ships: Pilots<'_, '_>,
) {
    let Some(ship) = resources.ship else {
        return;
    };
    let aboard = resources.mode == Mode::Aboard;
    let delta_time = time.delta_time.min(MAX_STEP);
    let views = collect_bodies(bodies);
    let thrust = config.number(Settings::Thrust);
    let turn_rate = AUTOPILOT_TURN * config.number(Settings::Autopilot);
    let step = turn_rate * delta_time;
    let manual =
        aboard && (actions.axis2(Ask::Move).magnitude() > 0.0 || actions.axis(Ask::Lift) != 0.0);
    let lock_pressed = aboard && actions.just_pressed(Ask::Lock);
    let engage_pressed = aboard && actions.just_pressed(Ask::Engage);

    ships.for_entity(
        ship,
        |(velocity, footing, ground, locked, autopilot, push, transform)| {
            let ahead =
                nalgebra_glm::quat_rotate_vec3(&transform.rotation, &Vec3::new(0.0, 0.0, -1.0));

            let sighted = views
                .iter()
                .map(|view| {
                    let toward = (view.center - transform.translation).normalize();
                    (view.entity, ahead.dot(&toward))
                })
                .max_by(|(_, first), (_, second)| first.total_cmp(second))
                .map(|(entity, _)| entity);
            if lock_pressed {
                locked.0 = if sighted == locked.0 { None } else { sighted };
            }

            if engage_pressed {
                if *autopilot == Autopilot::Off {
                    if locked.0.is_none() {
                        locked.0 = sighted;
                    }
                    if locked.0.is_some() {
                        if *footing == Footing::Landed
                            && let Some(view) = ground
                                .0
                                .and_then(|body| views.iter().find(|view| view.entity == body))
                            && let Some(next) = footing.process_event(FootingEvent::Burn)
                        {
                            *footing = next;
                            let outward = (transform.translation - view.center).normalize();
                            velocity.0 = view.motion + outward * LIFT_OFF;
                        }
                        if *footing == Footing::Flying
                            && let Some(next) = autopilot.process_event(PilotEvent::Engage)
                        {
                            *autopilot = next;
                        }
                    }
                } else if let Some(next) = autopilot.process_event(PilotEvent::Cancel) {
                    *autopilot = next;
                }
            }

            let target = locked
                .0
                .and_then(|entity| views.iter().find(|view| view.entity == entity));
            if (manual || *footing != Footing::Flying || target.is_none())
                && let Some(next) = autopilot.process_event(PilotEvent::Cancel)
            {
                *autopilot = next;
            }
            push.0 = Vec3::zeros();
            let Some(target) = target else {
                return;
            };

            let relative = transform.translation - target.center;
            let distance = relative.magnitude().max(1.0e-3);
            let outward = relative / distance;
            let toward = -outward;
            let drift = velocity.0 - target.motion;
            let speed = drift.magnitude();

            match *autopilot {
                Autopilot::Off => {}
                Autopilot::Align => {
                    let left = turn_toward(&mut transform.rotation, ahead, toward, step);
                    if let Some(index) = nearest_body(transform.translation, &views)
                        && altitude(transform.translation, &views[index]) < views[index].radius
                    {
                        push.0 = (transform.translation - views[index].center).normalize();
                    }
                    if left <= ALIGNED_ANGLE
                        && let Some(next) = autopilot.process_event(PilotEvent::Aligned)
                    {
                        *autopilot = next;
                    }
                }
                Autopilot::Burn => {
                    turn_toward(&mut transform.rotation, ahead, toward, step);
                    push.0 = toward;
                    let remaining = distance - target.radius * ARRIVAL_FACTOR;
                    let pull = target.pull / (target.radius * ARRIVAL_FACTOR);
                    let decel = (thrust * BRAKE_MARGIN - pull).max(thrust * 0.25);
                    let stop = speed * speed / (2.0 * decel);
                    if remaining <= stop
                        && let Some(next) = autopilot.process_event(PilotEvent::Turn)
                    {
                        *autopilot = next;
                    }
                }
                Autopilot::Brake => {
                    if speed <= ARRIVE_SPEED {
                        if let Some(next) = autopilot.process_event(PilotEvent::Arrive) {
                            *autopilot = next;
                        }
                        return;
                    }
                    turn_toward(&mut transform.rotation, ahead, toward, step);
                    let frame = (thrust * delta_time).max(1.0e-3);
                    push.0 = -drift / speed * (speed / frame).min(1.0);
                }
                Autopilot::Descend => {
                    let radial = drift.dot(&outward);
                    let lateral = drift - outward * radial;
                    let height = distance - target.radius;
                    let wanted = -(height * DESCENT_GAIN).clamp(DESCENT_MIN, DESCENT_MAX);
                    let pull = target.pull / distance.max(target.radius);
                    let wish = outward * ((wanted - radial) * DESCENT_RESPONSE + pull)
                        - lateral * DESCENT_RESPONSE;
                    let asked = wish / thrust;
                    let magnitude = asked.magnitude();
                    push.0 = if magnitude > 1.0 {
                        asked / magnitude
                    } else {
                        asked
                    };
                    let up = nalgebra_glm::quat_rotate_vec3(&transform.rotation, &Vec3::y());
                    turn_toward(&mut transform.rotation, up, outward, step);
                }
            }
        },
    );
}

fn turn_toward(rotation: &mut Quat, current: Vec3, desired: Vec3, step: f32) -> f32 {
    let angle = current.dot(&desired).clamp(-1.0, 1.0).acos();
    if angle < 1.0e-4 {
        return angle;
    }
    let cross = current.cross(&desired);
    let axis = if cross.magnitude() < 1.0e-4 {
        nalgebra_glm::quat_rotate_vec3(rotation, &Vec3::y())
    } else {
        cross.normalize()
    };
    let amount = angle.min(step);
    *rotation =
        nalgebra_glm::quat_normalize(&(nalgebra_glm::quat_angle_axis(amount, &axis) * *rotation));
    angle - amount
}
