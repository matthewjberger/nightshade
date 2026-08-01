use crate::commands::post_message;
use crate::components::{Handle, Heft, Item, Locked, Readable};
use crate::contract::{Fixture, ItemKind, item_name};
use crate::data::ask::Ask;
use crate::data::rooms::DOORWAYS;
use crate::data::rules::*;
use crate::queries::hold_push;
use crate::resources::{BlackmarrowResources, Console, Screen};
use crate::systems::station::hang_leaf;
use nightshade::prelude::*;
use nightshade_input::Actions;
use nightshade_physics::commands::{
    physics_world_apply_impulse_at_point, physics_world_set_angular_damping,
    physics_world_set_linear_velocity,
};
use nightshade_physics::queries::{
    physics_world_body_rotation, physics_world_body_translation, physics_world_cast_ray,
    physics_world_linear_velocity,
};
use nightshade_physics::resources::PhysicsWorld;
use nightshade_platform::prelude::*;
use nightshade_senses::Noises;
use nightshade_senses::commands::make_a_noise;
use nightshade_shell::{ShellState, shell_is_capturing_input};

const GRAVITY: f32 = 9.81;

pub type Handles<'world, 'item> = Query<'world, (&'item Handle, &'item Heft)>;
pub type Items<'world, 'item> = Query<'world, (&'item Item,)>;
pub type Locks<'world, 'item> = Query<'world, (&'item Locked,)>;
pub type Readables<'world, 'item> = Query<'world, (&'item Readable,)>;

nightshade::ecs::query_bundle! {
    pub struct Reach<'world, 'item> {
        pub handles: Handles<'world, 'item>,
        pub items: Items<'world, 'item>,
        pub locks: Locks<'world, 'item>,
        pub readables: Readables<'world, 'item>,
    }
}

pub fn update(
    time: Res<Time>,
    input: Res<Input>,
    actions: Res<Actions<Ask>>,
    mut physics: ResMut<PhysicsWorld>,
    mut resources: ResMut<BlackmarrowResources>,
    mut noises: ResMut<Noises>,
    shell: Res<ShellState<Console>>,
    mut commands: Commands,
    mut reach: Reach<'_, '_>,
) {
    let resources = &mut *resources;
    if resources.screen != Screen::Playing || shell_is_capturing_input(&shell) {
        resources.run.prompt.clear();
        return;
    }
    let Some(body) = resources.player.body else {
        return;
    };
    let eye = resources.player.eye;
    let forward = resources.player.forward;
    let delta = time.delta_time.max(0.0001);

    let push = hold_push(&input);
    if push.abs() > 0.01 {
        resources.player.hold_distance =
            (resources.player.hold_distance + push * HOLD_STEP).clamp(HOLD_NEAR, HOLD_FAR);
    }

    if resources.player.holding.is_none() {
        offer(resources, &physics, &mut reach, body);
        if actions.just_pressed(Ask::Pocket) {
            take_or_unlock(resources, &physics, &mut commands, &mut reach, body);
        }
    }

    if !actions.pressed(Ask::Reach) {
        if let Some(held) = resources.player.holding.take() {
            physics_world_set_angular_damping(&mut physics, held, LOOSE_SPIN_DAMPING);
            let mut swung = false;
            reach
                .handles
                .for_entity(held, |(handle, _)| swung = handle.0 != Fixture::Loose);
            if swung
                && let Some(at) = physics_world_body_translation(&physics, held)
                && physics_world_linear_velocity(&physics, held)
                    .unwrap_or_else(Vec3::zeros)
                    .magnitude()
                    > 1.2
            {
                make_a_noise(&mut noises, at, NOISE_DOOR);
            }
        }
        resources.player.hold_latched = false;
        return;
    }

    if resources.player.holding.is_none() {
        if resources.player.hold_latched {
            return;
        }
        let Some(hit) = physics_world_cast_ray(&physics, eye, forward, REACH, Some(body)) else {
            return;
        };
        let mut takeable = false;
        reach.handles.for_entity(hit.entity, |_| takeable = true);
        if !takeable {
            return;
        }
        let Some(origin) = physics_world_body_translation(&physics, hit.entity) else {
            return;
        };
        let rotation = physics_world_body_rotation(&physics, hit.entity)
            .unwrap_or_else(nalgebra_glm::quat_identity);
        let world_offset = hit.point - origin;
        resources.player.hold_offset =
            nalgebra_glm::quat_rotate_vec3(&nalgebra_glm::quat_conjugate(&rotation), &world_offset);
        resources.player.hold_distance = hit.distance.clamp(HOLD_NEAR, HOLD_FAR);
        resources.player.holding = Some(hit.entity);
        physics_world_set_angular_damping(&mut physics, hit.entity, HELD_SPIN_DAMPING);
        return;
    }

    let Some(held) = resources.player.holding else {
        return;
    };
    let mut fixture = None;
    let mut mass = 1.0;
    reach.handles.for_entity(held, |(handle, heft)| {
        fixture = Some(handle.0);
        mass = heft.0.max(0.05);
    });
    let Some(fixture) = fixture else {
        resources.player.holding = None;
        return;
    };

    if fixture == Fixture::Loose && actions.just_pressed(Ask::Throw) {
        resources.player.holding = None;
        resources.player.hold_latched = true;
        physics_world_set_angular_damping(&mut physics, held, LOOSE_SPIN_DAMPING);
        physics_world_set_linear_velocity(
            &mut physics,
            held,
            forward * THROW_SPEED + Vec3::new(0.0, THROW_LIFT, 0.0),
        );
        make_a_noise(&mut noises, eye, NOISE_THROW);
        return;
    }

    let Some(origin) = physics_world_body_translation(&physics, held) else {
        resources.player.holding = None;
        return;
    };
    let rotation =
        physics_world_body_rotation(&physics, held).unwrap_or_else(nalgebra_glm::quat_identity);
    let grip = origin + nalgebra_glm::quat_rotate_vec3(&rotation, &resources.player.hold_offset);
    let target = eye + forward * resources.player.hold_distance;
    let velocity = physics_world_linear_velocity(&physics, held).unwrap_or_else(Vec3::zeros);

    let stiffness = if fixture == Fixture::Loose {
        PULL_STIFFNESS
    } else {
        PULL_STIFFNESS * FIXTURE_SCALE
    };
    let mut pull = (target - grip) * stiffness - velocity * PULL_DAMPING;
    if pull.magnitude() > PULL_CEILING {
        pull = nalgebra_glm::normalize(&pull) * PULL_CEILING;
    }
    if fixture == Fixture::Loose {
        pull.y += GRAVITY;
    }
    physics_world_apply_impulse_at_point(&mut physics, held, pull * mass * delta, grip);
}

