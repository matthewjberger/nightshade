use crate::ecs::SandboxWorld;
use crate::systems::world::level::{default_spawn_position, find_player_spawn};
use nightshade::ecs::input::queries::query_active_gamepad;
use nightshade::ecs::lines::components::{Line, Lines};
use nightshade::ecs::world::{
    GLOBAL_TRANSFORM, LINES, LOCAL_TRANSFORM, LOCAL_TRANSFORM_DIRTY, NAME, VISIBILITY,
};
use nightshade::prelude::*;

const BUMPER_RADIUS: f32 = 0.55;
const BUMPER_HALF_HEIGHT: f32 = 0.55;
const BUMPER_CENTER_HEIGHT: f32 = 1.55;
const LIFTER_RADIUS: f32 = 0.45;
const LIFTER_CENTER_HEIGHT: f32 = 0.45;
const RIDE_PROBE_HEIGHT: f32 = 1.0;
const LIFTER_PROBE_DISTANCE: f32 = 0.4;
const LIFTER_SPRING_STIFFNESS: f32 = 200.0;
const LIFTER_SPRING_DAMPING: f32 = 22.0;
const GROUNDED_MARGIN: f32 = 0.15;
const MAX_GROUND_SPEED: f32 = 7.0;
const GROUND_ACCELERATION: f32 = 40.0;
const AIR_ACCELERATION: f32 = 12.0;
const GROUND_FRICTION: f32 = 8.0;
const JUMP_IMPULSE: f32 = 5.5;
const JUMP_SPRING_SUPPRESS_SECONDS: f32 = 0.3;
const FIRST_PERSON_CAPSULE_HALF_HEIGHT: f32 = 1.0;
const CAMERA_DISTANCE: f32 = 6.0;
const CAMERA_PITCH: f32 = 0.3;
const GAMEPAD_DEADZONE: f32 = 0.15;
const FACING_SPEED_THRESHOLD: f32 = 0.5;
const FACING_TURN_SHARPNESS: f32 = 12.0;
const LEAN_RADIANS_PER_ACCELERATION: f32 = 0.001;
const LEAN_MAX_RADIANS: f32 = 0.3;
const LEAN_SHARPNESS: f32 = 10.0;
const MAX_STEP_SECONDS: f32 = 1.0 / 30.0;
const BUMPER_RESOLVE_ITERATIONS: usize = 3;
const SPHERE_SEGMENTS: usize = 24;
const SPHERE_LATITUDE_RINGS: usize = 12;
const SPHERE_MERIDIAN_CIRCLES: usize = 12;
const STRIDE_WHEEL_RADIUS: f32 = 0.5;
const STRIDE_WHEEL_SPOKES: usize = 8;
const BUMPER_COLOR: Vec4 = Vec4::new(0.2, 1.0, 0.3, 1.0);
const LIFTER_COLOR: Vec4 = Vec4::new(1.0, 1.0, 1.0, 1.0);
const STRIDE_WHEEL_COLOR: Vec4 = Vec4::new(1.0, 0.85, 0.2, 1.0);

pub fn spawn(sandbox_world: &mut SandboxWorld, world: &mut World) {
    let feet = spawn_feet_position(world);

    let body = spawn_entities(
        world,
        NAME | LOCAL_TRANSFORM | GLOBAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY,
        1,
    )[0];
    if let Some(name) = world.core.get_name_mut(body) {
        name.0 = "Character".to_string();
    }
    if let Some(transform) = world.core.get_local_transform_mut(body) {
        transform.translation = feet;
    }
    mark_local_transform_dirty(world, body);

    let camera = spawn_third_person_camera(
        world,
        body,
        CAMERA_DISTANCE,
        0.0,
        CAMERA_PITCH,
        "Character Camera".to_string(),
    );

    let spheres = spawn_entities(
        world,
        NAME | LINES | VISIBILITY | LOCAL_TRANSFORM | GLOBAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY,
        1,
    )[0];
    if let Some(name) = world.core.get_name_mut(spheres) {
        name.0 = "Character Spheres".to_string();
    }
    world.core.set_lines(spheres, Lines::new(Vec::new()));
    if let Some(visibility) = world.core.get_visibility_mut(spheres) {
        visibility.visible = false;
    }

    let state = &mut sandbox_world.resources.character;
    state.body = Some(body);
    state.camera = Some(camera);
    state.spheres = Some(spheres);
    state.stride_wheel_visible = true;

    #[cfg(not(target_arch = "wasm32"))]
    spawn_model(sandbox_world, world, body);
}

