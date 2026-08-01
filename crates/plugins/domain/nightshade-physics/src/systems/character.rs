use crate::queries::physics_world::physics_world_query_pipeline;
use crate::resources::PhysicsWorld;
use nalgebra_glm;
use nightshade_input::contract::{Actions, EngineAction};
use rapier3d::prelude::*;

use crate::prelude::*;
pub fn character_controller_input_system(
    physics: Res<PhysicsWorld>,
    actions: Res<Actions<EngineAction>>,
    active_camera: Res<nightshade_camera::resources::ActiveCamera>,
    mut camera_transforms: Query<(&nightshade_scene::contract::LocalTransform,)>,
    controllers: Query<(&mut crate::contract::components::CharacterControllerComponent,)>,
) {
    let delta_time = physics.fixed_timestep;

    let mut basis = None;
    if let Some(camera_entity) = active_camera.0 {
        camera_transforms.for_entity(camera_entity, |(camera_transform,)| {
            let forward_vec = camera_transform.forward_vector();
            let right_vec = camera_transform.right_vector();
            basis = Some((
                nalgebra_glm::normalize(&nalgebra_glm::vec3(forward_vec.x, 0.0, forward_vec.z)),
                nalgebra_glm::normalize(&nalgebra_glm::vec3(right_vec.x, 0.0, right_vec.z)),
            ));
        });
    }
    let (forward, right) = basis.unwrap_or((
        nalgebra_glm::vec3(0.0, 0.0, -1.0),
        nalgebra_glm::vec3(1.0, 0.0, 0.0),
    ));

    controllers.for_each(|_entity, (controller,)| {
        let (
            speed,
            acceleration,
            mut velocity,
            mut can_jump,
            jump_impulse,
            crouch_enabled,
            crouch_speed_multiplier,
            sprint_speed_multiplier,
            was_crouching,
            was_sprinting,
            engine_input_enabled,
            external_movement,
            external_jump,
        ) = {
            let character_controller = &*controller;
            (
                character_controller.max_speed,
                character_controller.acceleration,
                character_controller.velocity,
                character_controller.can_jump,
                character_controller.jump_impulse,
                character_controller.crouch_enabled,
                character_controller.crouch_speed_multiplier,
                character_controller.sprint_speed_multiplier,
                character_controller.is_crouching,
                character_controller.is_sprinting,
                character_controller.engine_input_enabled,
                character_controller.external_movement,
                character_controller.external_jump,
            )
        };

        let asked_movement = actions.axis2(EngineAction::Move);
        let jump_asked = actions.pressed(EngineAction::Primary);
        let crouch_asked = actions.pressed(EngineAction::Crouch);
        let sprint_asked = actions.pressed(EngineAction::Sprint);

        let mut movement_direction = nalgebra_glm::vec3(0.0, 0.0, 0.0);

        if engine_input_enabled && nalgebra_glm::length(&asked_movement) > 0.0 {
            movement_direction = forward * asked_movement.y + right * asked_movement.x;
        }

        let external_magnitude = nalgebra_glm::length(&external_movement);
        if external_magnitude > 0.0 {
            let clamped = if external_magnitude > 1.0 {
                external_movement / external_magnitude
            } else {
                external_movement
            };
            movement_direction = forward * clamped.y + right * clamped.x;
        }

        if nalgebra_glm::length(&movement_direction) > 0.0 {
            movement_direction = nalgebra_glm::normalize(&movement_direction);
        }

        let has_movement_input = nalgebra_glm::length(&movement_direction) > 0.0;

        let (is_crouching, is_sprinting) = if engine_input_enabled {
            let crouching = crouch_enabled && crouch_asked && (can_jump || was_crouching);
            let sprinting = sprint_asked && has_movement_input && !crouching;

            (crouching, sprinting)
        } else {
            (was_crouching, was_sprinting)
        };

        let effective_speed = if is_crouching {
            speed * crouch_speed_multiplier
        } else if is_sprinting {
            speed * sprint_speed_multiplier
        } else {
            speed
        };

        let pre_horizontal_speed =
            nalgebra_glm::length(&nalgebra_glm::vec3(velocity.x, 0.0, velocity.z));

        if pre_horizontal_speed <= effective_speed {
            let target_velocity = movement_direction * effective_speed;
            let velocity_change = (target_velocity
                - nalgebra_glm::vec3(velocity.x, 0.0, velocity.z))
                * acceleration
                * delta_time;

            velocity.x += velocity_change.x;
            velocity.z += velocity_change.z;

            let horizontal_speed =
                nalgebra_glm::length(&nalgebra_glm::vec3(velocity.x, 0.0, velocity.z));
            if horizontal_speed > effective_speed {
                let normalized =
                    nalgebra_glm::normalize(&nalgebra_glm::vec3(velocity.x, 0.0, velocity.z));
                velocity.x = normalized.x * effective_speed;
                velocity.z = normalized.z * effective_speed;
            }
        } else if pre_horizontal_speed <= effective_speed * 2.0 {
            let current_horizontal = nalgebra_glm::vec3(velocity.x, 0.0, velocity.z);
            let target_velocity = if has_movement_input {
                movement_direction * effective_speed
            } else if pre_horizontal_speed > 0.01 {
                nalgebra_glm::normalize(&current_horizontal) * effective_speed
            } else {
                nalgebra_glm::Vec3::zeros()
            };
            let velocity_change =
                (target_velocity - current_horizontal) * acceleration * delta_time;

            velocity.x += velocity_change.x;
            velocity.z += velocity_change.z;

            let horizontal_speed =
                nalgebra_glm::length(&nalgebra_glm::vec3(velocity.x, 0.0, velocity.z));
            if horizontal_speed > pre_horizontal_speed {
                let normalized =
                    nalgebra_glm::normalize(&nalgebra_glm::vec3(velocity.x, 0.0, velocity.z));
                velocity.x = normalized.x * pre_horizontal_speed;
                velocity.z = normalized.z * pre_horizontal_speed;
            }
        } else if has_movement_input {
            let steer_accel = acceleration * 0.15;
            let current_horizontal = nalgebra_glm::vec3(velocity.x, 0.0, velocity.z);
            let target_velocity = movement_direction * effective_speed;
            let velocity_change = (target_velocity - current_horizontal) * steer_accel * delta_time;

            velocity.x += velocity_change.x;
            velocity.z += velocity_change.z;

            let horizontal_speed =
                nalgebra_glm::length(&nalgebra_glm::vec3(velocity.x, 0.0, velocity.z));
            if horizontal_speed > pre_horizontal_speed {
                let normalized =
                    nalgebra_glm::normalize(&nalgebra_glm::vec3(velocity.x, 0.0, velocity.z));
                velocity.x = normalized.x * pre_horizontal_speed;
                velocity.z = normalized.z * pre_horizontal_speed;
            }
        }

        if engine_input_enabled {
            let jump_pressed = jump_asked || external_jump;
            if jump_pressed && can_jump {
                velocity.y = jump_impulse;
                can_jump = false;
            }
        }

        controller.velocity = velocity;
        controller.can_jump = can_jump;
        controller.is_crouching = is_crouching;
        controller.is_sprinting = is_sprinting;
        controller.external_jump = false;
        let radius = match controller.shape {
            crate::contract::ColliderShape::Capsule { radius, .. } => radius,
            _ => 0.3,
        };
        let half_height = if is_crouching {
            controller.crouching_half_height
        } else {
            controller.standing_half_height
        };
        controller.shape = crate::contract::ColliderShape::Capsule {
            half_height,
            radius,
        };
    });
}