fn offer(
    resources: &mut BlackmarrowResources,
    physics: &PhysicsWorld,
    reach: &mut Reach<'_, '_>,
    body: Entity,
) {
    resources.run.prompt.clear();
    let eye = resources.player.eye;
    let forward = resources.player.forward;
    let Some(hit) = physics_world_cast_ray(physics, eye, forward, REACH, Some(body)) else {
        return;
    };
    let mut item = None;
    reach
        .items
        .for_entity(hit.entity, |(carried,)| item = Some(carried.0));
    let mut lock = None;
    reach
        .locks
        .for_entity(hit.entity, |(locked,)| lock = locked.0);
    let mut fixture = None;
    reach
        .handles
        .for_entity(hit.entity, |(handle, _)| fixture = Some(handle.0));
    let mut page = false;
    reach.readables.for_entity(hit.entity, |_| page = true);

    resources.run.prompt = match (item, lock, fixture) {
        _ if page => "Read it".to_string(),
        (Some(kind), ..) => format!("Take the {}", item_name(kind)),
        (None, Some(kind), _) if resources.player.carrying_key && kind == ItemKind::ShaftKey => {
            "Turn the key".to_string()
        }
        (None, Some(_), _) => "Locked. The plate wants a key".to_string(),
        (None, None, Some(Fixture::Door)) => "Pull the door".to_string(),
        (None, None, Some(Fixture::Drawer)) => "Pull the drawer".to_string(),
        (None, None, Some(Fixture::Loose)) => "Take hold of it".to_string(),
        (None, None, None) => String::new(),
    };
}

fn take_or_unlock(
    resources: &mut BlackmarrowResources,
    physics: &PhysicsWorld,
    commands: &mut Commands<'_>,
    reach: &mut Reach<'_, '_>,
    body: Entity,
) {
    let eye = resources.player.eye;
    let forward = resources.player.forward;
    let Some(hit) = physics_world_cast_ray(physics, eye, forward, REACH, Some(body)) else {
        return;
    };
    let mut page = None;
    reach
        .readables
        .for_entity(hit.entity, |(readable,)| page = Some(readable.0));
    if let Some(index) = page {
        resources.run.reading = Some(index);
        resources.pending_screen = Some(Screen::Reading);
        return;
    }
    let mut item = None;
    reach
        .items
        .for_entity(hit.entity, |(carried,)| item = Some(carried.0));
    if let Some(kind) = item {
        match kind {
            ItemKind::ShaftKey => {
                resources.player.carrying_key = true;
                post_message(
                    &mut resources.run,
                    "A brass key on a tag. It says SHAFT HEAD.",
                );
            }
            ItemKind::Cell => {
                resources.player.spare_cells += 1;
                post_message(
                    &mut resources.run,
                    "A spare cell. It still has weight in it.",
                );
            }
        }
        commands.despawn_recursive(hit.entity);
        return;
    }

    let mut lock = None;
    reach
        .locks
        .for_entity(hit.entity, |(locked,)| lock = locked.0);
    let Some(kind) = lock else {
        return;
    };
    if kind != ItemKind::ShaftKey || !resources.player.carrying_key {
        post_message(&mut resources.run, "It does not give.");
        return;
    }
    let Some(doorway) = DOORWAYS.iter().find(|doorway| doorway.lock == Some(kind)) else {
        return;
    };
    commands.despawn_recursive(hit.entity);
    hang_leaf(commands, doorway);
    resources.player.carrying_key = false;
    post_message(
        &mut resources.run,
        format!(
            "The lock turns. {} is loose on its hinge now.",
            doorway.name
        ),
    );
}