#[cfg(not(target_arch = "wasm32"))]
const MODEL_PATH: &str = "apps/sandbox/assets/racer.glb";

#[cfg(not(target_arch = "wasm32"))]
fn spawn_model(sandbox_world: &mut SandboxWorld, world: &mut World, body: Entity) {
    let bytes = match std::fs::read(MODEL_PATH) {
        Ok(bytes) => bytes,
        Err(error) => {
            tracing::warn!("no character model at {MODEL_PATH}: {error}");
            return;
        }
    };
    let mut result = match nightshade::ecs::prefab::import_gltf_from_bytes(&bytes) {
        Ok(result) => result,
        Err(error) => {
            tracing::error!("failed to import character model: {error}");
            return;
        }
    };
    nightshade::ecs::loading::queue_gltf_load(world, &mut result);
    let Some(prefab) = result.prefabs.first().cloned() else {
        return;
    };
    let model = nightshade::ecs::prefab::spawn_prefab_with_skins(
        world,
        &prefab,
        &result.animations,
        &result.skins,
        Vec3::zeros(),
    );
    if let Some(name) = world.core.get_name_mut(model) {
        name.0 = "Character Model".to_string();
    }
    world
        .core
        .add_components(model, nightshade::ecs::world::PARENT);
    world.core.set_parent(
        model,
        nightshade::ecs::transform::components::Parent(Some(body)),
    );
    world.resources.transform_state.children_cache_valid = false;
    if let Some(transform) = world.core.get_local_transform_mut(model) {
        transform.translation = Vec3::zeros();
    }
    mark_local_transform_dirty(world, model);

    if let Some(player) = world.core.get_animation_player_mut(model) {
        let idle_clip = player
            .clips
            .iter()
            .position(|clip| clip.name.to_lowercase().contains("idle"));
        let clip = idle_clip.or(if player.clips.is_empty() {
            None
        } else {
            Some(0)
        });
        if let Some(index) = clip {
            player.play(index);
            player.looping = true;
        }
    }

    sandbox_world.resources.character.model = Some(model);
    crate::systems::world::pose::initialize(sandbox_world, world, model);
    crate::systems::world::pose::set_pose(sandbox_world, world, "idle");
}

fn set_model_visible(sandbox_world: &SandboxWorld, world: &mut World, visible: bool) {
    let Some(model) = sandbox_world.resources.character.model else {
        return;
    };
    let mut stack = vec![model];
    while let Some(entity) = stack.pop() {
        if let Some(visibility) = world.core.get_visibility_mut(entity) {
            visibility.visible = visible;
        }
        stack.extend(nightshade::ecs::transform::queries::query_children(
            world, entity,
        ));
    }
}

