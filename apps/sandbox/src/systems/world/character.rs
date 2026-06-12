use crate::ecs::SandboxWorld;
use crate::systems::world::level::{default_spawn_position, find_player_spawn};
use nightshade::ecs::input::queries::query_active_gamepad;
use nightshade::ecs::lines::components::{Line, Lines};
use nightshade::ecs::world::{
    GLOBAL_TRANSFORM, LINES, LOCAL_TRANSFORM, LOCAL_TRANSFORM_DIRTY, NAME, VISIBILITY,
};
use nightshade::prelude::*;

const BUMPER_RADIUS: f32 = 0.55;
const BUMPER_CENTER_HEIGHT: f32 = 1.0;
const LIFTER_RADIUS: f32 = 0.45;
const LIFTER_CENTER_HEIGHT: f32 = 0.45;
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
const MAX_STEP_SECONDS: f32 = 1.0 / 30.0;
const BUMPER_RESOLVE_ITERATIONS: usize = 3;
const SPHERE_SEGMENTS: usize = 24;
const SPHERE_LATITUDE_RINGS: usize = 12;
const SPHERE_MERIDIAN_CIRCLES: usize = 12;
const BUMPER_COLOR: Vec4 = Vec4::new(0.2, 1.0, 0.3, 1.0);
const LIFTER_COLOR: Vec4 = Vec4::new(1.0, 1.0, 1.0, 1.0);

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

    let bumper_center = position + Vec3::new(0.0, BUMPER_CENTER_HEIGHT, 0.0);
    let ground_distance = sample_ground_distance(world, bumper_center);
    if let Some(distance) = ground_distance
        && state.jump_suppress_timer <= 0.0
    {
        let displacement = BUMPER_CENTER_HEIGHT - distance;
        let spring_acceleration =
            displacement * LIFTER_SPRING_STIFFNESS - velocity.y * LIFTER_SPRING_DAMPING;
        velocity.y += spring_acceleration * delta_time;
        state.grounded = distance < BUMPER_CENTER_HEIGHT + GROUNDED_MARGIN;
    } else {
        state.grounded = false;
    }

    position += velocity * delta_time;

    for _ in 0..BUMPER_RESOLVE_ITERATIONS {
        let contacts = physics_world_sphere_contacts(
            &world.resources.physics,
            position + Vec3::new(0.0, BUMPER_CENTER_HEIGHT, 0.0),
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

    state.velocity = velocity;
    if let Some(transform) = world.core.get_local_transform_mut(body) {
        transform.translation = position;
    }
    mark_local_transform_dirty(world, body);
}

fn sample_ground_distance(world: &World, bumper_center: Vec3) -> Option<f32> {
    let probe_radius = LIFTER_RADIUS * 0.6;
    let offsets = [
        Vec3::zeros(),
        Vec3::new(probe_radius, 0.0, 0.0),
        Vec3::new(-probe_radius, 0.0, 0.0),
        Vec3::new(0.0, 0.0, probe_radius),
        Vec3::new(0.0, 0.0, -probe_radius),
    ];
    let max_distance = BUMPER_CENTER_HEIGHT + LIFTER_PROBE_DISTANCE;
    let mut nearest: Option<f32> = None;
    for offset in offsets {
        if let Some(hit) = physics_world_cast_ray(
            &world.resources.physics,
            bumper_center + offset,
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
    push_wire_sphere(
        &mut lines.lines,
        feet + Vec3::new(0.0, BUMPER_CENTER_HEIGHT, 0.0),
        BUMPER_RADIUS,
        BUMPER_COLOR,
    );
    push_wire_sphere(
        &mut lines.lines,
        feet + Vec3::new(0.0, LIFTER_CENTER_HEIGHT, 0.0),
        LIFTER_RADIUS,
        LIFTER_COLOR,
    );
    lines.mark_dirty();
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