pub fn character_controller_system(
    mut physics: ResMut<PhysicsWorld>,
    mut commands: Commands,
    controllers: Query<(
        &mut crate::contract::components::CharacterControllerComponent,
        &nightshade_scene::contract::LocalTransform,
        Option<&mut crate::contract::components::PhysicsInterpolation>,
    )>,
) {
    let delta_time = physics.fixed_timestep;
    let gravity_y = physics.gravity.y;

    controllers.for_each(|entity, (controller, local_transform, interpolation)| {
        let (mut velocity, shape, controller_config, scale, position, disable_gravity) = (
            controller.velocity,
            controller.shape.clone(),
            controller.config.clone(),
            controller.scale,
            local_transform.translation,
            controller.disable_gravity,
        );

        if disable_gravity {
            controller.velocity = Vec3::zeros();
            controller.grounded = false;
            controller.can_jump = true;
            return;
        }

        velocity.y += gravity_y * delta_time;

        let desired_translation = vector![
            velocity.x * delta_time,
            velocity.y * delta_time,
            velocity.z * delta_time,
        ];

        let character_shape = match &shape {
            crate::contract::ColliderShape::Capsule {
                half_height,
                radius,
            } => SharedShape::capsule_y(*half_height, *radius),
            crate::contract::ColliderShape::Ball { radius } => SharedShape::ball(*radius),
            crate::contract::ColliderShape::Cuboid { hx, hy, hz } => {
                SharedShape::cuboid(*hx, *hy, *hz)
            }
            _ => SharedShape::capsule_y(0.9, 0.3),
        };

        let character_pos = rapier3d::na::Isometry3::new(
            vector![position.x, position.y, position.z],
            rapier3d::na::Vector3::zeros(),
        );

        let mut collisions = Vec::new();

        let rapier_controller =
            crate::data::rapier_convert::character_controller_to_rapier(&controller_config, scale);
        let corrected_movement = {
            let query_pipeline = physics_world_query_pipeline(&physics);
            rapier_controller.move_shape(
                delta_time,
                &query_pipeline,
                character_shape.as_ref(),
                &character_pos,
                desired_translation,
                |collision| collisions.push(collision),
            )
        };

        let new_translation = nalgebra_glm::vec3(
            character_pos.translation.x + corrected_movement.translation.x,
            character_pos.translation.y + corrected_movement.translation.y,
            character_pos.translation.z + corrected_movement.translation.z,
        );

        if new_translation != position {
            let mut moved = *local_transform;
            moved.translation = new_translation;
            commands.set(entity, moved);
        }

        if let Some(interpolation) = interpolation {
            interpolation.update_current(new_translation, nalgebra_glm::quat_identity());
        }

        for collision in &collisions {
            let parent_handle = physics
                .collider_set
                .get(collision.handle)
                .and_then(|collider| collider.parent());
            if let Some(parent_handle) = parent_handle
                && let Some(rb) = physics.rigid_body_set.get_mut(parent_handle)
                && rb.body_type() == RigidBodyType::Dynamic
            {
                let push_strength = 2.0;
                let impulse = vector![
                    velocity.x * push_strength * delta_time,
                    0.0,
                    velocity.z * push_strength * delta_time
                ];
                rb.apply_impulse(impulse, true);
            }
        }

        controller.grounded = corrected_movement.grounded;

        if controller.grounded && velocity.y <= 0.0 {
            controller.can_jump = true;
            velocity.y = 0.0;
        } else if !controller.grounded {
            controller.can_jump = false;
        }

        if controller.grounded && !controller.is_crouching {
            let horizontal_speed =
                nalgebra_glm::length(&nalgebra_glm::vec3(velocity.x, 0.0, velocity.z));
            if horizontal_speed <= controller.max_speed {
                let friction_factor = 1.0 - (controller.friction_rate * delta_time).min(1.0);
                velocity.x *= friction_factor;
                velocity.z *= friction_factor;
            } else {
                let friction = (-controller.above_max_friction_rate * delta_time).exp();
                velocity.x *= friction;
                velocity.z *= friction;
            }
        }

        controller.velocity = velocity;
        controller.config = controller_config;
    });
}