pub fn set_third_person(sandbox_world: &mut SandboxWorld, world: &mut World, enabled: bool) {
    sandbox_world.resources.character.third_person = enabled;
    let Some(body) = sandbox_world.resources.character.body else {
        return;
    };
    let Some(player_entity) = sandbox_world.resources.player.player_entity else {
        return;
    };
    let fly = sandbox_world.resources.player.fly;

    if enabled {
        let feet = world
            .core
            .get_local_transform(player_entity)
            .map(|transform| {
                transform.translation - Vec3::new(0.0, FIRST_PERSON_CAPSULE_HALF_HEIGHT, 0.0)
            })
            .unwrap_or_else(|| spawn_feet_position(world));
        if let Some(transform) = world.core.get_local_transform_mut(body) {
            transform.translation = feet;
        }
        mark_local_transform_dirty(world, body);

        let state = &mut sandbox_world.resources.character;
        state.velocity = Vec3::zeros();
        state.grounded = false;
        state.jump_suppress_timer = 0.0;

        if let Some(controller) = world.core.get_character_controller_mut(player_entity) {
            controller.disable_gravity = true;
            controller.velocity = Vec3::zeros();
        }
        if let Some(spheres) = sandbox_world.resources.character.spheres
            && let Some(visibility) = world.core.get_visibility_mut(spheres)
        {
            visibility.visible = true;
        }
        set_model_visible(sandbox_world, world, true);
        if !fly {
            world.resources.active_camera = sandbox_world.resources.character.camera;
        }
    } else {
        if let Some(controller) = world.core.get_character_controller_mut(player_entity) {
            controller.disable_gravity = false;
            controller.velocity = Vec3::zeros();
        }
        reset_physics_interpolation(world, player_entity);
        if let Some(spheres) = sandbox_world.resources.character.spheres
            && let Some(visibility) = world.core.get_visibility_mut(spheres)
        {
            visibility.visible = false;
        }
        set_model_visible(sandbox_world, world, false);
        if !fly {
            world.resources.active_camera = sandbox_world.resources.player.camera_entity;
        }
    }
}

pub fn update(sandbox_world: &mut SandboxWorld, world: &mut World) {
    if !sandbox_world.resources.character.third_person {
        return;
    }
    let Some(body) = sandbox_world.resources.character.body else {
        return;
    };
    let Some(camera) = sandbox_world.resources.character.camera else {
        return;
    };

    move_character(sandbox_world, world, body, camera);
    sync_first_person_player(sandbox_world, world, body);
    third_person_camera_system(world);
    draw_spheres(sandbox_world, world, body);
}

pub fn reset(sandbox_world: &mut SandboxWorld, world: &mut World) {
    let spawn = find_player_spawn(world).unwrap_or_else(default_spawn_position);
    teleport(sandbox_world, world, spawn);
}

pub fn teleport(sandbox_world: &mut SandboxWorld, world: &mut World, capsule_center: Vec3) {
    let Some(body) = sandbox_world.resources.character.body else {
        return;
    };
    let feet = capsule_center - Vec3::new(0.0, FIRST_PERSON_CAPSULE_HALF_HEIGHT, 0.0);
    if let Some(transform) = world.core.get_local_transform_mut(body) {
        transform.translation = feet;
    }
    mark_local_transform_dirty(world, body);

    let state = &mut sandbox_world.resources.character;
    state.velocity = Vec3::zeros();
    state.grounded = false;
    state.jump_suppress_timer = 0.0;
}

fn spawn_feet_position(world: &World) -> Vec3 {
    let spawn = find_player_spawn(world).unwrap_or_else(default_spawn_position);
    spawn - Vec3::new(0.0, FIRST_PERSON_CAPSULE_HALF_HEIGHT, 0.0)
}

struct CharacterInputs {
    direction: Vec3,
    jump_pressed: bool,
}

