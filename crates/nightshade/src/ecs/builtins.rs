//! Built-in entity operations layered above the domains: the standard
//! spawn and despawn helpers, material and texture setup, and the
//! command-processing and resource-cleanup frame systems.

use crate::ecs::material::components::MaterialRef;
use crate::ecs::material::resources::{material_registry_insert, material_registry_remove_unused};
use crate::ecs::world::commands::{
    EcsCommand, generate_checkerboard_texture, generate_gradient_texture, generate_uv_test_texture,
    spawn_entities,
};
use crate::ecs::world::{CORE, Entity, World};
#[cfg(feature = "physics")]
use crate::plugins::physics::resources::{
    physics_world_add_collider, physics_world_add_rigid_body,
};
use crate::prelude::*;
use crate::render::generational_registry::registry_entry_by_name;
use crate::render::mesh_cache::{mesh_cache_insert, mesh_cache_remove_unused};
use crate::render::procedural_meshes::{
    create_cone_mesh, create_cube_mesh, create_cylinder_mesh, create_plane_mesh,
    create_sphere_mesh, create_subdivided_plane_mesh, create_torus_mesh,
};
use crate::render::wgpu::texture_cache::{
    TextureOwner, texture_cache_acquire, texture_cache_release_entity,
};

const CLEANUP_INTERVAL_FRAMES: u64 = 300;

/// Insert a material into the registry, ref-count it, and attach it to `entity` via [`MaterialRef`].
pub fn register_material(
    world: &mut World,
    entity: Entity,
    name: String,
    material: crate::render::material::Material,
) {
    material_registry_insert(
        world.res_mut::<crate::ecs::material::resources::MaterialRegistry>(),
        name.clone(),
        material,
    );
    if let Some(&index) = world
        .res::<crate::ecs::material::resources::MaterialRegistry>()
        .registry
        .name_to_index
        .get(&name)
    {
        registry_add_reference(
            &mut world
                .res_mut::<crate::ecs::material::resources::MaterialRegistry>()
                .registry,
            index,
        );
    }
    world.set(entity, MaterialRef::new(name));
    world
        .res_mut::<crate::render::mesh_state::MeshRenderState>()
        .mark_entity_added(entity);
}

#[cfg(feature = "physics")]
pub fn spawn_physics_body(world: &mut World, entity: Entity) {
    if world.ecs.resource::<PhysicsWorld>().is_none() {
        return;
    }
    let rigid_body_comp = world
        .get::<crate::plugins::physics::components::RigidBodyComponent>(entity)
        .cloned();
    let collider_comp = world
        .get::<crate::plugins::physics::components::ColliderComponent>(entity)
        .cloned();

    if let Some(rigid_body_comp) = rigid_body_comp {
        let rapier_body = rigid_body_comp.to_rapier_rigid_body();
        let rapier_handle =
            physics_world_add_rigid_body(world.plugin_resource_mut::<PhysicsWorld>(), rapier_body);

        if let Some(collider_comp) = collider_comp {
            let rapier_collider = collider_comp.to_rapier_collider();
            physics_world_add_collider(
                world.plugin_resource_mut::<PhysicsWorld>(),
                rapier_collider,
                rapier_handle,
            );
        }

        if let Some(rigid_body_mut) =
            world.get_mut::<crate::plugins::physics::components::RigidBodyComponent>(entity)
        {
            rigid_body_mut.handle =
                Some(crate::plugins::physics::types::rigid_body_handle_from_rapier(rapier_handle));
        }

        world
            .plugin_resource_mut::<PhysicsWorld>()
            .handle_to_entity
            .insert(rapier_handle, entity);
    }
}

pub fn process_commands_system(world: &mut World) {
    let commands = std::mem::take(
        &mut world
            .res_mut::<crate::ecs::world::commands::CommandQueues>()
            .ecs,
    );
    let _span = tracing::info_span!("ecs_commands", count = commands.len()).entered();
    for command in commands {
        match command {
            EcsCommand::DespawnRecursive { entity } => {
                despawn_recursive_immediate(world, entity);
            }
            EcsCommand::ReloadMaterial { name, material } => {
                reload_material_command(world, name, *material);
            }
        }
    }
}

