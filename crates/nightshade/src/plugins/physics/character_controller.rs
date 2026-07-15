use super::resources::{PhysicsWorld, physics_world_query_pipeline};
use crate::ecs::world::World;
use rapier3d::prelude::*;

use crate::prelude::*;
pub fn character_controller_input_system(world: &mut World) {
    let entities: Vec<_> = world.ecs.worlds[CORE]
        .query_entities(crate::ecs::world::CHARACTER_CONTROLLER)
        .collect();

    let delta_time = world.plugin_resource::<PhysicsWorld>().fixed_timestep;

    for entity in entities {
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
            let Some(character_controller) =
                world.get::<crate::plugins::physics::components::CharacterControllerComponent>(
                    entity,
                )
            else {
                continue;
            };

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

        let (
            left_key_pressed,
            right_key_pressed,
            forward_key_pressed,
            backward_key_pressed,
            jump_key_pressed,
            crouch_key_pressed,
            sprint_key_pressed,
        ) = {
            let keyboard = &world
                .expect_resource_mut::<crate::ecs::input::resources::Input>()
                .keyboard;
            (
                keyboard.is_key_pressed(winit::keyboard::KeyCode::KeyA),
                keyboard.is_key_pressed(winit::keyboard::KeyCode::KeyD),
                keyboard.is_key_pressed(winit::keyboard::KeyCode::KeyW),
                keyboard.is_key_pressed(winit::keyboard::KeyCode::KeyS),
                keyboard.is_key_pressed(winit::keyboard::KeyCode::Space),
                keyboard.is_key_pressed(winit::keyboard::KeyCode::ControlLeft)
                    || keyboard.is_key_pressed(winit::keyboard::KeyCode::KeyC),
                keyboard.is_key_pressed(winit::keyboard::KeyCode::ShiftLeft),
            )
        };

        #[cfg(feature = "gamepad")]
        let (gamepad_movement, gamepad_jump, gamepad_crouch, gamepad_sprint) =
            crate::plugins::gamepad::with_active_gamepad(world, |gamepad| {
                let left_stick_x = gamepad.value(gilrs::Axis::LeftStickX);
                let left_stick_y = gamepad.value(gilrs::Axis::LeftStickY);
                let jump_button = gamepad.is_pressed(gilrs::Button::South);
                let crouch_button = gamepad.is_pressed(gilrs::Button::East);
                let sprint_button = gamepad.is_pressed(gilrs::Button::LeftThumb);

                let deadzone = 0.15;
                let left_stick_magnitude =
                    (left_stick_x * left_stick_x + left_stick_y * left_stick_y).sqrt();
                let movement = if left_stick_magnitude > deadzone {
                    let normalized_magnitude = (left_stick_magnitude - deadzone) / (1.0 - deadzone);
                    nalgebra_glm::vec2(left_stick_x, left_stick_y) * normalized_magnitude
                } else {
                    nalgebra_glm::vec2(0.0, 0.0)
                };

                (movement, jump_button, crouch_button, sprint_button)
            })
            .unwrap_or((nalgebra_glm::vec2(0.0, 0.0), false, false, false));
        #[cfg(not(feature = "gamepad"))]
        let (gamepad_movement, gamepad_jump, gamepad_crouch, gamepad_sprint) =
            (nalgebra_glm::vec2(0.0, 0.0), false, false, false);

        let (forward, right) = if let Some(camera_entity) = world
            .expect_resource_mut::<crate::ecs::camera::resources::ActiveCamera>()
            .0
        {
            if let Some(camera_transform) =
                world.get::<crate::ecs::transform::components::LocalTransform>(camera_entity)
            {
                let forward_vec = camera_transform.forward_vector();
                let right_vec = camera_transform.right_vector();
                let forward_horizontal =
                    nalgebra_glm::normalize(&nalgebra_glm::vec3(forward_vec.x, 0.0, forward_vec.z));
                let right_horizontal =
                    nalgebra_glm::normalize(&nalgebra_glm::vec3(right_vec.x, 0.0, right_vec.z));
                (forward_horizontal, right_horizontal)
            } else {
                (
                    nalgebra_glm::vec3(0.0, 0.0, -1.0),
                    nalgebra_glm::vec3(1.0, 0.0, 0.0),
                )
            }
        } else {
            (
                nalgebra_glm::vec3(0.0, 0.0, -1.0),
                nalgebra_glm::vec3(1.0, 0.0, 0.0),
            )
        };

        let mut movement_direction = nalgebra_glm::vec3(0.0, 0.0, 0.0);

        if forward_key_pressed {
            movement_direction += forward;
        }
        if backward_key_pressed {
            movement_direction -= forward;
        }
        if left_key_pressed {
            movement_direction -= right;
        }
        if right_key_pressed {
            movement_direction += right;
        }

        if nalgebra_glm::length(&gamepad_movement) > 0.0 {
            movement_direction = forward * gamepad_movement.y + right * gamepad_movement.x;
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
            let crouch_input_pressed = crouch_key_pressed || gamepad_crouch;
            let crouching = crouch_enabled && crouch_input_pressed && (can_jump || was_crouching);

            let sprint_input_pressed = sprint_key_pressed || gamepad_sprint;
            let sprinting = sprint_input_pressed && has_movement_input && !crouching;

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
            let jump_pressed = jump_key_pressed || gamepad_jump || external_jump;
            if jump_pressed && can_jump {
                velocity.y = jump_impulse;
                can_jump = false;
            }
        }

        if let Some(controller) = world
            .get_mut::<crate::plugins::physics::components::CharacterControllerComponent>(
            entity,
        ) {
            controller.velocity = velocity;
            controller.can_jump = can_jump;
            controller.is_crouching = is_crouching;
            controller.is_sprinting = is_sprinting;
            controller.external_jump = false;
            if is_crouching {
                controller.shape = super::ColliderShape::Capsule {
                    half_height: controller.crouching_half_height,
                    radius: match controller.shape {
                        super::ColliderShape::Capsule { radius, .. } => radius,
                        _ => 0.3,
                    },
                };
            } else {
                controller.shape = super::ColliderShape::Capsule {
                    half_height: controller.standing_half_height,
                    radius: match controller.shape {
                        super::ColliderShape::Capsule { radius, .. } => radius,
                        _ => 0.3,
                    },
                };
            }
        }
    }
}

