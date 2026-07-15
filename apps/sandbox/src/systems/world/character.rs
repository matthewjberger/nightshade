use crate::ecs::{CharacterState, RagdollMode, SandboxResources};
use crate::systems::world::level::{default_spawn_position, find_player_spawn};
use nightshade::ecs::lines::components::{Line, Lines};
use nightshade::plugins::gamepad::query_active_gamepad;
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
const MAX_GROUND_SPEED: f32 = 5.5;
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
const LEAN_RADIANS_PER_ACCELERATION: f32 = 0.0008;
const LEAN_MAX_RADIANS: f32 = 0.22;
const LEAN_SHARPNESS: f32 = 10.0;
const MAX_STEP_SECONDS: f32 = 1.0 / 30.0;
const BUMPER_RESOLVE_ITERATIONS: usize = 3;
const SPHERE_SEGMENTS: usize = 24;
const SPHERE_LATITUDE_RINGS: usize = 12;
const SPHERE_MERIDIAN_CIRCLES: usize = 12;
const RUN_WHEEL_RADIUS: f32 = 0.55;
const WALK_WHEEL_RADIUS: f32 = 0.24;
const CROUCH_WHEEL_RADIUS: f32 = 0.22;
const WALK_FULL_SPEED: f32 = 1.6;
const RUN_FULL_SPEED: f32 = 4.5;
const GAIT_FADE_START_SPEED: f32 = 0.15;
const GAIT_FULL_SPEED: f32 = 1.0;
const GAIT_WEIGHT_SHARPNESS: f32 = 8.0;
const GAIT_BLEND_SHARPNESS: f32 = 6.0;
const BOUNCE_WALK_MAX: f32 = 0.045;
const BOUNCE_RUN_MAX: f32 = 0.16;
const BOUNCE_ABSORB: f32 = 0.05;
const BOUNCE_CONTACT_LAG: f32 = 0.3;
const CROUCH_SPRING_STIFFNESS: f32 = 90.0;
const CROUCH_SPRING_DAMPING: f32 = 13.0;
const CROUCH_SPEED_PENALTY: f32 = 0.65;
const LANDING_CROUCH_FACTOR: f32 = 0.18;
const AIR_POSE_SHARPNESS: f32 = 6.0;
const FLIP_DURATION: f32 = 0.75;
const ROLL_DURATION: f32 = 0.7;
const FLIP_PIVOT_HEIGHT: f32 = 1.0;
const ROLL_PIVOT_HEIGHT: f32 = 0.55;
const FLIP_ANTICIPATION_RADIANS: f32 = 0.2;
const FLIP_ANTICIPATION_FRACTION: f32 = 0.12;
const ROLL_MINIMUM_SPEED: f32 = 2.0;
const WALL_RUN_MAX_SECONDS: f32 = 1.2;
const WALL_RUN_COOLDOWN_SECONDS: f32 = 0.4;
const WALL_RUN_MINIMUM_SPEED: f32 = 3.0;
const WALL_RUN_FALL_RATE: f32 = -1.6;
const WALL_RUN_GRAVITY_SCALE: f32 = 0.3;
const WALL_RUN_STICK: f32 = 2.5;
const WALL_RUN_LEAN: f32 = 0.35;
const WALL_NORMAL_MAX_Y: f32 = 0.35;
const WALL_JUMP_AWAY: f32 = 4.5;
const WALL_JUMP_UP: f32 = 4.5;
const HANG_REACH_MINIMUM: f32 = 1.25;
const HANG_REACH_MAXIMUM: f32 = 2.05;
const HANG_PROBE_RISE: f32 = 2.2;
const HANG_PROBE_FORWARD: f32 = 0.55;
const HANG_PROBE_DOWN: f32 = 1.2;
const HANG_DEPTH: f32 = 1.7;
const HANG_MAX_FALL_SPEED: f32 = 1.0;
const SHIMMY_SPEED: f32 = 0.8;
const HANG_POSE_SHARPNESS: f32 = 10.0;
const CLIMB_UP_SPEED: f32 = 5.5;
const CLIMB_FORWARD_SPEED: f32 = 2.0;
const LOOK_YAW_LIMIT: f32 = 1.05;
const LOOK_PITCH_LIMIT: f32 = 0.6;
const LOOK_SHARPNESS: f32 = 8.0;
const LOOK_FADE_FORWARDNESS: f32 = 0.3;
const RAGDOLL_IMPACT_SPEED: f32 = 9.0;
const STRIDE_WHEEL_SPOKES: usize = 8;
const BUMPER_COLOR: Vec4 = Vec4::new(0.2, 1.0, 0.3, 1.0);
const LIFTER_COLOR: Vec4 = Vec4::new(1.0, 1.0, 1.0, 1.0);
const STRIDE_WHEEL_COLOR: Vec4 = Vec4::new(1.0, 0.85, 0.2, 1.0);
const SKELETON_BONE_COLOR: Vec4 = Vec4::new(1.0, 0.4, 0.9, 1.0);
const SKELETON_JOINT_COLOR: Vec4 = Vec4::new(0.3, 0.9, 1.0, 1.0);
const SKELETON_JOINT_SIZE: f32 = 0.04;