fn reload_material_command(
    world: &mut World,
    name: String,
    new_material: crate::render::material::Material,
) {
    use crate::ecs::world::MATERIAL_REF;

    let new_textures: Vec<String> = new_material.texture_names().map(str::to_string).collect();

    crate::ecs::material::resources::material_registry_mutate(
        world.res_mut::<crate::ecs::material::resources::MaterialRegistry>(),
        &name,
        |existing| *existing = new_material,
    );

    let referencing_entities: Vec<Entity> = world.ecs.worlds[CORE]
        .query_entities(MATERIAL_REF)
        .filter(|&entity| {
            world
                .get::<crate::ecs::material::components::MaterialRef>(entity)
                .is_some_and(|mat_ref| mat_ref.name == name)
        })
        .collect();

    for entity in referencing_entities {
        texture_cache_acquire(
            world.res_mut::<crate::render::wgpu::texture_cache::TextureCache>(),
            TextureOwner::EntityMaterial(entity),
            new_textures.clone(),
        );
        world
            .res_mut::<crate::render::mesh_state::MeshRenderState>()
            .mark_material_dirty(entity);
    }
}

/// Despawn a batch of entities and clean up cached references (textures, materials, meshes).
/// Does not walk the hierarchy; use [`despawn_recursive_immediate`] or [`EcsCommand::DespawnRecursive`] for that.
pub fn despawn_entities_with_cache_cleanup(world: &mut World, entities: &[Entity]) {
    for &entity in entities {
        world
            .res_mut::<crate::ecs::entity_registry::EntityRegistry>()
            .tags
            .remove(&entity);
        if let Some(guid) = world.get::<crate::ecs::primitives::Guid>(entity).copied() {
            world
                .res_mut::<crate::ecs::entity_registry::EntityRegistry>()
                .guid_index
                .remove(&guid.0);
        }

        if let Some(render_mesh) = world.get::<crate::ecs::mesh::components::RenderMesh>(entity) {
            let mesh_name = render_mesh.name.clone();
            if let Some(&index) = world
                .res::<crate::render::mesh_cache::MeshCache>()
                .registry
                .name_to_index
                .get(&mesh_name)
            {
                registry_remove_reference(
                    &mut world
                        .res_mut::<crate::render::mesh_cache::MeshCache>()
                        .registry,
                    index,
                );
            }
            world
                .res_mut::<crate::render::mesh_state::MeshRenderState>()
                .mark_entity_removed(entity);
        }

        if let Some(instanced_mesh) =
            world.get::<crate::ecs::mesh::components::InstancedMesh>(entity)
        {
            let mesh_name = instanced_mesh.mesh_name.clone();
            if let Some(&index) = world
                .res::<crate::render::mesh_cache::MeshCache>()
                .registry
                .name_to_index
                .get(&mesh_name)
            {
                registry_remove_reference(
                    &mut world
                        .res_mut::<crate::render::mesh_cache::MeshCache>()
                        .registry,
                    index,
                );
            }
            world
                .res_mut::<crate::render::mesh_state::MeshRenderState>()
                .mark_instanced_meshes_changed();
        }

        texture_cache_release_entity(
            world.res_mut::<crate::render::wgpu::texture_cache::TextureCache>(),
            entity,
        );

        if let Some(material_ref) =
            world.get::<crate::ecs::material::components::MaterialRef>(entity)
        {
            let material_name = material_ref.name.clone();
            if let Some(&index) = world
                .res::<crate::ecs::material::resources::MaterialRegistry>()
                .registry
                .name_to_index
                .get(&material_name)
            {
                registry_remove_reference(
                    &mut world
                        .res_mut::<crate::ecs::material::resources::MaterialRegistry>()
                        .registry,
                    index,
                );
            }
        }
    }

    world.despawn_entities(entities);
}

/// Despawn `entity` and all of its descendants immediately (no deferral).
pub fn despawn_recursive_immediate(world: &mut World, entity: Entity) {
    crate::ecs::transform::systems::sync_hierarchy_index(world);
    let mut targets = world
        .res::<nightshade_ecs::dynamic::HierarchyIndex>()
        .descendants(entity);
    targets.insert(0, entity);
    despawn_entities_with_cache_cleanup(world, &targets);
}

