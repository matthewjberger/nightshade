use crate::components::{
    At, Crouch, Facing, Grounded, JumpHold, Steering, Velocity, Wants, WasAt, WasFacing,
};
use crate::data::Ask;
use crate::data::wire::{WANT_CROUCH, WANT_JUMP};
use crate::queries::step::{Motion, Wish, step_character};
use crate::resources::Session;
use crate::theme;
use nalgebra_glm::Vec3;
use nightshade::prelude::*;
use nightshade_camera::prelude::ActiveCamera;
use nightshade_input::Actions;
use nightshade_physics::prelude::PhysicsWorld;
use nightshade_platform::prelude::Time;
use nightshade_scene::prelude::LocalTransform;

type Driven<'a> = (
    &'a Steering,
    &'a mut Wants,
    &'a mut At,
    &'a mut Velocity,
    &'a mut Facing,
    &'a mut Crouch,
    &'a mut Grounded,
    &'a mut JumpHold,
);

pub fn advance_clock(time: Res<Time>, mut session: ResMut<Session>) {
    session.clock += time.delta_time;
    session.accumulator += time.delta_time;
    session.stepping = session.accumulator >= theme::TICK_SECONDS;
    if session.stepping {
        session.accumulator -= theme::TICK_SECONDS;
        if session.accumulator > theme::TICK_SECONDS {
            session.accumulator = 0.0;
        }
    }
}

pub fn latch_input(
    actions: Res<Actions<Ask>>,
    active: Res<ActiveCamera>,
    arguments: Res<crate::resources::Arguments>,
    mut session: ResMut<Session>,
    mut transforms: Query<(&LocalTransform,)>,
    mut steering: Query<(&mut Steering, &mut Wants)>,
) {
    let Some(body) = session.local else {
        return;
    };
    if session.typing || session.screen != crate::resources::Screen::Playing {
        steering.for_entity(body, |(walk, _wants)| walk.0 = Vec3::zeros());
        return;
    }

    let eye = active.0.and_then(|camera| {
        let mut view = None;
        transforms.for_entity(camera, |(transform,)| {
            view = Some((
                transform.translation,
                transform.forward_vector(),
                transform.right_vector(),
            ));
        });
        view
    });
    if let Some((origin, forward, _)) = eye {
        session.aim_origin = origin;
        session.aim_direction = forward;
    }
    if actions.just_pressed(Ask::Fire) {
        session.fired = true;
    }
    let (forward, right) = match eye {
        Some((_, forward, right)) => (flatten(forward, -Vec3::z()), flatten(right, Vec3::x())),
        None => (-Vec3::z(), Vec3::x()),
    };

    let walk = actions.axis2(Ask::Walk);
    let mut direction = forward * walk.y + right * walk.x;

    if arguments.wander {
        let turn = session.clock * theme::WANDER_TURN;
        direction = Vec3::new(turn.sin(), 0.0, turn.cos());
    }

    let jump = actions.just_pressed(Ask::Jump);
    let crouch = actions.pressed(Ask::Crouch);

    steering.for_entity(body, |(walk, wants)| {
        walk.0 = direction;
        if jump {
            wants.0 |= WANT_JUMP;
        }
        if crouch {
            wants.0 |= WANT_CROUCH;
        } else {
            wants.0 &= !WANT_CROUCH;
        }
    });
}

fn flatten(vector: Vec3, fallback: Vec3) -> Vec3 {
    let flat = Vec3::new(vector.x, 0.0, vector.z);
    if flat.norm() > 1e-4 {
        flat.normalize()
    } else {
        fallback
    }
}

pub fn remember(session: Res<Session>, marks: Query<(&At, &Facing, &mut WasAt, &mut WasFacing)>) {
    if !session.stepping {
        return;
    }
    marks.for_each(|_entity, (at, facing, was_at, was_facing)| {
        was_at.0 = at.0;
        was_facing.0 = facing.0;
    });
}

pub fn tick(session: Res<Session>, physics: Res<PhysicsWorld>, bodies: Query<Driven>) {
    if !session.stepping {
        return;
    }
    bodies.for_each(
        |_entity, (steering, wants, at, velocity, facing, crouch, grounded, hold)| {
            let motion = Motion {
                position: at.0,
                velocity: velocity.0,
                facing: facing.0,
                crouch: crouch.0,
                grounded: grounded.0,
                jump_hold: hold.0,
            };
            let wish = Wish {
                walk: steering.0,
                wants: wants.0,
            };
            let next = step_character(motion, wish, theme::TICK_SECONDS, &physics);
            at.0 = next.position;
            velocity.0 = next.velocity;
            facing.0 = next.facing;
            crouch.0 = next.crouch;
            grounded.0 = next.grounded;
            hold.0 = next.jump_hold;
            wants.0 &= !crate::data::wire::WANT_JUMP;
        },
    );
}

pub fn render(
    session: Res<Session>,
    bodies: Query<(&At, &WasAt, &Facing, &WasFacing, &mut LocalTransform)>,
) {
    let blend = (session.accumulator / theme::TICK_SECONDS).clamp(0.0, 1.0);
    bodies.for_each(|_entity, (at, was_at, facing, was_facing, transform)| {
        transform.translation = was_at.0 + (at.0 - was_at.0) * blend;
        let turn = crate::systems::net::turn_toward(was_facing.0, facing.0, blend);
        transform.rotation = nalgebra_glm::quat_angle_axis(turn, &Vec3::y());
    });
}
