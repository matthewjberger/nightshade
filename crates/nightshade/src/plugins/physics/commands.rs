use super::*;
use crate::ecs::material::resources::material_registry_find_or_insert;
use crate::ecs::world::{Entity, Vec3, World};

use crate::prelude::*;
pub fn create_textured_material(
    base_color: Vec3,
    roughness: f32,
    metallic: f32,
) -> crate::render::material::Material {
    crate::render::material::Material {
        base_color: [base_color.x, base_color.y, base_color.z, 1.0],
        emissive_factor: [0.0, 0.0, 0.0],
        roughness,
        metallic,
        ..Default::default()
    }
}

pub fn create_emissive_material(
    base_color: Vec3,
    emissive_strength: f32,
) -> crate::render::material::Material {
    crate::render::material::Material {
        base_color: [base_color.x, base_color.y, base_color.z, 1.0],
        emissive_factor: [
            base_color.x * emissive_strength,
            base_color.y * emissive_strength,
            base_color.z * emissive_strength,
        ],
        ..Default::default()
    }
}

pub fn spawn_static_physics_cube_with_material(
    world: &mut World,
    position: Vec3,
    size: Vec3,
    material: crate::render::material::Material,
) -> Entity {
    let entity = spawn_entities(
        world,
        crate::ecs::world::NAME
            | crate::ecs::world::LOCAL_TRANSFORM
            | crate::ecs::world::GLOBAL_TRANSFORM
            | crate::ecs::world::RENDER_MESH
            | crate::ecs::world::MATERIAL_REF
            | crate::ecs::world::BOUNDING_VOLUME
            | crate::ecs::world::CASTS_SHADOW
            | crate::ecs::world::RIGID_BODY
            | crate::ecs::world::COLLIDER
            | crate::ecs::world::VISIBILITY,
        1,
    )[0];

    if let Some(name) = world.get_mut::<crate::ecs::primitives::Name>(entity) {
        name.0 = "Physics Cube".to_string();
    }

    if let Some(transform) =
        world.get_mut::<crate::ecs::transform::components::LocalTransform>(entity)
    {
        transform.translation = position;
        transform.scale = size;
    }

    if let Some(mesh) = world.get_mut::<crate::ecs::mesh::components::RenderMesh>(entity) {
        mesh.name = "Cube".to_string();
    }

    let fallback_name = format!("StaticPhysicsCube_{}", entity.id);
    let material_name = material_registry_find_or_insert(
        &mut world
            .expect_resource_mut::<crate::ecs::asset_state::AssetState>()
            .material_registry,
        fallback_name,
        material,
    );
    if let Some(&index) = world
        .expect_resource_mut::<crate::ecs::asset_state::AssetState>()
        .material_registry
        .registry
        .name_to_index
        .get(&material_name)
    {
        registry_add_reference(
            &mut world
                .expect_resource_mut::<crate::ecs::asset_state::AssetState>()
                .material_registry
                .registry,
            index,
        );
    }
    world.set(
        entity,
        crate::ecs::material::components::MaterialRef::new(material_name),
    );

    if let Some(bv) = world.get_mut::<crate::render::bounding_volume::BoundingVolume>(entity) {
        *bv = crate::ecs::world::components::BoundingVolume::from_mesh_type("Cube");
    }

    if let Some(rigid_body) =
        world.get_mut::<crate::plugins::physics::components::RigidBodyComponent>(entity)
    {
        *rigid_body =
            RigidBodyComponent::new_static().with_translation(position.x, position.y, position.z);
    }

    if let Some(collider) =
        world.get_mut::<crate::plugins::physics::components::ColliderComponent>(entity)
    {
        *collider = ColliderComponent::new_cuboid(size.x / 2.0, size.y / 2.0, size.z / 2.0)
            .with_friction(0.8)
            .with_restitution(0.1);
    }

    world
        .expect_resource_mut::<crate::render::mesh_state::MeshRenderState>()
        .mark_entity_added(render_entity(entity));

    entity
}