/// Spawn a default directional light positioned above the origin, with shadows enabled.
pub fn spawn_sun(world: &mut World) -> Entity {
    use crate::ecs::bootstrap::components;
    use crate::ecs::bundles::{LightBundle, SpatialBundle};

    world.ecs.spawn_with(LightBundle {
        spatial: SpatialBundle {
            name: components::Name("Sun".to_string()),
            local_transform: components::LocalTransform {
                translation: nalgebra_glm::Vec3::new(5.0, 10.0, 5.0),
                rotation: nalgebra_glm::quat_angle_axis(
                    std::f32::consts::FRAC_PI_4,
                    &nalgebra_glm::Vec3::new(0.0, 1.0, 0.0),
                ) * nalgebra_glm::quat_angle_axis(
                    -std::f32::consts::FRAC_PI_6,
                    &nalgebra_glm::Vec3::new(1.0, 0.0, 0.0),
                ),
                scale: nalgebra_glm::Vec3::new(1.0, 1.0, 1.0),
            },
            ..Default::default()
        },
        light: components::Light {
            light_type: components::LightType::Directional,
            color: nalgebra_glm::Vec3::new(1.0, 0.95, 0.8),
            intensity: 5.0,
            range: 100.0,
            inner_cone_angle: std::f32::consts::PI / 6.0,
            outer_cone_angle: std::f32::consts::PI / 4.0,
            cast_shadows: true,
            shadow_bias: 0.0005,
            shadow_resolution: 0,
            shadow_distance: 0.0,
            cookie_texture: None,
            ..Default::default()
        },
    })
}

/// Spawn a directional light positioned above the origin without shadow casting.
pub fn spawn_sun_without_shadows(world: &mut World) -> Entity {
    use crate::ecs::bootstrap::components;
    use crate::ecs::bundles::{LightBundle, SpatialBundle};

    world.ecs.spawn_with(LightBundle {
        spatial: SpatialBundle {
            name: components::Name("Sun".to_string()),
            local_transform: components::LocalTransform {
                translation: nalgebra_glm::Vec3::new(5.0, 10.0, 5.0),
                rotation: nalgebra_glm::quat_angle_axis(
                    -std::f32::consts::FRAC_PI_4,
                    &nalgebra_glm::Vec3::new(1.0, 0.0, 0.0),
                ),
                scale: nalgebra_glm::Vec3::new(1.0, 1.0, 1.0),
            },
            ..Default::default()
        },
        light: components::Light {
            light_type: components::LightType::Directional,
            color: nalgebra_glm::Vec3::new(1.0, 0.95, 0.8),
            intensity: 5.0,
            range: 100.0,
            inner_cone_angle: std::f32::consts::PI / 6.0,
            outer_cone_angle: std::f32::consts::PI / 4.0,
            cast_shadows: false,
            shadow_bias: 0.0,
            shadow_resolution: 0,
            shadow_distance: 0.0,
            cookie_texture: None,
            ..Default::default()
        },
    })
}

/// Spawn an entity that references `mesh_name` in the mesh cache, with default material, at `position` scaled by `scale`.
pub fn spawn_mesh_at(
    world: &mut World,
    mesh_name: &str,
    position: nalgebra_glm::Vec3,
    scale: nalgebra_glm::Vec3,
) -> Entity {
    use crate::ecs::bootstrap::components;

    let render_mesh = components::RenderMesh::new(mesh_name);

    if !world
        .res::<crate::render::mesh_cache::MeshCache>()
        .registry
        .name_to_index
        .contains_key(&render_mesh.name)
    {
        let mesh = match mesh_name {
            "Cube" => Some(create_cube_mesh()),
            "Sphere" => Some(create_sphere_mesh(1.0, 16)),
            "Plane" => Some(create_plane_mesh(2.0)),
            "SubdividedPlane" => Some(create_subdivided_plane_mesh(2.0, 20)),
            "Torus" => Some(create_torus_mesh(1.0, 0.3, 32, 16)),
            "Cylinder" => Some(create_cylinder_mesh(0.5, 1.0, 16)),
            "Cone" => Some(create_cone_mesh(0.5, 1.0, 16)),
            _ => None,
        };
        if let Some(mesh) = mesh {
            mesh_cache_insert(
                world.res_mut::<crate::render::mesh_cache::MeshCache>(),
                mesh_name.to_string(),
                mesh,
            );
        }
    }

    if let Some(&index) = world
        .res::<crate::render::mesh_cache::MeshCache>()
        .registry
        .name_to_index
        .get(&render_mesh.name)
    {
        registry_add_reference(
            &mut world
                .res_mut::<crate::render::mesh_cache::MeshCache>()
                .registry,
            index,
        );
    }

    if let Some(&index) = world
        .res::<crate::ecs::material::resources::MaterialRegistry>()
        .registry
        .name_to_index
        .get("Default")
    {
        registry_add_reference(
            &mut world
                .res_mut::<crate::ecs::material::resources::MaterialRegistry>()
                .registry,
            index,
        );
    }

    use crate::ecs::bundles::{MeshBundle, SpatialBundle};

    let entity = world.ecs.spawn_with(MeshBundle {
        spatial: SpatialBundle {
            name: components::Name(mesh_name.to_string()),
            local_transform: components::LocalTransform {
                translation: position,
                scale,
                rotation: nalgebra_glm::Quat::identity(),
            },
            ..Default::default()
        },
        mesh: render_mesh,
        material: components::MaterialRef::new("Default"),
        bounds: components::BoundingVolume::from_mesh_type(mesh_name),
        casts_shadow: components::CastsShadow,
        visibility: crate::ecs::primitives::Visibility { visible: true },
    });
    world
        .res_mut::<crate::render::mesh_state::MeshRenderState>()
        .mark_entity_added(entity);
    crate::ecs::transform::systems::propagate_subtree_global_transforms(world, entity);

    entity
}