fn read_inputs(world: &mut World, camera: Entity) -> CharacterInputs {
    let (forward, right) = horizontal_camera_basis(world, camera);

    let mut direction = Vec3::zeros();
    let jump_key_pressed = {
        let keyboard = &world.resources.input.keyboard;
        if keyboard.is_key_pressed(KeyCode::KeyW) {
            direction += forward;
        }
        if keyboard.is_key_pressed(KeyCode::KeyS) {
            direction -= forward;
        }
        if keyboard.is_key_pressed(KeyCode::KeyD) {
            direction += right;
        }
        if keyboard.is_key_pressed(KeyCode::KeyA) {
            direction -= right;
        }
        keyboard.just_pressed(KeyCode::Space)
    };

    if let Some(gamepad) = query_active_gamepad(world) {
        let stick_x = gamepad.value(gilrs::Axis::LeftStickX);
        let stick_y = gamepad.value(gilrs::Axis::LeftStickY);
        let magnitude = (stick_x * stick_x + stick_y * stick_y).sqrt();
        if magnitude > GAMEPAD_DEADZONE {
            let normalized = (magnitude - GAMEPAD_DEADZONE) / (1.0 - GAMEPAD_DEADZONE);
            direction += forward * (stick_y * normalized / magnitude)
                + right * (stick_x * normalized / magnitude);
        }
    }

    if direction.norm() > 1.0 {
        direction = direction.normalize();
    }

    let gamepad_jump = world
        .resources
        .input
        .gamepad
        .just_pressed_buttons
        .contains(&gilrs::Button::South);

    CharacterInputs {
        direction,
        jump_pressed: jump_key_pressed || gamepad_jump,
    }
}

fn horizontal_camera_basis(world: &World, camera: Entity) -> (Vec3, Vec3) {
    let Some(transform) = world.core.get_local_transform(camera) else {
        return (Vec3::new(0.0, 0.0, -1.0), Vec3::new(1.0, 0.0, 0.0));
    };
    let forward = transform.forward_vector();
    let right = transform.right_vector();
    let forward_flat = Vec3::new(forward.x, 0.0, forward.z);
    let right_flat = Vec3::new(right.x, 0.0, right.z);
    let forward_normalized = if forward_flat.norm() > 1e-4 {
        forward_flat.normalize()
    } else {
        Vec3::new(0.0, 0.0, -1.0)
    };
    let right_normalized = if right_flat.norm() > 1e-4 {
        right_flat.normalize()
    } else {
        Vec3::new(1.0, 0.0, 0.0)
    };
    (forward_normalized, right_normalized)
}