pub fn spawn_dynamic_physics_sphere_with_material(
    world: &mut World,
    position: Vec3,
    radius: f32,
    mass: f32,
    material: crate::render::material::Material,
) -> Entity {
    let scale_factor = radius;
    let entity = spawn_entities(
        world,
        crate::ecs::world::NAME
            | crate::ecs::world::LOCAL_TRANSFORM
            | crate::ecs::world::GLOBAL_TRANSFORM
            | crate::ecs::world::RENDER_MESH
            | crate::ecs::world::MATERIAL_REF
            | crate::ecs::world::BOUNDING_VOLUME
            | crate::ecs::world::CASTS_SHADOW
            | crate::ecs::world::RIGID_BODY
            | crate::ecs::world::COLLIDER
            | crate::ecs::world::COLLISION_LISTENER
            | crate::ecs::world::PHYSICS_INTERPOLATION
            | crate::ecs::world::VISIBILITY,
        1,
    )[0];

    if let Some(name) = world.get_mut::<crate::ecs::primitives::Name>(entity) {
        name.0 = "Physics Sphere".to_string();
    }

    if let Some(transform) =
        world.get_mut::<crate::ecs::transform::components::LocalTransform>(entity)
    {
        transform.translation = position;
        transform.scale = nalgebra_glm::vec3(scale_factor, scale_factor, scale_factor);
    }

    if let Some(mesh) = world.get_mut::<crate::ecs::mesh::components::RenderMesh>(entity) {
        mesh.name = "Sphere".to_string();
    }

    let fallback_name = format!("DynamicPhysicsSphere_{}", entity.id);
    let material_name = material_registry_find_or_insert(
        &mut world
            .expect_resource_mut::<crate::ecs::asset_state::AssetState>()
            .material_registry,
        fallback_name,
        material,
    );
    if let Some(&index) = world
        .expect_resource_mut::<crate::ecs::asset_state::AssetState>()
        .material_registry
        .registry
        .name_to_index
        .get(&material_name)
    {
        registry_add_reference(
            &mut world
                .expect_resource_mut::<crate::ecs::asset_state::AssetState>()
                .material_registry
                .registry,
            index,
        );
    }
    world.set(
        entity,
        crate::ecs::material::components::MaterialRef::new(material_name),
    );

    if let Some(bv) = world.get_mut::<crate::render::bounding_volume::BoundingVolume>(entity) {
        *bv = crate::ecs::world::components::BoundingVolume::from_mesh_type("Sphere");
    }

    if let Some(rigid_body) =
        world.get_mut::<crate::plugins::physics::components::RigidBodyComponent>(entity)
    {
        *rigid_body = RigidBodyComponent::new_dynamic()
            .with_translation(position.x, position.y, position.z)
            .with_mass(mass);
    }

    if let Some(collider) =
        world.get_mut::<crate::plugins::physics::components::ColliderComponent>(entity)
    {
        *collider = ColliderComponent::new_ball(radius)
            .with_friction(0.7)
            .with_restitution(0.3);
    }

    world
        .expect_resource_mut::<crate::render::mesh_state::MeshRenderState>()
        .mark_entity_added(render_entity(entity));

    entity
}

pub fn spawn_dynamic_physics_cube_with_material(
    world: &mut World,
    position: Vec3,
    size: Vec3,
    mass: f32,
    material: crate::render::material::Material,
) -> Entity {
    let entity = spawn_entities(
        world,
        crate::ecs::world::NAME
            | crate::ecs::world::LOCAL_TRANSFORM
            | crate::ecs::world::GLOBAL_TRANSFORM
            | crate::ecs::world::RENDER_MESH
            | crate::ecs::world::MATERIAL_REF
            | crate::ecs::world::BOUNDING_VOLUME
            | crate::ecs::world::CASTS_SHADOW
            | crate::ecs::world::RIGID_BODY
            | crate::ecs::world::COLLIDER
            | crate::ecs::world::COLLISION_LISTENER
            | crate::ecs::world::PHYSICS_INTERPOLATION
            | crate::ecs::world::VISIBILITY,
        1,
    )[0];

    if let Some(name) = world.get_mut::<crate::ecs::primitives::Name>(entity) {
        name.0 = "Physics Cube".to_string();
    }

    if let Some(transform) =
        world.get_mut::<crate::ecs::transform::components::LocalTransform>(entity)
    {
        transform.translation = position;
        transform.scale = size;
    }

    if let Some(mesh) = world.get_mut::<crate::ecs::mesh::components::RenderMesh>(entity) {
        mesh.name = "Cube".to_string();
    }

    let fallback_name = format!("DynamicPhysicsCube_{}", entity.id);
    let material_name = material_registry_find_or_insert(
        &mut world
            .expect_resource_mut::<crate::ecs::asset_state::AssetState>()
            .material_registry,
        fallback_name,
        material,
    );
    if let Some(&index) = world
        .expect_resource_mut::<crate::ecs::asset_state::AssetState>()
        .material_registry
        .registry
        .name_to_index
        .get(&material_name)
    {
        registry_add_reference(
            &mut world
                .expect_resource_mut::<crate::ecs::asset_state::AssetState>()
                .material_registry
                .registry,
            index,
        );
    }
    world.set(
        entity,
        crate::ecs::material::components::MaterialRef::new(material_name),
    );

    if let Some(bv) = world.get_mut::<crate::render::bounding_volume::BoundingVolume>(entity) {
        *bv = crate::ecs::world::components::BoundingVolume::from_mesh_type("Cube");
    }

    if let Some(rigid_body) =
        world.get_mut::<crate::plugins::physics::components::RigidBodyComponent>(entity)
    {
        *rigid_body = RigidBodyComponent::new_dynamic()
            .with_translation(position.x, position.y, position.z)
            .with_mass(mass);
    }

    if let Some(collider) =
        world.get_mut::<crate::plugins::physics::components::ColliderComponent>(entity)
    {
        *collider = ColliderComponent::new_cuboid(size.x / 2.0, size.y / 2.0, size.z / 2.0)
            .with_friction(0.7)
            .with_restitution(0.2);
    }

    world
        .expect_resource_mut::<crate::render::mesh_state::MeshRenderState>()
        .mark_entity_added(render_entity(entity));

    entity
}