/// Spawn a unit cube mesh at `position` with the default material.
pub fn spawn_cube_at(world: &mut World, position: nalgebra_glm::Vec3) -> Entity {
    spawn_mesh_at(
        world,
        "Cube",
        position,
        nalgebra_glm::Vec3::new(1.0, 1.0, 1.0),
    )
}

/// Spawn a unit sphere mesh at `position` with the default material.
pub fn spawn_sphere_at(world: &mut World, position: nalgebra_glm::Vec3) -> Entity {
    spawn_mesh_at(
        world,
        "Sphere",
        position,
        nalgebra_glm::Vec3::new(1.0, 1.0, 1.0),
    )
}

/// Spawn a unit cylinder mesh at `position` with the default material.
pub fn spawn_cylinder_at(world: &mut World, position: nalgebra_glm::Vec3) -> Entity {
    spawn_mesh_at(
        world,
        "Cylinder",
        position,
        nalgebra_glm::Vec3::new(1.0, 1.0, 1.0),
    )
}

/// Spawn a torus mesh at `position` with the default material.
pub fn spawn_torus_at(world: &mut World, position: nalgebra_glm::Vec3) -> Entity {
    spawn_mesh_at(
        world,
        "Torus",
        position,
        nalgebra_glm::Vec3::new(1.0, 1.0, 1.0),
    )
}

/// Spawn a cone mesh at `position` with the default material.
pub fn spawn_cone_at(world: &mut World, position: nalgebra_glm::Vec3) -> Entity {
    spawn_mesh_at(
        world,
        "Cone",
        position,
        nalgebra_glm::Vec3::new(1.0, 1.0, 1.0),
    )
}

/// Spawn a unit-sized plane mesh at `position` with the default material.
pub fn spawn_plane_at(world: &mut World, position: nalgebra_glm::Vec3) -> Entity {
    spawn_mesh_at(
        world,
        "Plane",
        position,
        nalgebra_glm::Vec3::new(1.0, 1.0, 1.0),
    )
}

/// Spawn a 3D text entity at `position` with the supplied text and properties.
pub fn spawn_3d_text_with_properties(
    world: &mut World,
    text: &str,
    position: nalgebra_glm::Vec3,
    properties: crate::ecs::text::components::TextProperties,
) -> Entity {
    spawn_3d_text_impl(world, text, position, properties, false)
}

/// Spawn a 3D billboard text entity that always faces the active camera.
pub fn spawn_3d_billboard_text_with_properties(
    world: &mut World,
    text: &str,
    position: nalgebra_glm::Vec3,
    properties: crate::ecs::text::components::TextProperties,
) -> Entity {
    spawn_3d_text_impl(world, text, position, properties, true)
}

fn spawn_3d_text_impl(
    world: &mut World,
    text: &str,
    position: nalgebra_glm::Vec3,
    properties: crate::ecs::text::components::TextProperties,
    billboard: bool,
) -> Entity {
    use crate::ecs::bootstrap::components;
    use crate::ecs::world::{GLOBAL_TRANSFORM, LOCAL_TRANSFORM, NAME, TEXT, VISIBILITY};

    let text_index = world
        .res_mut::<crate::ecs::text::resources::TextState>()
        .cache
        .add_text(text);

    let entity = spawn_entities(
        world,
        NAME | LOCAL_TRANSFORM | GLOBAL_TRANSFORM | TEXT | VISIBILITY,
        1,
    )[0];

    world.set(entity, components::Name(text.to_string()));
    world.set(
        entity,
        components::LocalTransform {
            translation: position,
            rotation: nalgebra_glm::Quat::identity(),
            scale: nalgebra_glm::Vec3::new(1.0, 1.0, 1.0),
        },
    );
    world.set(entity, components::GlobalTransform::default());
    world.set(
        entity,
        components::Text {
            text_index,
            properties,
            dirty: true,
            cached_mesh: None,
            billboard,
        },
    );
    world.set(entity, components::Visibility { visible: true });

    entity
}