pub fn spawn(sandbox_world: &mut SandboxResources, world: &mut World) {
    let feet = spawn_feet_position(world);

    let body = world.spawn_with((
        Name("Character".to_string()),
        LocalTransform {
            translation: feet,
            ..Default::default()
        },
        GlobalTransform::default(),
    ));

    let camera = spawn_third_person_camera(
        world,
        body,
        CAMERA_DISTANCE,
        0.0,
        CAMERA_PITCH,
        "Character Camera".to_string(),
    );

    let spheres = world.spawn_with((
        Name("Character Spheres".to_string()),
        Lines::new(Vec::new()),
        Visibility { visible: false },
        LocalTransform::default(),
        GlobalTransform::default(),
    ));

    let skeleton_lines = world.spawn_with((
        Name("Character Skeleton".to_string()),
        Lines::new_always_on_top(Vec::new()),
        Visibility { visible: false },
        LocalTransform::default(),
        GlobalTransform::default(),
    ));

    let state = &mut sandbox_world.character;
    state.body = Some(body);
    state.camera = Some(camera);
    state.spheres = Some(spheres);
    state.skeleton_lines = Some(skeleton_lines);
    state.stride_wheel_visible = true;
    state.stride_radius = WALK_WHEEL_RADIUS;

    spawn_model(sandbox_world, world, body);
}

