use crate::components::{Aim, Hook, LineLength, Speed};
use crate::contract::TetherEvent;
use crate::data::{Ask, EYE_OFFSET, REACH, SHORTEST_LINE};
use crate::queries::in_control;
use crate::resources::TetherResources;
use nightshade::prelude::*;
use nightshade_input::Actions;
use nightshade_physics::queries::physics_world_cast_ray;
use nightshade_physics::resources::PhysicsWorld;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::LocalTransform;

pub fn aim_the_line(
    physics: Res<PhysicsWorld>,
    input: Res<Input>,
    window: Res<Window>,
    resources: Res<TetherResources>,
    mut commands: Commands,
    mut aimers: Query<(&LocalTransform, Option<&mut Aim>)>,
) {
    let Some(runner) = resources.scene.runner else {
        return;
    };
    let Some(camera) = resources.scene.camera else {
        return;
    };

    let mut forward = None;
    aimers.for_entity(camera, |(transform, _)| {
        forward = Some(nalgebra_glm::quat_rotate_vec3(
            &transform.rotation,
            &Vec3::new(0.0, 0.0, -1.0),
        ));
    });

    let mut body = None;
    let mut had_aim = false;
    aimers.for_entity(runner, |(transform, aim)| {
        body = Some(transform.translation);
        had_aim = aim.is_some();
    });

    let reached = match (in_control(&window, &input), forward, body) {
        (true, Some(forward), Some(body)) => {
            let eye = body + Vec3::new(0.0, EYE_OFFSET, 0.0);
            physics_world_cast_ray(&physics, eye, forward, REACH, Some(runner)).map(|hit| hit.point)
        }
        _ => None,
    };

    match (reached, had_aim) {
        (Some(point), true) => aimers.for_entity(runner, |(_, aim)| {
            if let Some(aim) = aim {
                aim.0 = point;
            }
        }),
        (Some(point), false) => commands.set(runner, Aim(point)),
        (None, true) => commands.remove::<(Aim,)>(runner),
        (None, false) => {}
    }
}

pub fn throw_the_line(
    input: Res<Input>,
    actions: Res<Actions<Ask>>,
    window: Res<Window>,
    resources: Res<TetherResources>,
    mut events: EventWriter<TetherEvent>,
    mut commands: Commands,
    mut runners: Query<(&LocalTransform, &Speed, Option<&Hook>, Option<&Aim>)>,
) {
    let Some(runner) = resources.scene.runner else {
        return;
    };
    let held = in_control(&window, &input) && actions.pressed(Ask::Throw);

    let mut body = Vec3::zeros();
    let mut speed = 0.0;
    let mut attached = false;
    let mut aim = None;
    runners.for_entity(runner, |(transform, runner_speed, anchor, runner_aim)| {
        body = transform.translation;
        speed = runner_speed.0;
        attached = anchor.is_some();
        aim = runner_aim.map(|aim| aim.0);
    });

    if held && !attached {
        let Some(point) = aim else {
            return;
        };
        commands.set(runner, Hook(point));
        commands.set(runner, LineLength((point - body).norm().max(SHORTEST_LINE)));
        return;
    }

    if !held && attached {
        commands.remove::<(Hook, LineLength)>(runner);
        events.send(TetherEvent::Released { speed });
    }
}
