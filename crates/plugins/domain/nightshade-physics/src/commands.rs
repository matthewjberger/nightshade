use nightshade_render::commands::mesh_state::mesh_state_mark_entity_added;
pub mod joints;
pub(crate) mod physics_world;
pub use physics_world::*;
mod picking;
pub use picking::*;

use nalgebra_glm::Vec3;
use nightshade::ecs::Entity;
use nightshade::ecs::system_param::Commands;
use nightshade_render::commands::material_registry::material_registry_find_or_insert;
use nightshade_render::data::generational_registry::registry_add_reference;

use crate::prelude::*;

type PhysicsMeshSet = (
    nightshade_scene::contract::primitives::Name,
    nightshade_scene::contract::LocalTransform,
    nightshade_scene::contract::GlobalTransform,
    nightshade_render::components::mesh::RenderMesh,
    nightshade_render::components::material::MaterialRef,
    nightshade_render::data::bounding_volume::BoundingVolume,
    nightshade_render::components::culling::CastsShadow,
    nightshade_scene::contract::primitives::Visibility,
);

pub fn create_textured_material(
    base_color: Vec3,
    roughness: f32,
    metallic: f32,
) -> nightshade_render::data::material::Material {
    nightshade_render::data::material::Material {
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
) -> nightshade_render::data::material::Material {
    nightshade_render::data::material::Material {
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
    commands: &mut Commands<'_>,
    position: Vec3,
    size: Vec3,
    material: nightshade_render::data::material::Material,
) -> Entity {
    let entity = commands.reserve_entity();
    commands.spawn_reserved(entity, ());
    commands.queue(move |storage, resources| {
        storage.add_components::<PhysicsMeshSet>(entity);

        if let Some(name) = storage.get_mut::<nightshade_scene::contract::primitives::Name>(entity)
        {
            name.0 = "Physics Cube".to_string();
        }

        if let Some(transform) =
            storage.get_mut::<nightshade_scene::contract::LocalTransform>(entity)
        {
            transform.translation = position;
            transform.scale = size;
        }

        if let Some(mesh) =
            storage.get_mut::<nightshade_render::components::mesh::RenderMesh>(entity)
        {
            mesh.name = "Cube".to_string();
            nightshade_render::commands::primitives::register_primitive_mesh(resources, "Cube");
        }

        let fallback_name = format!("StaticPhysicsCube_{}", entity.id);
        let material_name = material_registry_find_or_insert(
            resources
                .res_mut::<nightshade_render::resources::material_registry::MaterialRegistry>(),
            fallback_name,
            material,
        );
        registry_add_reference(
            &mut resources
                .res_mut::<nightshade_render::resources::material_registry::MaterialRegistry>()
                .registry,
            material_name.index,
        );
        storage.set(
            entity,
            nightshade_render::components::material::MaterialRef::new(material_name),
        );

        if let Some(bv) =
            storage.get_mut::<nightshade_render::data::bounding_volume::BoundingVolume>(entity)
        {
            *bv = nightshade_render::data::bounding_volume::BoundingVolume::from_mesh_type("Cube");
        }

        storage.set(
            entity,
            RigidBodyComponent::new_static().with_translation(position.x, position.y, position.z),
        );

        storage.set(
            entity,
            ColliderComponent::new_cuboid(size.x / 2.0, size.y / 2.0, size.z / 2.0)
                .with_friction(0.8)
                .with_restitution(0.1),
        );
        mesh_state_mark_entity_added(
            resources.res_mut::<nightshade_render::resources::mesh_state::MeshRenderState>(),
            entity,
        );
    });
    entity
}

pub fn spawn_dynamic_physics_sphere_with_material(
    commands: &mut Commands<'_>,
    position: Vec3,
    radius: f32,
    mass: f32,
    material: nightshade_render::data::material::Material,
) -> Entity {
    let entity = commands.reserve_entity();
    commands.spawn_reserved(entity, ());
    commands.queue(move |storage, resources| {
        storage.add_components::<PhysicsMeshSet>(entity);

        let scale_factor = radius;

        if let Some(name) = storage.get_mut::<nightshade_scene::contract::primitives::Name>(entity)
        {
            name.0 = "Physics Sphere".to_string();
        }

        if let Some(transform) =
            storage.get_mut::<nightshade_scene::contract::LocalTransform>(entity)
        {
            transform.translation = position;
            transform.scale = nalgebra_glm::vec3(scale_factor, scale_factor, scale_factor);
        }

        if let Some(mesh) =
            storage.get_mut::<nightshade_render::components::mesh::RenderMesh>(entity)
        {
            mesh.name = "Sphere".to_string();
            nightshade_render::commands::primitives::register_primitive_mesh(resources, "Sphere");
        }

        let fallback_name = format!("DynamicPhysicsSphere_{}", entity.id);
        let material_name = material_registry_find_or_insert(
            resources
                .res_mut::<nightshade_render::resources::material_registry::MaterialRegistry>(),
            fallback_name,
            material,
        );
        registry_add_reference(
            &mut resources
                .res_mut::<nightshade_render::resources::material_registry::MaterialRegistry>()
                .registry,
            material_name.index,
        );
        storage.set(
            entity,
            nightshade_render::components::material::MaterialRef::new(material_name),
        );

        if let Some(bv) =
            storage.get_mut::<nightshade_render::data::bounding_volume::BoundingVolume>(entity)
        {
            *bv =
                nightshade_render::data::bounding_volume::BoundingVolume::from_mesh_type("Sphere");
        }

        storage.set(
            entity,
            RigidBodyComponent::new_dynamic()
                .with_translation(position.x, position.y, position.z)
                .with_mass(mass),
        );

        storage.set(
            entity,
            ColliderComponent::new_ball(radius)
                .with_friction(0.7)
                .with_restitution(0.3),
        );
        mesh_state_mark_entity_added(
            resources.res_mut::<nightshade_render::resources::mesh_state::MeshRenderState>(),
            entity,
        );
    });
    entity
}

pub fn spawn_dynamic_physics_cube_with_material(
    commands: &mut Commands<'_>,
    position: Vec3,
    size: Vec3,
    mass: f32,
    material: nightshade_render::data::material::Material,
) -> Entity {
    let entity = commands.reserve_entity();
    commands.spawn_reserved(entity, ());
    commands.queue(move |storage, resources| {
        storage.add_components::<PhysicsMeshSet>(entity);

        if let Some(name) = storage.get_mut::<nightshade_scene::contract::primitives::Name>(entity)
        {
            name.0 = "Physics Cube".to_string();
        }

        if let Some(transform) =
            storage.get_mut::<nightshade_scene::contract::LocalTransform>(entity)
        {
            transform.translation = position;
            transform.scale = size;
        }

        if let Some(mesh) =
            storage.get_mut::<nightshade_render::components::mesh::RenderMesh>(entity)
        {
            mesh.name = "Cube".to_string();
            nightshade_render::commands::primitives::register_primitive_mesh(resources, "Cube");
        }

        let fallback_name = format!("DynamicPhysicsCube_{}", entity.id);
        let material_name = material_registry_find_or_insert(
            resources
                .res_mut::<nightshade_render::resources::material_registry::MaterialRegistry>(),
            fallback_name,
            material,
        );
        registry_add_reference(
            &mut resources
                .res_mut::<nightshade_render::resources::material_registry::MaterialRegistry>()
                .registry,
            material_name.index,
        );
        storage.set(
            entity,
            nightshade_render::components::material::MaterialRef::new(material_name),
        );

        if let Some(bv) =
            storage.get_mut::<nightshade_render::data::bounding_volume::BoundingVolume>(entity)
        {
            *bv = nightshade_render::data::bounding_volume::BoundingVolume::from_mesh_type("Cube");
        }

        storage.set(
            entity,
            RigidBodyComponent::new_dynamic()
                .with_translation(position.x, position.y, position.z)
                .with_mass(mass),
        );

        storage.set(
            entity,
            ColliderComponent::new_cuboid(size.x / 2.0, size.y / 2.0, size.z / 2.0)
                .with_friction(0.7)
                .with_restitution(0.2),
        );
        mesh_state_mark_entity_added(
            resources.res_mut::<nightshade_render::resources::mesh_state::MeshRenderState>(),
            entity,
        );
    });
    entity
}

pub fn spawn_dynamic_physics_cylinder_with_material(
    commands: &mut Commands<'_>,
    position: Vec3,
    half_height: f32,
    radius: f32,
    mass: f32,
    material: nightshade_render::data::material::Material,
) -> Entity {
    let entity = commands.reserve_entity();
    commands.spawn_reserved(entity, ());
    commands.queue(move |storage, resources| {
        storage.add_components::<PhysicsMeshSet>(entity);

        if let Some(name) = storage.get_mut::<nightshade_scene::contract::primitives::Name>(entity)
        {
            name.0 = "Physics Cylinder".to_string();
        }

        if let Some(transform) =
            storage.get_mut::<nightshade_scene::contract::LocalTransform>(entity)
        {
            transform.translation = position;
            transform.scale = nalgebra_glm::vec3(radius * 2.0, half_height * 2.0, radius * 2.0);
        }

        if let Some(mesh) =
            storage.get_mut::<nightshade_render::components::mesh::RenderMesh>(entity)
        {
            mesh.name = "Cylinder".to_string();
            nightshade_render::commands::primitives::register_primitive_mesh(resources, "Cylinder");
        }

        let fallback_name = format!("DynamicPhysicsCylinder_{}", entity.id);
        let material_name = material_registry_find_or_insert(
            resources
                .res_mut::<nightshade_render::resources::material_registry::MaterialRegistry>(),
            fallback_name,
            material,
        );
        registry_add_reference(
            &mut resources
                .res_mut::<nightshade_render::resources::material_registry::MaterialRegistry>()
                .registry,
            material_name.index,
        );
        storage.set(
            entity,
            nightshade_render::components::material::MaterialRef::new(material_name),
        );

        if let Some(bv) =
            storage.get_mut::<nightshade_render::data::bounding_volume::BoundingVolume>(entity)
        {
            *bv = nightshade_render::data::bounding_volume::BoundingVolume::from_mesh_type(
                "Cylinder",
            );
        }

        storage.set(
            entity,
            RigidBodyComponent::new_dynamic()
                .with_translation(position.x, position.y, position.z)
                .with_mass(mass),
        );

        storage.set(
            entity,
            ColliderComponent::new_cylinder(half_height, radius)
                .with_friction(0.5)
                .with_restitution(0.3),
        );
        mesh_state_mark_entity_added(
            resources.res_mut::<nightshade_render::resources::mesh_state::MeshRenderState>(),
            entity,
        );
    });
    entity
}

pub fn spawn_character_body(commands: &mut Commands<'_>, position: Vec3, name: &str) -> Entity {
    let entity = commands.reserve_entity();
    commands.spawn_reserved(entity, ());
    let name = name.to_string();
    commands.queue(move |storage, _| {
        storage.add_components::<(
            nightshade_scene::contract::primitives::Name,
            nightshade_scene::contract::LocalTransform,
            nightshade_scene::contract::GlobalTransform,
        )>(entity);

        if let Some(label) = storage.get_mut::<nightshade_scene::contract::primitives::Name>(entity)
        {
            label.0 = name;
        }

        if let Some(transform) =
            storage.get_mut::<nightshade_scene::contract::LocalTransform>(entity)
        {
            transform.translation = position;
        }

        let interpolation = crate::contract::components::PhysicsInterpolation {
            previous_translation: position,
            previous_rotation: nalgebra_glm::quat_identity(),
            current_translation: position,
            current_rotation: nalgebra_glm::quat_identity(),
            enabled: true,
        };
        storage.set(entity, interpolation);

        let mut controller = CharacterControllerComponent::new_capsule(0.7, 0.3);
        controller.max_speed = 5.0;
        controller.acceleration = 50.0;
        controller.jump_impulse = 5.0;
        storage.set(entity, controller);
    });
    entity
}

pub fn reset_physics_interpolation(commands: &mut Commands<'_>, entity: Entity) {
    commands.queue(move |storage, _| {
        let Some(transform) = storage
            .get::<nightshade_scene::contract::LocalTransform>(entity)
            .copied()
        else {
            return;
        };
        if let Some(interpolation) =
            storage.get_mut::<crate::contract::components::PhysicsInterpolation>(entity)
        {
            interpolation.previous_translation = transform.translation;
            interpolation.previous_rotation = transform.rotation;
            interpolation.current_translation = transform.translation;
            interpolation.current_rotation = transform.rotation;
        }
    });
}