fn move_character(
    sandbox_world: &mut SandboxWorld,
    world: &mut World,
    body: Entity,
    camera: Entity,
) {
    let delta_time = world
        .resources
        .window
        .timing
        .delta_time
        .min(MAX_STEP_SECONDS);
    let Some(mut position) = world
        .core
        .get_local_transform(body)
        .map(|transform| transform.translation)
    else {
        return;
    };

    let inputs = read_inputs(world, camera);
    let gravity_y = world.resources.physics.gravity.y;

    let state = &mut sandbox_world.resources.character;
    let mut velocity = state.velocity;

    let target_velocity = inputs.direction * MAX_GROUND_SPEED;
    let acceleration = if state.grounded {
        GROUND_ACCELERATION
    } else {
        AIR_ACCELERATION
    };
    let velocity_error = Vec3::new(
        target_velocity.x - velocity.x,
        0.0,
        target_velocity.z - velocity.z,
    );
    let mut lean_target = velocity_error * acceleration * LEAN_RADIANS_PER_ACCELERATION;
    if lean_target.norm() > LEAN_MAX_RADIANS {
        lean_target = lean_target.normalize() * LEAN_MAX_RADIANS;
    }
    let lean_blend = 1.0 - (-LEAN_SHARPNESS * delta_time).exp();
    state.lean += (lean_target - state.lean) * lean_blend;
    let blend = (acceleration * delta_time).min(1.0);
    velocity.x += (target_velocity.x - velocity.x) * blend;
    velocity.z += (target_velocity.z - velocity.z) * blend;
    if state.grounded && inputs.direction.norm() < 0.01 {
        let friction = 1.0 - (GROUND_FRICTION * delta_time).min(1.0);
        velocity.x *= friction;
        velocity.z *= friction;
    }

    velocity.y += gravity_y * delta_time;

    state.jump_suppress_timer = (state.jump_suppress_timer - delta_time).max(0.0);
    if inputs.jump_pressed && state.grounded {
        velocity.y = JUMP_IMPULSE;
        state.jump_suppress_timer = JUMP_SPRING_SUPPRESS_SECONDS;
        state.grounded = false;
    }

    let probe_origin = position + Vec3::new(0.0, RIDE_PROBE_HEIGHT, 0.0);
    let ground_distance = sample_ground_distance(world, probe_origin);
    if let Some(distance) = ground_distance
        && state.jump_suppress_timer <= 0.0
    {
        let displacement = RIDE_PROBE_HEIGHT - distance;
        let spring_acceleration =
            displacement * LIFTER_SPRING_STIFFNESS - velocity.y * LIFTER_SPRING_DAMPING;
        velocity.y += spring_acceleration * delta_time;
        state.grounded = distance < RIDE_PROBE_HEIGHT + GROUNDED_MARGIN;
    } else {
        state.grounded = false;
    }

    position += velocity * delta_time;

    for _ in 0..BUMPER_RESOLVE_ITERATIONS {
        let contacts = physics_world_capsule_contacts(
            &world.resources.physics,
            position + Vec3::new(0.0, BUMPER_CENTER_HEIGHT, 0.0),
            BUMPER_HALF_HEIGHT,
            BUMPER_RADIUS,
            None,
        );
        if contacts.is_empty() {
            break;
        }
        for contact in contacts {
            position += contact.normal * contact.depth;
            let into_surface = velocity.dot(&contact.normal);
            if into_surface < 0.0 {
                velocity -= contact.normal * into_surface;
            }
        }
    }

    let horizontal_velocity = Vec3::new(velocity.x, 0.0, velocity.z);
    if horizontal_velocity.norm() > FACING_SPEED_THRESHOLD {
        let target_yaw = horizontal_velocity.x.atan2(horizontal_velocity.z);
        let difference = (target_yaw - state.facing_yaw + std::f32::consts::PI)
            .rem_euclid(std::f32::consts::TAU)
            - std::f32::consts::PI;
        state.facing_yaw += difference * (1.0 - (-FACING_TURN_SHARPNESS * delta_time).exp());
    }
    if state.grounded {
        state.stride_distance += horizontal_velocity.norm() * delta_time;
    }
    let facing_yaw = state.facing_yaw;
    let lean = state.lean;

    state.velocity = velocity;
    if let Some(transform) = world.core.get_local_transform_mut(body) {
        transform.translation = position;
        let yaw_rotation = nalgebra_glm::quat_angle_axis(facing_yaw, &Vec3::y());
        let lean_angle = lean.norm();
        transform.rotation = if lean_angle > 1e-4 {
            let lean_axis = nalgebra_glm::cross(&Vec3::y(), &(lean / lean_angle));
            nalgebra_glm::quat_angle_axis(lean_angle, &lean_axis) * yaw_rotation
        } else {
            yaw_rotation
        };
    }
    mark_local_transform_dirty(world, body);
}

fn sample_ground_distance(world: &World, probe_origin: Vec3) -> Option<f32> {
    let probe_radius = LIFTER_RADIUS * 0.6;
    let offsets = [
        Vec3::zeros(),
        Vec3::new(probe_radius, 0.0, 0.0),
        Vec3::new(-probe_radius, 0.0, 0.0),
        Vec3::new(0.0, 0.0, probe_radius),
        Vec3::new(0.0, 0.0, -probe_radius),
    ];
    let max_distance = RIDE_PROBE_HEIGHT + LIFTER_PROBE_DISTANCE;
    let mut nearest: Option<f32> = None;
    for offset in offsets {
        if let Some(hit) = physics_world_cast_ray(
            &world.resources.physics,
            probe_origin + offset,
            Vec3::new(0.0, -1.0, 0.0),
            max_distance,
            None,
        ) {
            nearest = Some(nearest.map_or(hit.distance, |current| current.min(hit.distance)));
        }
    }
    nearest
}

