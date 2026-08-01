use crate::commands::post_message;
use crate::components::{Alive, Armed, Health, Heft, HuskState, Last, Life, NpcIndex, Pace};
use crate::data::ask::Ask;
use crate::data::rules::*;
use crate::queries::hold_push;
use crate::resources::{AshfallResources, Impact, Screen, Senses};
use crate::systems::{effects, husks};
use crate::tags::{Eyes, Fragile, Grabbable};
use nightshade::prelude::*;
use nightshade_input::Actions;
use nightshade_physics::commands::{
    physics_world_set_angular_damping, physics_world_set_angular_velocity,
    physics_world_set_linear_velocity,
};
use nightshade_physics::queries::physics_world_cast_ray;
use nightshade_physics::resources::PhysicsWorld;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::*;
use nightshade_senses::Noises;
use nightshade_senses::commands::make_a_noise;

pub fn update(
    input: Res<Input>,
    actions: Res<Actions<Ask>>,
    mut physics: ResMut<PhysicsWorld>,
    mut resources: ResMut<AshfallResources>,
    mut noises: ResMut<Noises>,
    mut commands: Commands,
    mut eyes: Query<(&GlobalTransform,), WithTag<Eyes>>,
    mut loose: Query<(&LocalTransform,), WithTag<Grabbable>>,
) {
    let resources = &mut *resources;
    if resources.screen != Screen::Playing {
        return;
    }
    let Some(body) = resources.player.body else {
        return;
    };
    let mut view = None;
    if let Some(eye_entity) = resources.player.eyes {
        eyes.for_entity(eye_entity, |(global,)| {
            view = Some((global.translation(), global.forward_vector()));
        });
    }
    let Some((eye, forward)) = view else {
        return;
    };
    let forward = nalgebra_glm::normalize(&forward);

    let scroll = hold_push(&input, &actions);
    if scroll.abs() > 0.01 {
        resources.player.hold_distance =
            (resources.player.hold_distance + scroll * GRAB_STEP).clamp(GRAB_NEAR, GRAB_FAR);
    }

    let reaching = actions.pressed(Ask::Reach);
    if !reaching {
        if let Some(held) = resources.player.holding.take() {
            physics_world_set_angular_damping(&mut physics, held, LOOSE_SPIN_DAMPING);
        }
        resources.player.hold_latched = false;
        return;
    }

    if resources.player.holding.is_none() {
        if resources.player.hold_latched {
            return;
        }
        let Some(hit) = physics_world_cast_ray(&physics, eye, forward, GRAB_REACH, Some(body))
        else {
            return;
        };
        let mut takeable = false;
        loose.for_entity(hit.entity, |_| takeable = true);
        if !takeable {
            return;
        }
        resources.player.holding = Some(hit.entity);
        resources.player.hold_distance = hit.distance.clamp(GRAB_NEAR, GRAB_FAR);
        physics_world_set_angular_damping(&mut physics, hit.entity, GRAB_SPIN_DAMPING);
        commands.remove::<(Life, Armed, Last, Pace)>(hit.entity);
        return;
    }

    let Some(held) = resources.player.holding else {
        return;
    };

    if actions.just_pressed(Ask::Throw) {
        resources.player.holding = None;
        resources.player.hold_latched = true;
        physics_world_set_angular_damping(&mut physics, held, LOOSE_SPIN_DAMPING);
        physics_world_set_linear_velocity(
            &mut physics,
            held,
            forward * THROW_SPEED + Vec3::new(0.0, THROW_LIFT, 0.0),
        );
        commands.set(held, Life(THROW_LIFE));
        commands.set(held, Armed(THROW_ARMING));
        commands.set(held, Last(eye));
        commands.set(held, Pace(THROW_SPEED));
        make_a_noise(&mut noises, eye, NOISE_SWING);
        return;
    }

    let target = eye + forward * resources.player.hold_distance;
    let mut here = None;
    loose.for_entity(held, |(transform,)| here = Some(transform.translation));
    let Some(here) = here else {
        resources.player.holding = None;
        return;
    };
    let toward = target - here;
    let velocity = if toward.magnitude() > 0.001 {
        let speed = (toward.magnitude() * GRAB_STIFFNESS).min(GRAB_MAX_SPEED);
        nalgebra_glm::normalize(&toward) * speed
    } else {
        Vec3::zeros()
    };
    physics_world_set_linear_velocity(&mut physics, held, velocity);
    physics_world_set_angular_velocity(&mut physics, held, Vec3::zeros());
}

