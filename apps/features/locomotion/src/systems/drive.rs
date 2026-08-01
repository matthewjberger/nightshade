use nightshade::prelude::*;
use nightshade_animation::contract::AnimationPlayer;
use nightshade_camera_controller::contract::ThirdPersonCamera;
use nightshade_input::prelude::Actions;
use nightshade_scene::prelude::*;

use crate::data;
use crate::data::Move;
use crate::resources::LocomotionResources;

struct Intent {
    direction: Vec2,
    facing_locked: bool,
}

pub fn drive(
    mut locomotion_resources: ResMut<LocomotionResources>,
    time: Res<nightshade_platform::contract::time::Time>,
    actions: Res<Actions<Move>>,
    storage: &mut nightshade::ecs::Storage,
) {
    let Some(body) = locomotion_resources.character.body else {
        return;
    };
    let model = locomotion_resources.character.model;
    let Some(camera) = locomotion_resources.character.camera else {
        return;
    };
    let delta_time = time.delta_time;
    if delta_time <= 0.0 {
        return;
    }

    let intent = read_intent(&actions);
    let camera_yaw = storage
        .get::<ThirdPersonCamera>(camera)
        .map(|follow| follow.yaw)
        .unwrap_or(0.0);

    let forward = Vec3::new(-camera_yaw.sin(), 0.0, -camera_yaw.cos());
    let right = Vec3::new(camera_yaw.cos(), 0.0, -camera_yaw.sin());
    let wish = forward * intent.direction.y + right * intent.direction.x;
    let wish_magnitude = intent.direction.norm().min(1.0);

    let target_velocity = if wish_magnitude > 0.0 {
        wish.normalize() * (wish_magnitude * data::RUN_SPEED)
    } else {
        Vec3::zeros()
    };

    let state = &mut locomotion_resources.character;
    state.locked = intent.facing_locked;
    let sharpness = if target_velocity.norm_squared() > state.velocity.norm_squared() {
        data::ACCELERATION
    } else {
        data::DECELERATION
    };
    let sharpness = if state.airborne {
        sharpness * data::AIR_CONTROL
    } else {
        sharpness
    };
    let blend = 1.0 - (-sharpness * delta_time).exp();
    state.velocity += (target_velocity - state.velocity) * blend;
    if state.velocity.norm() < 0.02 {
        state.velocity = Vec3::zeros();
    }

    let desired_facing = if state.airborne {
        state.facing
    } else if intent.facing_locked {
        camera_yaw + std::f32::consts::PI
    } else if state.velocity.norm_squared() > 0.01 {
        state.velocity.x.atan2(state.velocity.z)
    } else {
        state.facing
    };
    let error = wrap_angle(desired_facing - state.facing);
    state.facing_error = if wish_magnitude > 0.0 {
        wrap_angle(wish.x.atan2(wish.z) - state.facing)
    } else {
        error
    };
    let turn = 1.0 - (-data::TURN_SHARPNESS * delta_time).exp();
    state.facing = wrap_angle(state.facing + error * turn);

    let facing = state.facing;
    let velocity = state.velocity;
    let speed = velocity.norm();
    let (target_forward, target_lateral) = if intent.facing_locked {
        let wish = intent.direction.norm();
        if wish > 1.0e-4 {
            (
                intent.direction.y / wish * speed,
                intent.direction.x / wish * speed,
            )
        } else {
            (0.0, 0.0)
        }
    } else {
        (speed, 0.0)
    };

    let parameter_blend = 1.0 - (-data::PARAMETER_SHARPNESS * delta_time).exp();
    state.forward_parameter += (target_forward - state.forward_parameter) * parameter_blend;
    state.lateral_parameter += (target_lateral - state.lateral_parameter) * parameter_blend;
    let forward_parameter = state.forward_parameter;
    let lateral_parameter = state.lateral_parameter;
    let walking = speed > 0.05 && speed < data::WALK_SPEED * 1.35;
    locomotion_resources.walking = walking;

    let blocks = locomotion_resources.arena.blocks.clone();
    if let Some(transform) = storage.get_mut::<LocalTransform>(body) {
        transform.translation += velocity * delta_time;
        transform.translation =
            crate::queries::collision::push_out(transform.translation, data::BODY_RADIUS, &blocks);
        transform.rotation = nalgebra_glm::quat_angle_axis(facing, &Vec3::new(0.0, 1.0, 0.0));
    }
    let Some(model) = model else {
        return;
    };
    if let Some(player) = storage.get_mut::<AnimationPlayer>(model) {
        player
            .parameters
            .insert(data::FORWARD.to_string(), forward_parameter);
        player
            .parameters
            .insert(data::LATERAL.to_string(), lateral_parameter);
    }
}

fn read_intent(actions: &Actions<Move>) -> Intent {
    let mut direction = actions.axis2(Move::Walk);
    if actions.pressed(Move::Slow) {
        direction *= data::WALK_SPEED / data::RUN_SPEED;
    }
    let sideways =
        direction.norm() > 0.05 && direction.x.abs() > direction.y.abs() * data::STRAFE_BIAS;
    Intent {
        direction,
        facing_locked: actions.pressed(Move::LockFacing) || sideways,
    }
}

fn wrap_angle(angle: f32) -> f32 {
    let mut wrapped = angle;
    while wrapped > std::f32::consts::PI {
        wrapped -= std::f32::consts::TAU;
    }
    while wrapped < -std::f32::consts::PI {
        wrapped += std::f32::consts::TAU;
    }
    wrapped
}