pub fn cleanup_unused_resources_system(world: &mut World) {
    let _span = tracing::info_span!("cleanup").entered();
    world
        .res_mut::<crate::ecs::world::cleanup::CleanupState>()
        .frame_counter += 1;

    if world
        .res::<crate::ecs::world::cleanup::CleanupState>()
        .frame_counter
        >= CLEANUP_INTERVAL_FRAMES
    {
        world
            .res_mut::<crate::ecs::world::cleanup::CleanupState>()
            .frame_counter = 0;
        mesh_cache_remove_unused(world.res_mut::<crate::render::mesh_cache::MeshCache>());
        material_registry_remove_unused(
            world.res_mut::<crate::ecs::material::resources::MaterialRegistry>(),
        );
    }
}

pub fn set_material_with_textures(
    world: &mut World,
    entity: Entity,
    material: crate::render::material::Material,
) {
    let material_ref = world
        .get::<crate::ecs::material::components::MaterialRef>(entity)
        .cloned();

    let texture_names: Vec<String> = material.texture_names().map(str::to_string).collect();
    texture_cache_acquire(
        world.res_mut::<crate::render::wgpu::texture_cache::TextureCache>(),
        TextureOwner::EntityMaterial(entity),
        texture_names,
    );

    if let Some(ref mat_ref) = material_ref {
        crate::ecs::material::resources::material_registry_mutate(
            world.res_mut::<crate::ecs::material::resources::MaterialRegistry>(),
            &mat_ref.name,
            |existing_mat| *existing_mat = material,
        );
        world
            .res_mut::<crate::render::mesh_state::MeshRenderState>()
            .mark_material_dirty(entity);
    } else {
        let material_name = format!("TexturedMaterial_{}", entity.id);
        material_registry_insert(
            world.res_mut::<crate::ecs::material::resources::MaterialRegistry>(),
            material_name.clone(),
            material,
        );
        if let Some(&index) = world
            .res::<crate::ecs::material::resources::MaterialRegistry>()
            .registry
            .name_to_index
            .get(&material_name)
        {
            registry_add_reference(
                &mut world
                    .res_mut::<crate::ecs::material::resources::MaterialRegistry>()
                    .registry,
                index,
            );
        }
        world.set(
            entity,
            crate::ecs::material::components::MaterialRef::new(material_name),
        );
    }
}

pub fn load_procedural_textures(world: &mut World) {
    let default_sampler = crate::render::texture_data::SamplerSettings::DEFAULT;
    let color = crate::render::texture_data::TextureUsage::Color;

    let checkerboard = generate_checkerboard_texture();
    crate::ecs::loading::queue_decoded_texture(
        world,
        "checkerboard".to_string(),
        checkerboard.0,
        checkerboard.1,
        checkerboard.2,
        color,
        default_sampler,
    );

    let gradient = generate_gradient_texture();
    crate::ecs::loading::queue_decoded_texture(
        world,
        "gradient".to_string(),
        gradient.0,
        gradient.1,
        gradient.2,
        color,
        default_sampler,
    );

    let uv_test = generate_uv_test_texture();
    crate::ecs::loading::queue_decoded_texture(
        world,
        "uv_test".to_string(),
        uv_test.0,
        uv_test.1,
        uv_test.2,
        color,
        default_sampler,
    );

    for name in ["checkerboard", "gradient", "uv_test"] {
        crate::render::wgpu::texture_cache::texture_cache_protect(
            world.res_mut::<crate::render::wgpu::texture_cache::TextureCache>(),
            name.to_string(),
        );
    }
}