pub fn spawn_dynamic_physics_cylinder_with_material(
    world: &mut World,
    position: Vec3,
    half_height: f32,
    radius: f32,
    mass: f32,
    material: crate::render::material::Material,
) -> Entity {
    let entity = spawn_entities(
        world,
        crate::ecs::world::NAME
            | crate::ecs::world::LOCAL_TRANSFORM
            | crate::ecs::world::GLOBAL_TRANSFORM
            | crate::ecs::world::RENDER_MESH
            | crate::ecs::world::MATERIAL_REF
            | crate::ecs::world::BOUNDING_VOLUME
            | crate::ecs::world::CASTS_SHADOW
            | crate::ecs::world::RIGID_BODY
            | crate::ecs::world::COLLIDER
            | crate::ecs::world::COLLISION_LISTENER
            | crate::ecs::world::PHYSICS_INTERPOLATION
            | crate::ecs::world::VISIBILITY,
        1,
    )[0];

    if let Some(name) = world.get_mut::<crate::ecs::primitives::Name>(entity) {
        name.0 = "Physics Cylinder".to_string();
    }

    if let Some(transform) =
        world.get_mut::<crate::ecs::transform::components::LocalTransform>(entity)
    {
        transform.translation = position;
        transform.scale = nalgebra_glm::vec3(radius * 2.0, half_height * 2.0, radius * 2.0);
    }

    if let Some(mesh) = world.get_mut::<crate::ecs::mesh::components::RenderMesh>(entity) {
        mesh.name = "Cylinder".to_string();
    }

    let fallback_name = format!("DynamicPhysicsCylinder_{}", entity.id);
    let material_name = material_registry_find_or_insert(
        &mut world
            .expect_resource_mut::<crate::ecs::asset_state::AssetState>()
            .material_registry,
        fallback_name,
        material,
    );
    if let Some(&index) = world
        .expect_resource_mut::<crate::ecs::asset_state::AssetState>()
        .material_registry
        .registry
        .name_to_index
        .get(&material_name)
    {
        registry_add_reference(
            &mut world
                .expect_resource_mut::<crate::ecs::asset_state::AssetState>()
                .material_registry
                .registry,
            index,
        );
    }
    world.set(
        entity,
        crate::ecs::material::components::MaterialRef::new(material_name),
    );

    if let Some(bv) = world.get_mut::<crate::render::bounding_volume::BoundingVolume>(entity) {
        *bv = crate::ecs::world::components::BoundingVolume::from_mesh_type("Cylinder");
    }

    if let Some(rigid_body) =
        world.get_mut::<crate::plugins::physics::components::RigidBodyComponent>(entity)
    {
        *rigid_body = RigidBodyComponent::new_dynamic()
            .with_translation(position.x, position.y, position.z)
            .with_mass(mass);
    }

    if let Some(collider) =
        world.get_mut::<crate::plugins::physics::components::ColliderComponent>(entity)
    {
        *collider = ColliderComponent::new_cylinder(half_height, radius)
            .with_friction(0.5)
            .with_restitution(0.3);
    }

    world
        .expect_resource_mut::<crate::render::mesh_state::MeshRenderState>()
        .mark_entity_added(render_entity(entity));

    entity
}