fn sync_first_person_player(sandbox_world: &SandboxWorld, world: &mut World, body: Entity) {
    let Some(player_entity) = sandbox_world.resources.player.player_entity else {
        return;
    };
    let Some(feet) = world
        .core
        .get_local_transform(body)
        .map(|transform| transform.translation)
    else {
        return;
    };
    let capsule_center = feet + Vec3::new(0.0, FIRST_PERSON_CAPSULE_HALF_HEIGHT, 0.0);
    if let Some(transform) = world.core.get_local_transform_mut(player_entity) {
        transform.translation = capsule_center;
    }
    if let Some(interpolation) = world.core.get_physics_interpolation_mut(player_entity) {
        interpolation.previous_translation = capsule_center;
        interpolation.current_translation = capsule_center;
    }
    mark_local_transform_dirty(world, player_entity);
}

fn draw_spheres(sandbox_world: &SandboxWorld, world: &mut World, body: Entity) {
    let Some(spheres) = sandbox_world.resources.character.spheres else {
        return;
    };
    let Some(feet) = world
        .core
        .get_local_transform(body)
        .map(|transform| transform.translation)
    else {
        return;
    };
    let Some(lines) = world.core.get_lines_mut(spheres) else {
        return;
    };
    lines.lines.clear();
    let state = &sandbox_world.resources.character;
    if state.spheres_visible {
        push_wire_capsule(
            &mut lines.lines,
            feet + Vec3::new(0.0, BUMPER_CENTER_HEIGHT, 0.0),
            BUMPER_HALF_HEIGHT,
            BUMPER_RADIUS,
            BUMPER_COLOR,
        );
        push_wire_sphere(
            &mut lines.lines,
            feet + Vec3::new(0.0, LIFTER_CENTER_HEIGHT, 0.0),
            LIFTER_RADIUS,
            LIFTER_COLOR,
        );
    }
    if state.stride_wheel_visible {
        push_stride_wheel(
            &mut lines.lines,
            feet,
            state.facing_yaw,
            state.stride_distance,
        );
    }
    lines.mark_dirty();
}

fn push_stride_wheel(lines: &mut Vec<Line>, feet: Vec3, facing_yaw: f32, stride_distance: f32) {
    let forward = Vec3::new(facing_yaw.sin(), 0.0, facing_yaw.cos());
    let center = feet + Vec3::new(0.0, STRIDE_WHEEL_RADIUS, 0.0);
    push_circle(
        lines,
        center,
        STRIDE_WHEEL_RADIUS,
        forward,
        Vec3::y(),
        STRIDE_WHEEL_COLOR,
    );
    let phase = stride_distance / STRIDE_WHEEL_RADIUS;
    for spoke_index in 0..STRIDE_WHEEL_SPOKES {
        let angle =
            (spoke_index as f32 / STRIDE_WHEEL_SPOKES as f32) * std::f32::consts::TAU - phase;
        let rim = center
            + forward * angle.cos() * STRIDE_WHEEL_RADIUS
            + Vec3::new(0.0, angle.sin() * STRIDE_WHEEL_RADIUS, 0.0);
        lines.push(Line {
            start: center,
            end: rim,
            color: STRIDE_WHEEL_COLOR,
        });
    }
}

fn push_wire_sphere(lines: &mut Vec<Line>, center: Vec3, radius: f32, color: Vec4) {
    for ring_index in 1..SPHERE_LATITUDE_RINGS {
        let polar_angle = (ring_index as f32 / SPHERE_LATITUDE_RINGS as f32) * std::f32::consts::PI;
        let ring_center = center + Vec3::new(0.0, polar_angle.cos() * radius, 0.0);
        let ring_radius = polar_angle.sin() * radius;
        push_circle(lines, ring_center, ring_radius, Vec3::x(), Vec3::z(), color);
    }
    for meridian_index in 0..SPHERE_MERIDIAN_CIRCLES {
        let meridian_angle =
            (meridian_index as f32 / SPHERE_MERIDIAN_CIRCLES as f32) * std::f32::consts::PI;
        let meridian_axis = Vec3::new(meridian_angle.cos(), 0.0, meridian_angle.sin());
        push_circle(lines, center, radius, meridian_axis, Vec3::y(), color);
    }
}