pub fn character_controller_system(world: &mut World) {
    let entities: Vec<_> = world.ecs.worlds[CORE]
        .query_entities(
            crate::ecs::world::CHARACTER_CONTROLLER | crate::ecs::world::LOCAL_TRANSFORM,
        )
        .collect();

    let delta_time = world.plugin_resource::<PhysicsWorld>().fixed_timestep;
    let gravity_y = world.plugin_resource::<PhysicsWorld>().gravity.y;

    for entity in entities {
        let (mut velocity, shape, controller_config, scale, position, disable_gravity) = {
            let character_controller =
                world.get::<crate::plugins::physics::components::CharacterControllerComponent>(
                    entity,
                );
            let local_transform =
                world.get::<crate::ecs::transform::components::LocalTransform>(entity);

            if let (Some(cc), Some(lt)) = (character_controller, local_transform) {
                (
                    cc.velocity,
                    cc.shape.clone(),
                    cc.config.clone(),
                    cc.scale,
                    lt.translation,
                    cc.disable_gravity,
                )
            } else {
                continue;
            }
        };

        if disable_gravity {
            if let Some(controller) =
                world.get_mut::<crate::plugins::physics::components::CharacterControllerComponent>(
                    entity,
                )
            {
                controller.velocity = Vec3::zeros();
                controller.grounded = false;
                controller.can_jump = true;
            }
            continue;
        }

        velocity.y += gravity_y * delta_time;

        let desired_translation = vector![
            velocity.x * delta_time,
            velocity.y * delta_time,
            velocity.z * delta_time,
        ];

        let character_shape = match &shape {
            super::ColliderShape::Capsule {
                half_height,
                radius,
            } => SharedShape::capsule_y(*half_height, *radius),
            super::ColliderShape::Ball { radius } => SharedShape::ball(*radius),
            super::ColliderShape::Cuboid { hx, hy, hz } => SharedShape::cuboid(*hx, *hy, *hz),
            _ => SharedShape::capsule_y(0.9, 0.3),
        };

        let character_pos = rapier3d::na::Isometry3::new(
            vector![position.x, position.y, position.z],
            rapier3d::na::Vector3::zeros(),
        );

        let mut collisions = Vec::new();

        let rapier_controller = controller_config.to_rapier(scale);
        let corrected_movement = {
            let physics = world.plugin_resource::<PhysicsWorld>();
            let query_pipeline = physics_world_query_pipeline(physics);
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

        if let Some(local_transform) =
            world.get_mut::<crate::ecs::transform::components::LocalTransform>(entity)
        {
            local_transform.translation = new_translation;
        }

        if let Some(interpolation) =
            world.get_mut::<crate::plugins::physics::components::PhysicsInterpolation>(entity)
        {
            interpolation.update_current(new_translation, nalgebra_glm::quat_identity());
        }

        for collision in &collisions {
            let parent_handle = world
                .plugin_resource::<PhysicsWorld>()
                .collider_set
                .get(collision.handle)
                .and_then(|collider| collider.parent());
            if let Some(parent_handle) = parent_handle
                && let Some(rb) = world
                    .plugin_resource_mut::<PhysicsWorld>()
                    .rigid_body_set
                    .get_mut(parent_handle)
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

        if let Some(character_controller) =
            world.get_mut::<crate::plugins::physics::components::CharacterControllerComponent>(
                entity,
            )
        {
            character_controller.grounded = corrected_movement.grounded;

            if character_controller.grounded && velocity.y <= 0.0 {
                character_controller.can_jump = true;
                velocity.y = 0.0;
            } else if !character_controller.grounded {
                character_controller.can_jump = false;
            }

            if character_controller.grounded && !character_controller.is_crouching {
                let horizontal_speed =
                    nalgebra_glm::length(&nalgebra_glm::vec3(velocity.x, 0.0, velocity.z));
                if horizontal_speed <= character_controller.max_speed {
                    let friction_factor =
                        1.0 - (character_controller.friction_rate * delta_time).min(1.0);
                    velocity.x *= friction_factor;
                    velocity.z *= friction_factor;
                } else {
                    let friction =
                        (-character_controller.above_max_friction_rate * delta_time).exp();
                    velocity.x *= friction;
                    velocity.z *= friction;
                }
            }

            character_controller.velocity = velocity;
            character_controller.config = controller_config;
        }
    }
}