pub fn spawn_first_person_player(world: &mut World, position: Vec3) -> (Entity, Entity) {
    let player_entity = spawn_entities(
        world,
        crate::ecs::world::NAME
            | crate::ecs::world::LOCAL_TRANSFORM
            | crate::ecs::world::GLOBAL_TRANSFORM
            | crate::ecs::world::CHARACTER_CONTROLLER
            | crate::ecs::world::PHYSICS_INTERPOLATION,
        1,
    )[0];

    if let Some(name) = world.get_mut::<crate::ecs::primitives::Name>(player_entity) {
        name.0 = "Player".to_string();
    }

    if let Some(transform) =
        world.get_mut::<crate::ecs::transform::components::LocalTransform>(player_entity)
    {
        transform.translation = position;
    }

    if let Some(interpolation) =
        world.get_mut::<crate::plugins::physics::components::PhysicsInterpolation>(player_entity)
    {
        interpolation.previous_translation = position;
        interpolation.previous_rotation = nalgebra_glm::quat_identity();
        interpolation.current_translation = position;
        interpolation.current_rotation = nalgebra_glm::quat_identity();
        interpolation.enabled = true;
    }

    if let Some(controller) = world
        .get_mut::<crate::plugins::physics::components::CharacterControllerComponent>(player_entity)
    {
        *controller = CharacterControllerComponent::new_capsule(0.7, 0.3);
        controller.max_speed = 5.0;
        controller.acceleration = 50.0;
        controller.jump_impulse = 5.0;
    }

    let camera_entity = spawn_entities(
        world,
        crate::ecs::world::NAME
            | crate::ecs::world::CAMERA
            | crate::ecs::world::LOCAL_TRANSFORM
            | crate::ecs::world::GLOBAL_TRANSFORM
            | crate::ecs::world::PARENT,
        1,
    )[0];

    if let Some(name) = world.get_mut::<crate::ecs::primitives::Name>(camera_entity) {
        name.0 = "Player Camera".to_string();
    }

    if let Some(transform) =
        world.get_mut::<crate::ecs::transform::components::LocalTransform>(camera_entity)
    {
        transform.translation = nalgebra_glm::vec3(0.0, 1.3, 0.0);
    }

    if let Some(parent) = world.get_mut::<freecs::dynamic::ChildOf>(camera_entity) {
        *parent = crate::ecs::transform::components::ChildOf(player_entity);
    }

    if let Some(camera) = world.get_mut::<crate::ecs::camera::components::Camera>(camera_entity) {
        camera.projection = crate::ecs::camera::components::Projection::Perspective(
            crate::ecs::camera::components::PerspectiveCamera {
                y_fov_rad: 85.0_f32.to_radians(),
                z_near: 0.01,
                z_far: None,
                aspect_ratio: None,
            },
        );
        camera.smoothing = Some(crate::ecs::camera::components::Smoothing {
            mouse_sensitivity: 1.5,
            ..Default::default()
        });
    }

    world
        .expect_resource_mut::<crate::ecs::camera::resources::ActiveCamera>()
        .0 = Some(camera_entity);

    (player_entity, camera_entity)
}