fn push_wire_capsule(
    lines: &mut Vec<Line>,
    center: Vec3,
    half_height: f32,
    radius: f32,
    color: Vec4,
) {
    let top_center = center + Vec3::new(0.0, half_height, 0.0);
    let bottom_center = center - Vec3::new(0.0, half_height, 0.0);
    for ring_index in 1..SPHERE_LATITUDE_RINGS {
        if ring_index * 2 == SPHERE_LATITUDE_RINGS {
            continue;
        }
        let polar_angle = (ring_index as f32 / SPHERE_LATITUDE_RINGS as f32) * std::f32::consts::PI;
        let hemisphere_center = if polar_angle < std::f32::consts::FRAC_PI_2 {
            top_center
        } else {
            bottom_center
        };
        let ring_center = hemisphere_center + Vec3::new(0.0, polar_angle.cos() * radius, 0.0);
        let ring_radius = polar_angle.sin() * radius;
        push_circle(lines, ring_center, ring_radius, Vec3::x(), Vec3::z(), color);
    }
    push_circle(lines, top_center, radius, Vec3::x(), Vec3::z(), color);
    push_circle(lines, bottom_center, radius, Vec3::x(), Vec3::z(), color);
    for meridian_index in 0..SPHERE_MERIDIAN_CIRCLES {
        let meridian_angle =
            (meridian_index as f32 / SPHERE_MERIDIAN_CIRCLES as f32) * std::f32::consts::PI;
        let meridian_axis = Vec3::new(meridian_angle.cos(), 0.0, meridian_angle.sin());
        push_semicircle(lines, top_center, radius, meridian_axis, 1.0, color);
        push_semicircle(lines, bottom_center, radius, meridian_axis, -1.0, color);
        for side in [meridian_axis, -meridian_axis] {
            lines.push(Line {
                start: top_center + side * radius,
                end: bottom_center + side * radius,
                color,
            });
        }
    }
}

fn push_semicircle(
    lines: &mut Vec<Line>,
    center: Vec3,
    radius: f32,
    axis: Vec3,
    vertical_sign: f32,
    color: Vec4,
) {
    let segment_count = SPHERE_SEGMENTS / 2;
    for segment_index in 0..segment_count {
        let angle_start = (segment_index as f32 / segment_count as f32) * std::f32::consts::PI;
        let angle_end = ((segment_index + 1) as f32 / segment_count as f32) * std::f32::consts::PI;
        let start = center
            + axis * angle_start.cos() * radius
            + Vec3::new(0.0, angle_start.sin() * radius * vertical_sign, 0.0);
        let end = center
            + axis * angle_end.cos() * radius
            + Vec3::new(0.0, angle_end.sin() * radius * vertical_sign, 0.0);
        lines.push(Line { start, end, color });
    }
}

fn push_circle(
    lines: &mut Vec<Line>,
    center: Vec3,
    radius: f32,
    axis_a: Vec3,
    axis_b: Vec3,
    color: Vec4,
) {
    for segment_index in 0..SPHERE_SEGMENTS {
        let angle_start = (segment_index as f32 / SPHERE_SEGMENTS as f32) * std::f32::consts::TAU;
        let angle_end =
            ((segment_index + 1) as f32 / SPHERE_SEGMENTS as f32) * std::f32::consts::TAU;
        let start =
            center + axis_a * angle_start.cos() * radius + axis_b * angle_start.sin() * radius;
        let end = center + axis_a * angle_end.cos() * radius + axis_b * angle_end.sin() * radius;
        lines.push(Line { start, end, color });
    }
}