/// Spawn an entity that draws `count` instanced copies of `mesh_name` using per-instance transforms.
pub fn spawn_instanced_mesh(
    world: &mut World,
    mesh_name: &str,
    instances: Vec<crate::ecs::mesh::components::InstanceTransform>,
) -> Entity {
    use crate::ecs::bootstrap::components;
    use crate::ecs::world::{
        CASTS_SHADOW, GLOBAL_TRANSFORM, INSTANCED_MESH, LOCAL_TRANSFORM, MATERIAL_REF, NAME,
        VISIBILITY,
    };

    let entity = spawn_entities(
        world,
        NAME | LOCAL_TRANSFORM
            | GLOBAL_TRANSFORM
            | INSTANCED_MESH
            | MATERIAL_REF
            | VISIBILITY
            | CASTS_SHADOW,
        1,
    )[0];

    world.set(entity, components::Name(format!("Instanced {}", mesh_name)));
    world.set(
        entity,
        components::LocalTransform {
            translation: nalgebra_glm::Vec3::new(0.0, 0.0, 0.0),
            scale: nalgebra_glm::Vec3::new(1.0, 1.0, 1.0),
            rotation: nalgebra_glm::Quat::identity(),
        },
    );
    world.set(entity, components::GlobalTransform::default());

    world.set(
        entity,
        crate::ecs::mesh::components::InstancedMesh::with_instances(mesh_name, instances),
    );

    if let Some(&index) = world
        .res::<crate::ecs::material::resources::MaterialRegistry>()
        .registry
        .name_to_index
        .get("Default")
    {
        registry_add_reference(
            &mut world
                .res_mut::<crate::ecs::material::resources::MaterialRegistry>()
                .registry,
            index,
        );
    }
    world.set(entity, components::MaterialRef::new("Default"));

    world.set(entity, components::Visibility { visible: true });
    world.set(entity, components::CastsShadow);

    entity
}

/// Spawn an instanced-mesh entity using `mesh_name` and the supplied material.
pub fn spawn_instanced_mesh_with_material(
    world: &mut World,
    mesh_name: &str,
    instances: Vec<crate::ecs::mesh::components::InstanceTransform>,
    material_name: &str,
) -> Entity {
    use crate::ecs::bootstrap::components;
    use crate::ecs::world::{
        CASTS_SHADOW, GLOBAL_TRANSFORM, INSTANCED_MESH, LOCAL_TRANSFORM, MATERIAL_REF, NAME,
        VISIBILITY,
    };

    let entity = spawn_entities(
        world,
        NAME | LOCAL_TRANSFORM
            | GLOBAL_TRANSFORM
            | INSTANCED_MESH
            | MATERIAL_REF
            | VISIBILITY
            | CASTS_SHADOW,
        1,
    )[0];

    world.set(entity, components::Name(format!("Instanced {}", mesh_name)));
    world.set(
        entity,
        components::LocalTransform {
            translation: nalgebra_glm::Vec3::new(0.0, 0.0, 0.0),
            scale: nalgebra_glm::Vec3::new(1.0, 1.0, 1.0),
            rotation: nalgebra_glm::Quat::identity(),
        },
    );
    world.set(entity, components::GlobalTransform::default());

    world.set(
        entity,
        crate::ecs::mesh::components::InstancedMesh::with_instances(mesh_name, instances),
    );

    if let Some(&index) = world
        .res::<crate::ecs::material::resources::MaterialRegistry>()
        .registry
        .name_to_index
        .get(material_name)
    {
        registry_add_reference(
            &mut world
                .res_mut::<crate::ecs::material::resources::MaterialRegistry>()
                .registry,
            index,
        );
    }

    let texture_names: Vec<String> = registry_entry_by_name(
        &world
            .res::<crate::ecs::material::resources::MaterialRegistry>()
            .registry,
        material_name,
    )
    .map(|mat| mat.texture_names().map(str::to_string).collect())
    .unwrap_or_default();
    texture_cache_acquire(
        world.res_mut::<crate::render::wgpu::texture_cache::TextureCache>(),
        TextureOwner::EntityMaterial(entity),
        texture_names,
    );

    world.set(entity, components::MaterialRef::new(material_name));

    world.set(entity, components::Visibility { visible: true });
    world.set(entity, components::CastsShadow);

    entity
}

/// Register `material` in the registry under `name` and attach it to `entity` via [`MaterialRef`].
pub fn spawn_material(
    world: &mut World,
    entity: Entity,
    name: String,
    material: crate::render::material::Material,
) {
    material_registry_insert(
        world.res_mut::<crate::ecs::material::resources::MaterialRegistry>(),
        name.clone(),
        material,
    );
    if let Some(&index) = world
        .res::<crate::ecs::material::resources::MaterialRegistry>()
        .registry
        .name_to_index
        .get(&name)
    {
        registry_add_reference(
            &mut world
                .res_mut::<crate::ecs::material::resources::MaterialRegistry>()
                .registry,
            index,
        );
    }
    world.set(entity, MaterialRef::new(name));
}
