use crate::components::{Autopilot, Footing, Ground, Heading, Offset, Push, Velocity};
use crate::contract::TidefallEvent;
use crate::data::footing::Event as FootingEvent;
use crate::data::{
    Ask, COCKPIT, FRICTION, GAMEPAD_LOOK, LEAN_RATE, LIFT_OFF, LOOK_RADIANS_PER_PIXEL, MAX_STEP,
    REST_SPEED, SHIP_RADIUS, Settings,
};
use crate::queries::{
    Bodies, Drafts, Features, collect_bodies, collect_colliders, collect_drafts, draft_at,
    gravity_at, nearest_body,
};
use crate::resources::{Mode, TidefallResources};
use nightshade::prelude::*;
use nightshade_config::prelude::Config;
use nightshade_input::Actions;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::*;

pub fn fly(
    time: Res<Time>,
    input: Res<Input>,
    window: Res<Window>,
    actions: Res<Actions<Ask>>,
    config: Res<Config<Settings>>,
    mut resources: ResMut<TidefallResources>,
    mut events: EventWriter<TidefallEvent>,
    bodies: Bodies<'_, '_>,
    features: Features<'_, '_>,
    drafts: Drafts<'_, '_>,
    mut ships: Query<(
        &mut Velocity,
        &mut Footing,
        &mut Ground,
        &Autopilot,
        &Push,
        &mut LocalTransform,
    )>,
    mut berths: Query<(&mut Offset, &mut Heading)>,
    mut cameras: Query<(&mut LocalTransform,)>,
) {
    let (Some(ship), Some(walker)) = (resources.ship, resources.walker) else {
        return;
    };
    let delta_time = time.delta_time.min(MAX_STEP);
    if delta_time <= f32::EPSILON {
        return;
    }
    let aboard = resources.mode == Mode::Aboard;
    let views = collect_bodies(bodies);
    let colliders = collect_colliders(features, &views);
    let updrafts = collect_drafts(drafts, &views);

    let mouse = if aboard && window.cursor_locked {
        input.mouse.raw_mouse_delta * LOOK_RADIANS_PER_PIXEL * config.number(Settings::Look)
    } else {
        Vec2::zeros()
    };
    let stick = if aboard {
        actions.axis2(Ask::Look) * GAMEPAD_LOOK * delta_time
    } else {
        Vec2::zeros()
    };
    let looked = mouse + Vec2::new(stick.x, -stick.y);
    let (turn, roll) = if aboard && actions.pressed(Ask::Roll) {
        (
            Vec2::new(0.0, looked.y),
            -looked.x * config.number(Settings::Roll),
        )
    } else {
        (looked, 0.0)
    };
    let slide = if aboard {
        actions.axis2(Ask::Move)
    } else {
        Vec2::zeros()
    };
    let lift = if aboard { actions.axis(Ask::Lift) } else { 0.0 };
    let thrust = config.number(Settings::Thrust);
    let idle = slide.magnitude() == 0.0 && lift == 0.0;
    let assist = if aboard && actions.pressed(Ask::Match) {
        thrust
    } else if idle {
        thrust * config.number(Settings::Assist)
    } else {
        0.0
    };
    let gravity_scale = config.number(Settings::Gravity);

    let mut berth = None;
    berths.for_entity(ship, |(offset, heading)| {
        berth = Some((offset.0, heading.0));
    });
    let Some((mut berth_offset, mut berth_heading)) = berth else {
        return;
    };

    let mut landed_on = None;
    let mut cockpit = None;
    ships.for_entity(
        ship,
        |(velocity, footing, ground, autopilot, push, transform)| {
            let nearest = nearest_body(transform.translation, &views);

            match *footing {
                Footing::Landed => {
                    let Some(view) = ground
                        .0
                        .and_then(|body| views.iter().find(|view| view.entity == body))
                    else {
                        *footing = Footing::Flying;
                        return;
                    };
                    transform.translation =
                        view.center + nalgebra_glm::quat_rotate_vec3(&view.rotation, &berth_offset);
                    transform.rotation =
                        nalgebra_glm::quat_normalize(&(view.rotation * berth_heading));
                    let outward =
                        nalgebra_glm::quat_rotate_vec3(&view.rotation, &berth_offset.normalize());
                    velocity.0 = view.motion;
                    if lift > 0.0
                        && let Some(next) = footing.process_event(FootingEvent::Burn)
                    {
                        *footing = next;
                        velocity.0 += outward * (LIFT_OFF + thrust * lift * delta_time);
                        events.send(TidefallEvent::Lifted {
                            body: view.name.clone(),
                        });
                    }
                }
                Footing::Flying => {
                    let piloted = *autopilot != Autopilot::Off;
                    if !piloted {
                        let yaw = nalgebra_glm::quat_angle_axis(-turn.x, &Vec3::y());
                        let pitch = nalgebra_glm::quat_angle_axis(-turn.y, &Vec3::x());
                        let bank = nalgebra_glm::quat_angle_axis(roll, &Vec3::z());
                        transform.rotation = nalgebra_glm::quat_normalize(
                            &(transform.rotation * yaw * pitch * bank),
                        );
                    }
                    let ahead = nalgebra_glm::quat_rotate_vec3(
                        &transform.rotation,
                        &Vec3::new(0.0, 0.0, -1.0),
                    );
                    let right = nalgebra_glm::quat_rotate_vec3(&transform.rotation, &Vec3::x());
                    let up = nalgebra_glm::quat_rotate_vec3(&transform.rotation, &Vec3::y());
                    let burn = (ahead * slide.y + right * slide.x + up * lift + push.0) * thrust;

                    let draft = nearest
                        .map(|index| {
                            draft_at(transform.translation, views[index].entity, &updrafts)
                        })
                        .unwrap_or_else(Vec3::zeros);
                    let acceleration =
                        burn + gravity_at(transform.translation, &views) * gravity_scale + draft;
                    velocity.0 += acceleration * delta_time;
                    if let Some(index) = nearest
                        && !piloted
                        && assist > 0.0
                    {
                        let view = &views[index];
                        let relative = velocity.0 - view.motion;
                        let brake = assist * delta_time;
                        velocity.0 = if relative.magnitude() <= brake {
                            view.motion
                        } else {
                            velocity.0 - relative.normalize() * brake
                        };
                    }
                    transform.translation += velocity.0 * delta_time;

                    if let Some(index) = nearest {
                        let view = &views[index];
                        let mut normals = Vec3::zeros();
                        let mut drift = velocity.0 - view.motion;
                        let radial = transform.translation - view.center;
                        let distance = radial.magnitude();
                        if distance < view.radius + SHIP_RADIUS {
                            let normal = radial / distance;
                            transform.translation =
                                view.center + normal * (view.radius + SHIP_RADIUS);
                            drift = press(drift, normal, delta_time * view.grip);
                            normals += normal;
                        }
                        for collider in colliders
                            .iter()
                            .filter(|collider| collider.body == view.entity)
                        {
                            let gap = transform.translation - collider.center;
                            let distance = gap.magnitude();
                            if distance < collider.radius + SHIP_RADIUS && distance > f32::EPSILON {
                                let normal = gap / distance;
                                transform.translation =
                                    collider.center + normal * (collider.radius + SHIP_RADIUS);
                                drift = press(drift, normal, delta_time * view.grip);
                                normals += normal;
                            }
                        }
                        if normals.magnitude() > f32::EPSILON {
                            velocity.0 = view.motion + drift;
                            let normal = normals.normalize();
                            let current_up =
                                nalgebra_glm::quat_rotate_vec3(&transform.rotation, &Vec3::y());
                            let upright = nalgebra_glm::quat_normalize(
                                &(nalgebra_glm::quat_rotation(&current_up, &normal)
                                    * transform.rotation),
                            );
                            transform.rotation =
                                nalgebra_glm::quat_normalize(&nalgebra_glm::quat_slerp(
                                    &transform.rotation,
                                    &upright,
                                    (LEAN_RATE * delta_time).min(1.0),
                                ));
                            if drift.magnitude() < REST_SPEED
                                && let Some(next) = footing.process_event(FootingEvent::Touch)
                            {
                                *footing = next;
                                velocity.0 = view.motion;
                                ground.0 = Some(view.entity);
                                landed_on = Some(index);
                                let unturn = nalgebra_glm::quat_conjugate(&view.rotation);
                                berth_offset = nalgebra_glm::quat_rotate_vec3(
                                    &unturn,
                                    &(transform.translation - view.center),
                                );
                                berth_heading =
                                    nalgebra_glm::quat_normalize(&(unturn * transform.rotation));
                                events.send(TidefallEvent::Landed {
                                    body: view.name.clone(),
                                });
                            }
                        }
                    }
                }
            }
            cockpit = Some((transform.translation, transform.rotation));
        },
    );

    berths.for_entity(ship, |(offset, heading)| {
        offset.0 = berth_offset;
        heading.0 = berth_heading;
    });

    if aboard && let Some((translation, rotation)) = cockpit {
        cameras.for_entity(walker, |(transform,)| {
            transform.translation =
                translation + nalgebra_glm::quat_rotate_vec3(&rotation, &COCKPIT);
            transform.rotation = rotation;
        });
    }

    if let Some(index) = landed_on {
        resources.visited.insert(views[index].entity);
    }
}

fn press(drift: Vec3, normal: Vec3, delta_time: f32) -> Vec3 {
    let into = drift.dot(&normal);
    let mut pressed = drift;
    if into < 0.0 {
        pressed -= normal * into;
    }
    let along = pressed - normal * pressed.dot(&normal);
    pressed - along * (FRICTION * delta_time).min(1.0)
}