pub fn land(
    time: Res<Time>,
    mut resources: ResMut<AshfallResources>,
    mut senses: ResMut<Senses>,
    mut noises: ResMut<Noises>,
    mut commands: Commands,
    flights: Query<(&mut Life, &mut Armed, &mut Last, &mut Pace, &LocalTransform)>,
    mut fragile: Query<(&LocalTransform,), WithTag<Fragile>>,
    mut heft: Query<(&Heft,)>,
    mut husks: husks::Wounded<'_, '_>,
    mut people: Query<(&NpcIndex, &mut Health, &mut Alive, &LocalTransform)>,
) {
    let resources = &mut *resources;
    if resources.screen != Screen::Playing {
        return;
    }
    let delta = time.delta_time.max(0.0001);
    let mut flying = Vec::new();
    flights.for_each(|entity, (life, armed, last, flight_pace, transform)| {
        let here = transform.translation;
        life.0 -= delta;
        armed.0 = (armed.0 - delta).max(0.0);
        let pace = (here - last.0).magnitude() / delta;
        last.0 = here;
        let carried = flight_pace.0.max(pace);
        flight_pace.0 = pace;
        if armed.0 > 0.0 {
            return;
        }
        let resting = pace < LANDING_PACE || life.0 <= 0.0;
        flying.push((entity, here, carried, resting));
    });
    for (entity, at, pace, resting) in flying {
        let mut mass = 1.0;
        heft.for_entity(entity, |(weight,)| mass = weight.mass);
        let force = mass * pace;
        let struck = if force >= THROWN_MINIMUM {
            strike_a_body(
                resources,
                &mut senses,
                &mut noises,
                &mut husks,
                &mut people,
                at,
                (force * THROWN_DAMAGE).min(THROWN_CAP),
            )
        } else {
            false
        };
        if !struck && !resting {
            continue;
        }
        let mut breaks = false;
        fragile.for_entity(entity, |_| breaks = true);
        if breaks {
            commands.despawn_recursive(entity);
            effects::shatter(&mut commands, at);
            make_a_noise(&mut noises, at, NOISE_GLASS);
        } else {
            commands.remove::<(Life, Armed, Last, Pace)>(entity);
            make_a_noise(&mut noises, at, NOISE_IMPACT);
        }
        senses.impacts.push(Impact { at, force });
    }
}

fn strike_a_body(
    resources: &mut AshfallResources,
    senses: &mut Senses,
    noises: &mut Noises,
    husks: &mut husks::Wounded<'_, '_>,
    people: &mut Query<(&NpcIndex, &mut Health, &mut Alive, &LocalTransform)>,
    at: Vec3,
    damage: f32,
) -> bool {
    let mut husk_hit: Option<(Entity, f32)> = None;
    husks.for_each_mut(|entity, (state, _, _, _, transform)| {
        if *state == HuskState::Dead {
            return;
        }
        let reach = (transform.translation - at).magnitude();
        if reach <= THROWN_REACH && husk_hit.is_none_or(|(_, best)| reach < best) {
            husk_hit = Some((entity, reach));
        }
    });
    let mut person_hit: Option<(Entity, f32)> = None;
    people.for_each_mut(|entity, (_, _, alive, transform)| {
        if !alive.0 {
            return;
        }
        let reach = (transform.translation - at).magnitude();
        if reach <= THROWN_REACH && person_hit.is_none_or(|(_, best)| reach < best) {
            person_hit = Some((entity, reach));
        }
    });
    let husk_reach = husk_hit.map(|(_, reach)| reach).unwrap_or(f32::MAX);
    let person_reach = person_hit.map(|(_, reach)| reach).unwrap_or(f32::MAX);
    if husk_reach <= person_reach
        && let Some((entity, _)) = husk_hit
    {
        let mut killed = false;
        husks.for_entity(entity, |(state, health, timer, moving, _)| {
            killed = husks::wound(state, health, timer, moving, damage)
        });
        if killed {
            resources.run.kills += 1;
        }
        post_message(
            &mut resources.run,
            "It lands hard enough to turn the thing around.",
        );
        return true;
    }
    let Some((entity, _)) = person_hit else {
        return false;
    };
    let mut fallen = None;
    people.for_entity(entity, |(_, health, alive, transform)| {
        health.0 -= damage;
        if health.0 <= 0.0 {
            alive.0 = false;
            fallen = Some(transform.translation);
        }
    });
    if let Some(where_they_fell) = fallen {
        make_a_noise(noises, where_they_fell, PERSON_DEATH_NOISE);
        senses.stains.push(where_they_fell);
        post_message(
            &mut resources.run,
            "They go down and stay down. Everything in the fog heard that.",
        );
    } else {
        post_message(&mut resources.run, "It hits them. They shout at you.");
    }
    true
}
