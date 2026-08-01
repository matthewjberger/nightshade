use crate::commands::{count_fall, record_best};
use crate::components::{Grounded, Hook, LineLength, Speed};
use crate::contract::TetherEvent;
use crate::data::{
    AIR_ACCELERATION, AIR_SPEED_CAP, Ask, DRAG, FALL_LEVEL, GROUND_ACCELERATION, GROUND_FRICTION,
    GROUND_PROBE, GROUND_SPEED_CAP, JUMP_SPEED, LONGEST_STEP, PLAYER_RADIUS, PULL_ACCELERATION,
    REEL_RATE, SHORTEST_LINE, START,
};
use crate::queries::in_control;
use crate::resources::TetherResources;
use nightshade::prelude::*;
use nightshade_input::Actions;
use nightshade_physics::commands::{
    physics_world_set_linear_velocity, physics_world_set_translation, reset_physics_interpolation,
};
use nightshade_physics::queries::{
    physics_world_body_translation, physics_world_cast_ray, physics_world_linear_velocity,
};
use nightshade_physics::resources::PhysicsWorld;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::LocalTransform;

pub fn drive_the_runner(
    time: Res<Time>,
    input: Res<Input>,
    actions: Res<Actions<Ask>>,
    window: Res<Window>,
    mut physics: ResMut<PhysicsWorld>,
    mut resources: ResMut<TetherResources>,
    mut runners: Query<(
        &mut Speed,
        &mut Grounded,
        Option<&Hook>,
        Option<&mut LineLength>,
    )>,
    mut cameras: Query<(&LocalTransform,)>,
) {
    let Some(runner) = resources.scene.runner else {
        return;
    };
    let Some(camera) = resources.scene.camera else {
        return;
    };
    let Some(body) = physics_world_body_translation(&physics, runner) else {
        return;
    };
    let Some(mut velocity) = physics_world_linear_velocity(&physics, runner) else {
        return;
    };

    let delta_time = time.delta_time.min(LONGEST_STEP);
    let playing = in_control(&window, &input);
    let standing = physics_world_cast_ray(
        &physics,
        body,
        -Vec3::y(),
        PLAYER_RADIUS + GROUND_PROBE,
        Some(runner),
    )
    .is_some();

    let wish = if playing {
        wish_direction(&actions, camera, &mut cameras)
    } else {
        Vec3::zeros()
    };

    runners.for_entity(runner, |(speed, grounded, anchor, line)| {
        grounded.0 = standing;

        if standing && anchor.is_none() {
            velocity = rub_the_ground(velocity, delta_time);
            velocity = accelerate(
                velocity,
                wish,
                GROUND_SPEED_CAP,
                GROUND_ACCELERATION,
                delta_time,
            );
            if playing && actions.pressed(Ask::Jump) {
                velocity.y = JUMP_SPEED;
            }
        } else {
            velocity = accelerate(velocity, wish, AIR_SPEED_CAP, AIR_ACCELERATION, delta_time);
        }

        if let (Some(anchor), Some(line)) = (anchor, line) {
            let to_anchor = anchor.0 - body;
            let distance = to_anchor.norm();
            if distance > f32::EPSILON {
                let direction = to_anchor / distance;
                velocity += direction * PULL_ACCELERATION * delta_time;
                line.0 = (line.0.min(distance) - REEL_RATE * delta_time).max(SHORTEST_LINE);
                if distance > line.0 {
                    let radial = velocity.dot(&direction);
                    if radial < 0.0 {
                        velocity -= direction * radial;
                    }
                }
            }
        }

        let moving = velocity.norm();
        if moving > 0.0 {
            let slowing = (moving * moving * DRAG * delta_time).min(moving);
            velocity -= velocity / moving * slowing;
        }
        speed.0 = velocity.norm();
    });

    physics_world_set_linear_velocity(&mut physics, runner, velocity);
    record_best(&mut resources.run, velocity.norm());
}

pub fn catch_the_falls(
    mut physics: ResMut<PhysicsWorld>,
    mut resources: ResMut<TetherResources>,
    mut commands: Commands,
    mut events: EventWriter<TetherEvent>,
    mut runners: Query<(&mut Speed,)>,
) {
    let Some(runner) = resources.scene.runner else {
        return;
    };
    let Some(body) = physics_world_body_translation(&physics, runner) else {
        return;
    };
    if body.y > FALL_LEVEL {
        return;
    }

    let mut speed = 0.0;
    runners.for_entity(runner, |(runner_speed,)| speed = runner_speed.0);

    return_to_start(&mut physics, &mut commands, &mut runners, runner);
    count_fall(&mut resources.run);
    events.send(TetherEvent::Fell { speed });
}

pub fn return_to_start(
    physics: &mut PhysicsWorld,
    commands: &mut Commands,
    runners: &mut Query<'_, (&mut Speed,)>,
    runner: Entity,
) {
    let start = Vec3::new(START[0], START[1], START[2]);
    physics_world_set_translation(physics, runner, start);
    physics_world_set_linear_velocity(physics, runner, Vec3::zeros());
    commands.remove::<(Hook, LineLength)>(runner);
    runners.for_entity(runner, |(speed,)| speed.0 = 0.0);

    commands.queue(move |storage, _| {
        if let Some(transform) = storage.get_mut::<LocalTransform>(runner) {
            transform.translation = start;
        }
    });
    reset_physics_interpolation(commands, runner);
}

fn wish_direction(
    actions: &Actions<Ask>,
    camera: Entity,
    cameras: &mut Query<'_, (&LocalTransform,)>,
) -> Vec3 {
    let mut basis = None;
    cameras.for_entity(camera, |(transform,)| {
        let forward =
            nalgebra_glm::quat_rotate_vec3(&transform.rotation, &Vec3::new(0.0, 0.0, -1.0));
        let right = nalgebra_glm::quat_rotate_vec3(&transform.rotation, &Vec3::x());
        basis = Some((level(forward), level(right)));
    });
    let Some((forward, right)) = basis else {
        return Vec3::zeros();
    };

    let walk = actions.axis2(Ask::Walk);
    let wish = forward * walk.y + right * walk.x;
    if wish.norm_squared() <= f32::EPSILON {
        return Vec3::zeros();
    }
    wish.normalize()
}

fn level(direction: Vec3) -> Vec3 {
    let flat = Vec3::new(direction.x, 0.0, direction.z);
    if flat.norm_squared() <= f32::EPSILON {
        return Vec3::zeros();
    }
    flat.normalize()
}

fn accelerate(velocity: Vec3, wish: Vec3, cap: f32, acceleration: f32, delta_time: f32) -> Vec3 {
    if wish.norm_squared() <= f32::EPSILON {
        return velocity;
    }
    let along = velocity.dot(&wish);
    let room = cap - along;
    if room <= 0.0 {
        return velocity;
    }
    velocity + wish * (acceleration * delta_time).min(room)
}

fn rub_the_ground(velocity: Vec3, delta_time: f32) -> Vec3 {
    let flat = Vec3::new(velocity.x, 0.0, velocity.z);
    let speed = flat.norm();
    if speed <= f32::EPSILON {
        return velocity;
    }
    let kept = ((speed - speed * GROUND_FRICTION * delta_time) / speed).max(0.0);
    Vec3::new(velocity.x * kept, velocity.y, velocity.z * kept)
}