pub fn look_camera_system(world: &mut World) {
    let Some(camera_entity) = world
        .expect_resource_mut::<crate::ecs::camera::resources::ActiveCamera>()
        .0
    else {
        return;
    };

    let delta_time = world
        .expect_resource_mut::<crate::ecs::time::Time>()
        .delta_time;
    let right_clicked = world
        .expect_resource_mut::<crate::ecs::input::resources::Input>()
        .mouse
        .state
        .contains(crate::ecs::input::resources::MouseState::RIGHT_CLICKED);
    let raw_mouse_delta = world
        .expect_resource_mut::<crate::ecs::input::resources::Input>()
        .mouse
        .raw_mouse_delta;

    #[cfg(feature = "gamepad")]
    let gamepad_input = crate::plugins::gamepad::with_active_gamepad(world, |gamepad| {
        let right_stick_x = gamepad.value(gilrs::Axis::RightStickX);
        let right_stick_y = gamepad.value(gilrs::Axis::RightStickY);
        (right_stick_x, right_stick_y)
    });
    #[cfg(not(feature = "gamepad"))]
    let gamepad_input: Option<(f32, f32)> = None;

    let Some(camera) = world.get_mut::<crate::ecs::camera::components::Camera>(camera_entity)
    else {
        return;
    };

    let Some(smoothing) = camera.smoothing.as_mut() else {
        return;
    };

    let mouse_delta = if right_clicked {
        let smoothing_factor = if smoothing.mouse_smoothness > 0.0 {
            1.0 - smoothing.mouse_smoothness.powi(7).powf(delta_time)
        } else {
            1.0
        };
        smoothing.smoothed_mouse_delta = smoothing.smoothed_mouse_delta * (1.0 - smoothing_factor)
            + raw_mouse_delta * smoothing_factor;

        let pixels_to_radians = (std::f32::consts::PI / 1000.0) * smoothing.mouse_dpi_scale;
        let mut delta =
            smoothing.smoothed_mouse_delta * smoothing.mouse_sensitivity * pixels_to_radians;
        delta.x *= -1.0;
        delta.y *= -1.0;
        Some(delta)
    } else {
        let decay_smoothness = (smoothing.mouse_smoothness * 0.5).max(0.01);
        let smoothing_factor = 1.0 - decay_smoothness.powi(7).powf(delta_time);
        smoothing.smoothed_mouse_delta *= 1.0 - smoothing_factor;
        None
    };

    let gamepad_delta = if let Some((right_stick_x, right_stick_y)) = gamepad_input {
        let deadzone = smoothing.gamepad_deadzone;
        let right_stick_magnitude =
            (right_stick_x * right_stick_x + right_stick_y * right_stick_y).sqrt();

        if right_stick_magnitude > deadzone {
            let target_input = nalgebra_glm::vec2(
                apply_deadzone_physics(right_stick_x, deadzone),
                apply_deadzone_physics(right_stick_y, deadzone),
            );

            let smoothing_factor = if smoothing.gamepad_smoothness > 0.0 {
                1.0 - smoothing.gamepad_smoothness.powi(7).powf(delta_time)
            } else {
                1.0
            };
            smoothing.smoothed_gamepad_input = smoothing.smoothed_gamepad_input
                * (1.0 - smoothing_factor)
                + target_input * smoothing_factor;

            let delta = nalgebra_glm::vec2(
                -smoothing.smoothed_gamepad_input.x,
                smoothing.smoothed_gamepad_input.y,
            ) * smoothing.gamepad_sensitivity
                * delta_time;
            Some(delta)
        } else {
            let smoothing_factor = if smoothing.gamepad_smoothness > 0.0 {
                1.0 - smoothing.gamepad_smoothness.powi(7).powf(delta_time)
            } else {
                1.0
            };
            smoothing.smoothed_gamepad_input *= 1.0 - smoothing_factor;
            None
        }
    } else {
        None
    };

    let final_delta = match (mouse_delta, gamepad_delta) {
        (Some(mouse), Some(gamepad)) => mouse + gamepad,
        (Some(mouse), None) => mouse,
        (None, Some(gamepad)) => gamepad,
        (None, None) => return,
    };

    let Some(local_transform) =
        world.get_mut::<crate::ecs::transform::components::LocalTransform>(camera_entity)
    else {
        return;
    };

    let yaw = nalgebra_glm::quat_angle_axis(final_delta.x, &nalgebra_glm::vec3(0.0, 1.0, 0.0));
    local_transform.rotation = yaw * local_transform.rotation;

    let forward_vector = local_transform.forward_vector();
    let current_pitch = forward_vector.y.asin();

    let new_pitch = current_pitch + final_delta.y;
    if new_pitch.abs() <= 89_f32.to_radians() {
        let pitch =
            nalgebra_glm::quat_angle_axis(final_delta.y, &nalgebra_glm::vec3(1.0, 0.0, 0.0));
        local_transform.rotation *= pitch;
    }
}

fn apply_deadzone_physics(value: f32, deadzone: f32) -> f32 {
    if value.abs() > deadzone {
        let sign = value.signum();
        let compensated = (value.abs() - deadzone) / (1.0 - deadzone);
        sign * compensated
    } else {
        0.0
    }
}

pub fn reset_physics_interpolation(world: &mut World, entity: Entity) {
    let Some(transform) = world
        .get::<crate::ecs::transform::components::LocalTransform>(entity)
        .copied()
    else {
        return;
    };
    if let Some(interpolation) =
        world.get_mut::<crate::plugins::physics::components::PhysicsInterpolation>(entity)
    {
        interpolation.previous_translation = transform.translation;
        interpolation.previous_rotation = transform.rotation;
        interpolation.current_translation = transform.translation;
        interpolation.current_rotation = transform.rotation;
    }
}