fn spawn_model(sandbox_world: &mut SandboxResources, world: &mut World, body: Entity) {
    const MODEL_BYTES: &[u8] = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../assets/gltf/x_bot.glb"
    ));
    let mut result = match nightshade::ecs::prefab::import_gltf_from_bytes(MODEL_BYTES) {
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
    if let Some(name) = world.get_mut::<nightshade::ecs::primitives::Name>(model) {
        name.0 = "Character Model".to_string();
    }
    world.ecs.worlds[CORE].add_components(model, nightshade::ecs::world::PARENT);
    world.set(model, nightshade::ecs::transform::components::ChildOf(body));
    if let Some(transform) =
        world.get_mut::<nightshade::ecs::transform::components::LocalTransform>(model)
    {
        transform.translation = Vec3::zeros();
    }

    if let Some(player) =
        world.get_mut::<nightshade::ecs::animation::components::AnimationPlayer>(model)
    {
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

    sandbox_world.character.model = Some(model);
    crate::systems::world::pose::initialize(sandbox_world, world, model);
    crate::systems::world::pose::set_pose(sandbox_world, world, "idle");
}

fn set_model_visible(sandbox_world: &SandboxResources, world: &mut World, visible: bool) {
    let Some(model) = sandbox_world.character.model else {
        return;
    };
    let mut stack = vec![model];
    while let Some(entity) = stack.pop() {
        if let Some(visibility) = world.get_mut::<nightshade::ecs::primitives::Visibility>(entity) {
            visibility.visible = visible;
        }
        stack.extend(nightshade::ecs::transform::queries::query_children(
            world, entity,
        ));
    }
}

pub fn set_third_person(sandbox_world: &mut SandboxResources, world: &mut World, enabled: bool) {
    sandbox_world.character.third_person = enabled;
    let Some(body) = sandbox_world.character.body else {
        return;
    };
    let Some(player_entity) = sandbox_world.player.player_entity else {
        return;
    };
    let fly = sandbox_world.player.fly;

    if enabled {
        let feet = world
            .get::<nightshade::ecs::transform::components::LocalTransform>(player_entity)
            .map(|transform| {
                transform.translation - Vec3::new(0.0, FIRST_PERSON_CAPSULE_HALF_HEIGHT, 0.0)
            })
            .unwrap_or_else(|| spawn_feet_position(world));
        if let Some(transform) =
            world.get_mut::<nightshade::ecs::transform::components::LocalTransform>(body)
        {
            transform.translation = feet;
        }

        let state = &mut sandbox_world.character;
        state.velocity = Vec3::zeros();
        state.grounded = false;
        state.jump_suppress_timer = 0.0;

        if let Some(controller) =
            world.get_mut::<nightshade::plugins::physics::components::CharacterControllerComponent>(
                player_entity,
            )
        {
            controller.disable_gravity = true;
            controller.velocity = Vec3::zeros();
        }
        if let Some(spheres) = sandbox_world.character.spheres
            && let Some(visibility) =
                world.get_mut::<nightshade::ecs::primitives::Visibility>(spheres)
        {
            visibility.visible = true;
        }
        set_model_visible(sandbox_world, world, true);
        if !fly {
            world.resources.active_camera = sandbox_world.character.camera;
        }
    } else {
        if let Some(controller) =
            world.get_mut::<nightshade::plugins::physics::components::CharacterControllerComponent>(
                player_entity,
            )
        {
            controller.disable_gravity = false;
            controller.velocity = Vec3::zeros();
        }
        reset_physics_interpolation(world, player_entity);
        if let Some(spheres) = sandbox_world.character.spheres
            && let Some(visibility) =
                world.get_mut::<nightshade::ecs::primitives::Visibility>(spheres)
        {
            visibility.visible = false;
        }
        if let Some(skeleton_lines) = sandbox_world.character.skeleton_lines
            && let Some(visibility) =
                world.get_mut::<nightshade::ecs::primitives::Visibility>(skeleton_lines)
        {
            visibility.visible = false;
        }
        set_model_visible(sandbox_world, world, false);
        if !fly {
            world.resources.active_camera = sandbox_world.player.camera_entity;
        }
    }
}

pub fn update(sandbox_world: &mut SandboxResources, world: &mut World) {
    if !sandbox_world.character.third_person {
        return;
    }
    let Some(body) = sandbox_world.character.body else {
        return;
    };
    let Some(camera) = sandbox_world.character.camera else {
        return;
    };

    if sandbox_world.ragdoll.mode != RagdollMode::Off {
        crate::systems::world::ragdoll::update(sandbox_world, world, body);
        sync_first_person_player(sandbox_world, world, body);
        third_person_camera_system(world);
        draw_spheres(sandbox_world, world, body);
        draw_skeleton(sandbox_world, world);
        return;
    }

    move_character(sandbox_world, world, body, camera);
    if sandbox_world.character.landing_impact_speed > RAGDOLL_IMPACT_SPEED {
        sandbox_world.character.landing_impact_speed = 0.0;
        crate::systems::world::ragdoll::activate_from_impact(sandbox_world, world);
        sync_first_person_player(sandbox_world, world, body);
        third_person_camera_system(world);
        draw_spheres(sandbox_world, world, body);
        draw_skeleton(sandbox_world, world);
        return;
    }
    apply_model_motion(sandbox_world, world);
    sync_first_person_player(sandbox_world, world, body);
    third_person_camera_system(world);
    draw_spheres(sandbox_world, world, body);
    draw_skeleton(sandbox_world, world);
}

/// Vertical bob synced to the stride plus the flip and roll rotation
/// of the whole model around its center of mass. Each step is a
/// ballistic arc that floats to an apex at mid-flight and dips into a
/// short absorb at contact, and its height grows from a gentle walk to
/// a weighty run. Flips rotate with a small anticipation wind up and
/// ease out into the air pose.
fn apply_model_motion(sandbox_world: &SandboxResources, world: &mut World) {
    let state = &sandbox_world.character;
    let Some(model) = state.model else {
        return;
    };
    let run_factor = state.gait_run_factor.clamp(0.0, 1.0);
    let apex =
        (BOUNCE_WALK_MAX + (BOUNCE_RUN_MAX - BOUNCE_WALK_MAX) * run_factor) * state.gait_weight;
    let step_phase = (state.stride_phase - BOUNCE_CONTACT_LAG).rem_euclid(std::f32::consts::PI)
        / std::f32::consts::PI;
    let arc = 4.0 * step_phase * (1.0 - step_phase);
    let absorb = BOUNCE_ABSORB * run_factor * state.gait_weight * (1.0 - arc).powi(3);
    let bounce = apex * arc - absorb;

    let mut rotation = nalgebra_glm::quat_identity();
    let mut translation = Vec3::new(0.0, bounce, 0.0);
    if state.flip_duration > 0.0 && state.flip_axis.norm() > 0.5 {
        let progress = (state.flip_time / state.flip_duration).clamp(0.0, 1.0);
        let angle = flip_angle(progress);
        rotation = nalgebra_glm::quat_angle_axis(angle, &state.flip_axis);
        let pivot = Vec3::new(0.0, state.flip_pivot, 0.0);
        translation += pivot - nalgebra_glm::quat_rotate_vec3(&rotation, &pivot);
    }

    if let Some(transform) =
        world.get_mut::<nightshade::ecs::transform::components::LocalTransform>(model)
    {
        transform.translation = translation;
        transform.rotation = rotation;
    }
}

fn flip_angle(progress: f32) -> f32 {
    let anticipation = -FLIP_ANTICIPATION_RADIANS
        * (std::f32::consts::PI * (progress / FLIP_ANTICIPATION_FRACTION).min(1.0)).sin();
    let main_progress = ((progress - FLIP_ANTICIPATION_FRACTION)
        / (1.0 - FLIP_ANTICIPATION_FRACTION))
        .clamp(0.0, 1.0);
    let eased = main_progress * main_progress * (3.0 - 2.0 * main_progress);
    anticipation + std::f32::consts::TAU * eased
}

fn draw_skeleton(sandbox_world: &SandboxResources, world: &mut World) {
    let state = &sandbox_world.character;
    let Some(skeleton_lines) = state.skeleton_lines else {
        return;
    };
    if let Some(visibility) =
        world.get_mut::<nightshade::ecs::primitives::Visibility>(skeleton_lines)
    {
        visibility.visible = state.skeleton_visible;
    }
    if !state.skeleton_visible {
        return;
    }

    let skin_owners: Vec<Entity> = world.ecs.worlds[CORE]
        .query_entities(nightshade::ecs::world::SKIN)
        .collect();
    let mut bone_lines = Vec::new();
    for owner in skin_owners {
        let Some(skin) = world.get::<nightshade::ecs::skin::components::Skin>(owner) else {
            continue;
        };
        let joint_set: std::collections::HashSet<Entity> = skin.joints.iter().copied().collect();
        for &joint_entity in &skin.joints {
            let Some(joint_transform) =
                world.get::<nightshade::ecs::transform::components::GlobalTransform>(joint_entity)
            else {
                continue;
            };
            let joint_position = joint_transform.translation();
            push_joint_cross(&mut bone_lines, joint_position);

            let Some(parent) = world
                .get::<freecs::dynamic::ChildOf>(joint_entity)
                .map(|child_of| child_of.0)
            else {
                continue;
            };
            if !joint_set.contains(&parent) {
                continue;
            }
            let Some(parent_transform) =
                world.get::<nightshade::ecs::transform::components::GlobalTransform>(parent)
            else {
                continue;
            };
            bone_lines.push(Line {
                start: parent_transform.translation(),
                end: joint_position,
                color: SKELETON_BONE_COLOR,
            });
        }
    }

    if let Some(lines) = world.get_mut::<nightshade::ecs::lines::components::Lines>(skeleton_lines)
    {
        lines.lines = bone_lines;
        lines.mark_dirty();
    }
}

fn push_joint_cross(lines: &mut Vec<Line>, position: Vec3) {
    let half = SKELETON_JOINT_SIZE * 0.5;
    for axis in [Vec3::x(), Vec3::y(), Vec3::z()] {
        lines.push(Line {
            start: position - axis * half,
            end: position + axis * half,
            color: SKELETON_JOINT_COLOR,
        });
    }
}

pub fn reset(sandbox_world: &mut SandboxResources, world: &mut World) {
    let spawn = find_player_spawn(world).unwrap_or_else(default_spawn_position);
    teleport(sandbox_world, world, spawn);
}

pub fn teleport(sandbox_world: &mut SandboxResources, world: &mut World, capsule_center: Vec3) {
    let Some(body) = sandbox_world.character.body else {
        return;
    };
    let feet = capsule_center - Vec3::new(0.0, FIRST_PERSON_CAPSULE_HALF_HEIGHT, 0.0);
    if let Some(transform) =
        world.get_mut::<nightshade::ecs::transform::components::LocalTransform>(body)
    {
        transform.translation = feet;
    }

    let state = &mut sandbox_world.character;
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
    crouch_held: bool,
    flip_pressed: bool,
}

fn read_inputs(world: &mut World, camera: Entity) -> CharacterInputs {
    let (forward, right) = horizontal_camera_basis(world, camera);

    let mut direction = Vec3::zeros();
    let (jump_key_pressed, crouch_key_held) = {
        let keyboard = &world
            .expect_resource::<nightshade::ecs::input::resources::Input>()
            .keyboard;
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
        (
            keyboard.just_pressed(KeyCode::Space),
            keyboard.is_key_pressed(KeyCode::ControlLeft) || keyboard.is_key_pressed(KeyCode::KeyC),
        )
    };

    let mut gamepad_crouch_held = false;
    if let Some(gamepad) = query_active_gamepad(world) {
        let stick_x = gamepad.value(gilrs::Axis::LeftStickX);
        let stick_y = gamepad.value(gilrs::Axis::LeftStickY);
        let magnitude = (stick_x * stick_x + stick_y * stick_y).sqrt();
        if magnitude > GAMEPAD_DEADZONE {
            let normalized = (magnitude - GAMEPAD_DEADZONE) / (1.0 - GAMEPAD_DEADZONE);
            direction += forward * (stick_y * normalized / magnitude)
                + right * (stick_x * normalized / magnitude);
        }
        gamepad_crouch_held = gamepad.is_pressed(gilrs::Button::East);
    }

    if direction.norm() > 1.0 {
        direction = direction.normalize();
    }

    let gamepad_jump = world
        .resources
        .gamepad
        .just_pressed_buttons
        .contains(&gilrs::Button::South);
    let gamepad_flip = world
        .resources
        .gamepad
        .just_pressed_buttons
        .contains(&gilrs::Button::West);
    let flip_key_pressed = world
        .expect_resource::<nightshade::ecs::input::resources::Input>()
        .keyboard
        .just_pressed(KeyCode::KeyF);

    CharacterInputs {
        direction,
        jump_pressed: jump_key_pressed || gamepad_jump,
        crouch_held: crouch_key_held || gamepad_crouch_held,
        flip_pressed: flip_key_pressed || gamepad_flip,
    }
}

fn horizontal_camera_basis(world: &World, camera: Entity) -> (Vec3, Vec3) {
    let Some(transform) =
        world.get::<nightshade::ecs::transform::components::LocalTransform>(camera)
    else {
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
    sandbox_world: &mut SandboxResources,
    world: &mut World,
    body: Entity,
    camera: Entity,
) {
    let delta_time = world
        .expect_resource_mut::<nightshade::ecs::time::Time>()
        .delta_time
        .min(MAX_STEP_SECONDS);
    let Some(mut position) = world
        .get::<nightshade::ecs::transform::components::LocalTransform>(body)
        .map(|transform| transform.translation)
    else {
        return;
    };

    let inputs = read_inputs(world, camera);
    let gravity_y = world.plugin_resource::<PhysicsWorld>().gravity.y;
    let camera_aim = world
        .get::<nightshade::ecs::transform::components::LocalTransform>(camera)
        .map(|transform| transform.forward_vector())
        .unwrap_or_else(|| Vec3::new(0.0, 0.0, -1.0));

    let state = &mut sandbox_world.character;
    let mut velocity = state.velocity;
    let was_grounded = state.grounded;

    let hang_target = if state.hanging { 1.0 } else { 0.0 };
    state.hang_weight +=
        (hang_target - state.hang_weight) * (1.0 - (-HANG_POSE_SHARPNESS * delta_time).exp());
    if state.hanging {
        hang_update(state, world, body, position, &inputs, delta_time);
        return;
    }

    let crouch_target = if inputs.crouch_held && state.grounded {
        1.0
    } else {
        0.0
    };
    let crouch_spring = (crouch_target - state.crouch_amount) * CROUCH_SPRING_STIFFNESS
        - state.crouch_velocity * CROUCH_SPRING_DAMPING;
    state.crouch_velocity += crouch_spring * delta_time;
    state.crouch_amount =
        (state.crouch_amount + state.crouch_velocity * delta_time).clamp(-0.05, 1.2);

    let crouch_factor = state.crouch_amount.clamp(0.0, 1.0);
    let effective_max_speed = MAX_GROUND_SPEED * (1.0 - CROUCH_SPEED_PENALTY * crouch_factor);
    let target_velocity = inputs.direction * effective_max_speed;
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
    if state.wall_running {
        lean_target += -state.wall_normal * WALL_RUN_LEAN;
    }
    if lean_target.norm() > LEAN_MAX_RADIANS + WALL_RUN_LEAN {
        lean_target = lean_target.normalize() * (LEAN_MAX_RADIANS + WALL_RUN_LEAN);
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

    if state.wall_running {
        velocity.y += gravity_y * WALL_RUN_GRAVITY_SCALE * delta_time;
        velocity.y = velocity.y.max(WALL_RUN_FALL_RATE);
        velocity -= state.wall_normal * WALL_RUN_STICK * delta_time;
    } else {
        velocity.y += gravity_y * delta_time;
    }

    state.jump_suppress_timer = (state.jump_suppress_timer - delta_time).max(0.0);
    if inputs.jump_pressed && state.grounded {
        velocity.y = JUMP_IMPULSE;
        state.jump_suppress_timer = JUMP_SPRING_SUPPRESS_SECONDS;
        state.grounded = false;
    } else if inputs.jump_pressed && state.wall_running {
        velocity += state.wall_normal * WALL_JUMP_AWAY;
        velocity.y = WALL_JUMP_UP;
        state.wall_running = false;
        state.wall_run_timer = 0.0;
        state.wall_run_cooldown = WALL_RUN_COOLDOWN_SECONDS;
        state.jump_suppress_timer = JUMP_SPRING_SUPPRESS_SECONDS;
    }

    let impact_speed = (-velocity.y).max(0.0);
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
    if !was_grounded && state.grounded {
        state.crouch_velocity += impact_speed * LANDING_CROUCH_FACTOR;
        state.landing_impact_speed = impact_speed;
    } else {
        state.landing_impact_speed = 0.0;
    }

    let air_target = if state.grounded || state.wall_running {
        0.0
    } else {
        1.0
    };
    state.air_weight +=
        (air_target - state.air_weight) * (1.0 - (-AIR_POSE_SHARPNESS * delta_time).exp());

    let flip_active = state.flip_duration > 0.0 && state.flip_time < state.flip_duration;
    if inputs.flip_pressed && !flip_active {
        let horizontal = Vec3::new(velocity.x, 0.0, velocity.z);
        let flip_direction = if inputs.direction.norm() > 0.3 {
            inputs.direction.normalize()
        } else if horizontal.norm() > 0.5 {
            horizontal.normalize()
        } else {
            Vec3::new(state.facing_yaw.sin(), 0.0, state.facing_yaw.cos())
        };
        let start = if !state.grounded {
            Some((FLIP_DURATION, FLIP_PIVOT_HEIGHT))
        } else if horizontal.norm() > ROLL_MINIMUM_SPEED {
            Some((ROLL_DURATION, ROLL_PIVOT_HEIGHT))
        } else {
            None
        };
        if let Some((duration, pivot)) = start {
            let axis_world = nalgebra_glm::cross(&Vec3::y(), &flip_direction).normalize();
            let inverse_yaw = nalgebra_glm::quat_angle_axis(-state.facing_yaw, &Vec3::y());
            state.flip_axis = nalgebra_glm::quat_rotate_vec3(&inverse_yaw, &axis_world);
            let facing_right = Vec3::new(state.facing_yaw.cos(), 0.0, -state.facing_yaw.sin());
            state.flip_side = flip_direction.dot(&facing_right).abs();
            state.flip_time = 0.0;
            state.flip_duration = duration;
            state.flip_pivot = pivot;
        }
    }
    if state.flip_duration > 0.0 {
        state.flip_time += delta_time;
        if state.flip_time >= state.flip_duration {
            state.flip_duration = 0.0;
            state.tuck_weight = 0.0;
        } else {
            let progress = state.flip_time / state.flip_duration;
            state.tuck_weight = (std::f32::consts::PI * progress).sin();
        }
    }

    position += velocity * delta_time;

    let mut wall_contact_normal: Option<Vec3> = None;
    for _ in 0..BUMPER_RESOLVE_ITERATIONS {
        let contacts = physics_world_capsule_contacts(
            world.plugin_resource::<PhysicsWorld>(),
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
            if contact.normal.y.abs() < WALL_NORMAL_MAX_Y {
                wall_contact_normal = Some(contact.normal);
            }
        }
    }

    state.wall_run_cooldown = (state.wall_run_cooldown - delta_time).max(0.0);
    let wall_speed = Vec3::new(velocity.x, 0.0, velocity.z).norm();
    let can_wall_run = !state.grounded
        && wall_speed > WALL_RUN_MINIMUM_SPEED
        && state.wall_run_cooldown <= 0.0
        && state.wall_run_timer < WALL_RUN_MAX_SECONDS;
    if let Some(normal) = wall_contact_normal
        && can_wall_run
    {
        state.wall_running = true;
        state.wall_normal = Vec3::new(normal.x, 0.0, normal.z).normalize();
        state.wall_run_timer += delta_time;
        if state.wall_run_timer >= WALL_RUN_MAX_SECONDS {
            state.wall_running = false;
            state.wall_run_cooldown = WALL_RUN_COOLDOWN_SECONDS;
        }
    } else if state.wall_running {
        state.wall_running = false;
        state.wall_run_timer = 0.0;
        state.wall_run_cooldown = WALL_RUN_COOLDOWN_SECONDS;
    } else if state.grounded {
        state.wall_run_timer = 0.0;
    }

    if !state.grounded
        && !state.wall_running
        && velocity.y < HANG_MAX_FALL_SPEED
        && let Some(normal) = wall_contact_normal
    {
        let wall_normal = Vec3::new(normal.x, 0.0, normal.z);
        if wall_normal.norm() > 0.5 {
            let wall_normal = wall_normal.normalize();
            let probe_origin = position
                + Vec3::new(0.0, HANG_PROBE_RISE, 0.0)
                + (-wall_normal) * HANG_PROBE_FORWARD;
            if let Some(hit) = physics_world_cast_ray(
                world.plugin_resource::<PhysicsWorld>(),
                probe_origin,
                Vec3::new(0.0, -1.0, 0.0),
                HANG_PROBE_DOWN,
                None,
            ) {
                let ledge_height = hit.point.y - position.y;
                if (HANG_REACH_MINIMUM..=HANG_REACH_MAXIMUM).contains(&ledge_height) {
                    state.hanging = true;
                    state.hang_normal = wall_normal;
                    state.hang_tangent = nalgebra_glm::cross(&Vec3::y(), &wall_normal).normalize();
                    state.hang_edge_y = hit.point.y;
                    position.y = hit.point.y - HANG_DEPTH;
                    velocity = Vec3::zeros();
                    state.facing_yaw = (-wall_normal.x).atan2(-wall_normal.z);
                }
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
    let horizontal_speed = horizontal_velocity.norm();
    let run_factor_target =
        ((horizontal_speed - WALK_FULL_SPEED) / (RUN_FULL_SPEED - WALK_FULL_SPEED)).clamp(0.0, 1.0);
    state.gait_run_factor += (run_factor_target - state.gait_run_factor)
        * (1.0 - (-GAIT_BLEND_SHARPNESS * delta_time).exp());
    let upright_radius =
        WALK_WHEEL_RADIUS + (RUN_WHEEL_RADIUS - WALK_WHEEL_RADIUS) * state.gait_run_factor;
    state.stride_radius = upright_radius
        + (CROUCH_WHEEL_RADIUS - upright_radius) * state.crouch_amount.clamp(0.0, 1.0);
    if state.grounded || state.wall_running {
        state.stride_phase = (state.stride_phase
            + horizontal_speed * delta_time / state.stride_radius)
            .rem_euclid(std::f32::consts::TAU);
    }
    let gait_target = if state.grounded || state.wall_running {
        ((horizontal_speed - GAIT_FADE_START_SPEED) / (GAIT_FULL_SPEED - GAIT_FADE_START_SPEED))
            .clamp(0.0, 1.0)
    } else {
        0.0
    };
    state.gait_weight +=
        (gait_target - state.gait_weight) * (1.0 - (-GAIT_WEIGHT_SHARPNESS * delta_time).exp());
    let inverse_facing = nalgebra_glm::quat_angle_axis(-state.facing_yaw, &Vec3::y());
    let aim_in_body = nalgebra_glm::quat_rotate_vec3(&inverse_facing, &camera_aim);
    let forwardness = (aim_in_body.z / LOOK_FADE_FORWARDNESS).clamp(0.0, 1.0);
    let look_yaw_target = aim_in_body
        .x
        .atan2(aim_in_body.z)
        .clamp(-LOOK_YAW_LIMIT, LOOK_YAW_LIMIT)
        * forwardness;
    let look_pitch_target = (-aim_in_body.y.clamp(-1.0, 1.0).asin())
        .clamp(-LOOK_PITCH_LIMIT, LOOK_PITCH_LIMIT)
        * forwardness;
    let look_blend = 1.0 - (-LOOK_SHARPNESS * delta_time).exp();
    state.look_yaw += (look_yaw_target - state.look_yaw) * look_blend;
    state.look_pitch += (look_pitch_target - state.look_pitch) * look_blend;

    let facing_yaw = state.facing_yaw;
    let lean = state.lean;

    state.velocity = velocity;
    if let Some(transform) =
        world.get_mut::<nightshade::ecs::transform::components::LocalTransform>(body)
    {
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
}

/// Ledge hang: gravity is suspended while the hands own the edge.
/// Lateral input shimmies along the edge as long as the ledge
/// continues, jump climbs up and over, and crouch lets go.
fn hang_update(
    state: &mut CharacterState,
    world: &mut World,
    body: Entity,
    mut position: Vec3,
    inputs: &CharacterInputs,
    delta_time: f32,
) {
    state.velocity = Vec3::zeros();
    state.grounded = false;
    let decay = 1.0 - (-8.0 * delta_time).exp();
    state.gait_weight -= state.gait_weight * decay;
    state.air_weight -= state.air_weight * decay;
    state.tuck_weight = 0.0;
    state.flip_duration = 0.0;

    if inputs.jump_pressed {
        state.hanging = false;
        state.velocity =
            -state.hang_normal * CLIMB_FORWARD_SPEED + Vec3::new(0.0, CLIMB_UP_SPEED, 0.0);
        state.jump_suppress_timer = JUMP_SPRING_SUPPRESS_SECONDS;
        return;
    }
    if inputs.crouch_held {
        state.hanging = false;
        return;
    }

    let shimmy = inputs.direction.dot(&state.hang_tangent);
    if shimmy.abs() > 0.2 {
        let movement = state.hang_tangent * shimmy.signum() * SHIMMY_SPEED * delta_time;
        let candidate = position + movement;
        let probe_origin = candidate
            + Vec3::new(0.0, HANG_PROBE_RISE, 0.0)
            + (-state.hang_normal) * HANG_PROBE_FORWARD;
        if let Some(hit) = physics_world_cast_ray(
            world.plugin_resource::<PhysicsWorld>(),
            probe_origin,
            Vec3::new(0.0, -1.0, 0.0),
            HANG_PROBE_DOWN,
            None,
        ) {
            let ledge_height = hit.point.y - candidate.y;
            if (HANG_REACH_MINIMUM..=HANG_REACH_MAXIMUM).contains(&ledge_height) {
                position = candidate;
                state.hang_edge_y = hit.point.y;
                position.y = hit.point.y - HANG_DEPTH;
            }
        }
    }

    if let Some(transform) =
        world.get_mut::<nightshade::ecs::transform::components::LocalTransform>(body)
    {
        transform.translation = position;
        transform.rotation = nalgebra_glm::quat_angle_axis(state.facing_yaw, &Vec3::y());
    }
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
            world.plugin_resource::<PhysicsWorld>(),
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

fn sync_first_person_player(sandbox_world: &SandboxResources, world: &mut World, body: Entity) {
    let Some(player_entity) = sandbox_world.player.player_entity else {
        return;
    };
    let Some(feet) = world
        .get::<nightshade::ecs::transform::components::LocalTransform>(body)
        .map(|transform| transform.translation)
    else {
        return;
    };
    let capsule_center = feet + Vec3::new(0.0, FIRST_PERSON_CAPSULE_HALF_HEIGHT, 0.0);
    if let Some(transform) =
        world.get_mut::<nightshade::ecs::transform::components::LocalTransform>(player_entity)
    {
        transform.translation = capsule_center;
    }
    if let Some(interpolation) = world
        .get_mut::<nightshade::plugins::physics::components::PhysicsInterpolation>(player_entity)
    {
        interpolation.previous_translation = capsule_center;
        interpolation.current_translation = capsule_center;
    }
}

fn draw_spheres(sandbox_world: &SandboxResources, world: &mut World, body: Entity) {
    let Some(spheres) = sandbox_world.character.spheres else {
        return;
    };
    let Some(feet) = world
        .get::<nightshade::ecs::transform::components::LocalTransform>(body)
        .map(|transform| transform.translation)
    else {
        return;
    };
    let Some(lines) = world.get_mut::<nightshade::ecs::lines::components::Lines>(spheres) else {
        return;
    };
    lines.lines.clear();
    let state = &sandbox_world.character;
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
            state.stride_phase,
            state.stride_radius,
        );
    }
    lines.mark_dirty();
}

fn push_stride_wheel(lines: &mut Vec<Line>, feet: Vec3, facing_yaw: f32, phase: f32, radius: f32) {
    let forward = Vec3::new(facing_yaw.sin(), 0.0, facing_yaw.cos());
    let center = feet + Vec3::new(0.0, radius, 0.0);
    push_circle(
        lines,
        center,
        radius,
        forward,
        Vec3::y(),
        STRIDE_WHEEL_COLOR,
    );
    for spoke_index in 0..STRIDE_WHEEL_SPOKES {
        let angle =
            (spoke_index as f32 / STRIDE_WHEEL_SPOKES as f32) * std::f32::consts::TAU - phase;
        let rim =
            center + forward * angle.cos() * radius + Vec3::new(0.0, angle.sin() * radius, 0.0);
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
