use crate::ecs::generational_registry::{registry_entry_by_name, registry_entry_by_name_mut};
use crate::ecs::graphics::Atmosphere;
use crate::ecs::material::components::MaterialRef;
use crate::ecs::material::resources::{material_registry_insert, material_registry_remove_unused};
use crate::ecs::mesh::components::{
    create_cone_mesh, create_cube_mesh, create_cylinder_mesh, create_plane_mesh,
    create_sphere_mesh, create_subdivided_plane_mesh, create_torus_mesh,
};
use crate::ecs::prefab::resources::mesh_cache_insert;
use crate::ecs::world::{Entity, NAME, World};
use crate::render::wgpu::texture_cache::{
    texture_cache_add_reference, texture_cache_remove_reference, texture_cache_remove_unused,
};

use crate::prelude::*;
#[derive(Debug, Clone)]
pub enum EcsCommand {
    DespawnRecursive {
        entity: Entity,
    },
    ReloadMaterial {
        name: String,
        material: Box<crate::ecs::material::components::Material>,
    },
}

#[derive(Debug, Clone)]
pub enum RenderCommand {
    UploadUiImageLayer {
        layer: u32,
        rgba_data: Vec<u8>,
        width: u32,
        height: u32,
    },
    LoadHdrSkybox {
        hdr_data: Vec<u8>,
    },
    LoadHdrSkyboxFromPath {
        path: std::path::PathBuf,
    },
    CaptureProceduralAtmosphereIBL {
        atmosphere: Atmosphere,
        time: f32,
    },
    CaptureIblSnapshots {
        atmosphere: Atmosphere,
        hours: Vec<f32>,
    },
    CaptureScreenshot {
        path: Option<std::path::PathBuf>,
        /// When set, the saved PNG is downscaled (preserving aspect) so
        /// the longer side is at most this many pixels. Used for prefab
        /// thumbnails where a full-resolution screenshot is wasteful.
        max_dimension: Option<u32>,
    },
    ReloadTexture {
        name: String,
        rgba_data: Vec<u8>,
        width: u32,
        height: u32,
    },
}

#[derive(Default)]
pub struct CommandQueues {
    pub ecs: Vec<EcsCommand>,
    pub render: Vec<RenderCommand>,
}

/// Spawn `count` entities in the `core` archetype with the supplied component mask.
pub fn spawn_entities(world: &mut World, core_mask: u64, count: usize) -> Vec<Entity> {
    let entities = world.spawn_count(count);
    for &entity in &entities {
        world.core.add_components(entity, core_mask);
    }
    entities
}

/// Spawn `count` entities in the retained-UI archetype with the supplied component mask.
pub fn spawn_entities_ui(world: &mut World, ui_mask: u64, count: usize) -> Vec<Entity> {
    let entities = world.spawn_count(count);
    for &entity in &entities {
        world.ui.add_components(entity, ui_mask);
    }
    entities
}

/// Queue an [`EcsCommand`] to be drained by `process_commands_system` later in the frame schedule.
pub fn queue_ecs_command(world: &mut World, command: EcsCommand) {
    world.resources.commands.ecs.push(command);
}

/// Queue a [`RenderCommand`] to be drained by the renderer at frame setup.
pub fn queue_render_command(world: &mut World, command: RenderCommand) {
    world.resources.commands.render.push(command);
}

/// Lock the OS cursor to the window (relative mouse mode). Used for first-person controls.
pub fn set_cursor_locked(world: &mut World, locked: bool) {
    if let Some(window_handle) = &world.resources.window.handle {
        if locked {
            if window_handle
                .set_cursor_grab(winit::window::CursorGrabMode::Locked)
                .is_err()
            {
                let _ = window_handle.set_cursor_grab(winit::window::CursorGrabMode::Confined);
            }
        } else {
            let _ = window_handle.set_cursor_grab(winit::window::CursorGrabMode::None);
        }
    }
}

/// Show or hide the OS cursor.
pub fn set_cursor_visible(world: &mut World, visible: bool) {
    if let Some(window_handle) = &world.resources.window.handle {
        window_handle.set_cursor_visible(visible);
    }
}

/// Insert a material into the registry, ref-count it, and attach it to `entity` via [`MaterialRef`].
pub fn register_material(
    world: &mut World,
    entity: Entity,
    name: String,
    material: crate::ecs::material::components::Material,
) {
    material_registry_insert(
        &mut world.resources.assets.material_registry,
        name.clone(),
        material,
    );
    if let Some(&index) = world
        .resources
        .assets
        .material_registry
        .registry
        .name_to_index
        .get(&name)
    {
        registry_add_reference(
            &mut world.resources.assets.material_registry.registry,
            index,
        );
    }
    world.core.set_material_ref(entity, MaterialRef::new(name));
    world.resources.mesh_render_state.mark_entity_added(entity);
}

#[cfg(feature = "physics")]
pub fn spawn_physics_body(world: &mut World, entity: Entity) {
    let rigid_body_comp = world.core.get_rigid_body(entity).cloned();
    let collider_comp = world.core.get_collider(entity).cloned();

    if let Some(rigid_body_comp) = rigid_body_comp {
        let rapier_body = rigid_body_comp.to_rapier_rigid_body();
        let rapier_handle = physics_world_add_rigid_body(&mut world.resources.physics, rapier_body);

        if let Some(collider_comp) = collider_comp {
            let rapier_collider = collider_comp.to_rapier_collider();
            physics_world_add_collider(
                &mut world.resources.physics,
                rapier_collider,
                rapier_handle,
            );
        }

        if let Some(rigid_body_mut) = world.core.get_rigid_body_mut(entity) {
            rigid_body_mut.handle = Some(rapier_handle.into());
        }

        world
            .resources
            .physics
            .handle_to_entity
            .insert(rapier_handle, entity);
    }
}

pub fn process_commands_system(world: &mut World) {
    let commands = std::mem::take(&mut world.resources.commands.ecs);
    let _span = tracing::info_span!("ecs_commands", count = commands.len()).entered();
    for command in commands {
        match command {
            EcsCommand::DespawnRecursive { entity } => {
                if !world.resources.transform_state.children_cache_valid {
                    validate_and_rebuild_children_cache(world);
                }
                let descendents = query_descendents(world, entity);
                despawn_entities_with_cache_cleanup(world, &descendents);
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
    new_material: crate::ecs::material::components::Material,
) {
    use crate::ecs::world::MATERIAL_REF;

    let old_textures: Vec<String> =
        registry_entry_by_name(&world.resources.assets.material_registry.registry, &name)
            .map(|mat| mat.texture_names().map(str::to_string).collect())
            .unwrap_or_default();
    for texture in &old_textures {
        texture_cache_remove_reference(&mut world.resources.texture_cache, texture);
    }

    for texture in new_material.texture_names() {
        texture_cache_add_reference(&mut world.resources.texture_cache, texture);
    }

    if let Some(existing) = registry_entry_by_name_mut(
        &mut world.resources.assets.material_registry.registry,
        &name,
    ) {
        *existing = new_material;
    }

    let entities_to_dirty: Vec<Entity> = world
        .core
        .query_entities(MATERIAL_REF)
        .filter(|&entity| {
            world
                .core
                .get_material_ref(entity)
                .is_some_and(|mat_ref| mat_ref.name == name)
        })
        .collect();

    for entity in entities_to_dirty {
        world
            .resources
            .mesh_render_state
            .mark_material_dirty(entity);
    }
}

fn query_children(world: &World, entity: Entity) -> Vec<Entity> {
    world
        .resources
        .transform_state
        .children_cache
        .get(&entity)
        .cloned()
        .unwrap_or_default()
}

fn query_descendents(world: &World, target_entity: Entity) -> Vec<Entity> {
    let mut descendents = Vec::new();
    let mut stack = vec![target_entity];
    while let Some(entity) = stack.pop() {
        descendents.push(entity);
        for child in query_children(world, entity) {
            stack.push(child);
        }
    }
    descendents
}

/// Despawn a batch of entities and clean up cached references (textures, materials, meshes).
/// Does not walk the hierarchy; use [`despawn_recursive_immediate`] or [`EcsCommand::DespawnRecursive`] for that.
pub fn despawn_entities_with_cache_cleanup(world: &mut World, entities: &[Entity]) {
    let mut has_parent_entities = false;
    let is_runtime = world.resources.is_runtime;

    for &entity in entities {
        world
            .resources
            .transform_state
            .children_cache
            .remove(&entity);
        world.resources.entities.tags.remove(&entity);
        if let Some(guid) = world.core.get_guid(entity).copied() {
            world.resources.entities.guid_index.remove(&guid.0);
        }

        if let Some(parent_component) = world.core.get_parent(entity)
            && let Some(parent_entity) = parent_component.0
            && let Some(children) = world
                .resources
                .transform_state
                .children_cache
                .get_mut(&parent_entity)
        {
            children.retain(|&e| e != entity);
            has_parent_entities = true;
        }

        if let Some(render_mesh) = world.core.get_render_mesh(entity) {
            if !is_runtime {
                let mesh_name = render_mesh.name.clone();
                if let Some(&index) = world
                    .resources
                    .assets
                    .mesh_cache
                    .registry
                    .name_to_index
                    .get(&mesh_name)
                {
                    registry_remove_reference(
                        &mut world.resources.assets.mesh_cache.registry,
                        index,
                    );
                }
            }
            world
                .resources
                .mesh_render_state
                .mark_entity_removed(entity);
        }

        if !is_runtime {
            if let Some(instanced_mesh) = world.core.get_instanced_mesh(entity) {
                let mesh_name = instanced_mesh.mesh_name.clone();
                if let Some(&index) = world
                    .resources
                    .assets
                    .mesh_cache
                    .registry
                    .name_to_index
                    .get(&mesh_name)
                {
                    registry_remove_reference(
                        &mut world.resources.assets.mesh_cache.registry,
                        index,
                    );
                }
            }

            if let Some(material_ref) = world.core.get_material_ref(entity) {
                let material_name = material_ref.name.clone();
                let texture_names: Vec<String> = registry_entry_by_name(
                    &world.resources.assets.material_registry.registry,
                    &material_name,
                )
                .map(|material| material.texture_names().map(str::to_string).collect())
                .unwrap_or_default();
                for texture in &texture_names {
                    texture_cache_remove_reference(&mut world.resources.texture_cache, texture);
                }
                if let Some(&index) = world
                    .resources
                    .assets
                    .material_registry
                    .registry
                    .name_to_index
                    .get(&material_name)
                {
                    registry_remove_reference(
                        &mut world.resources.assets.material_registry.registry,
                        index,
                    );
                }
            }
        }
    }

    if has_parent_entities {
        world.resources.transform_state.children_cache_valid = false;
    }

    world.despawn_entities(entities);
}

/// Despawn `entity` and all of its descendants immediately (no deferral).
pub fn despawn_recursive_immediate(world: &mut World, entity: Entity) {
    if !world.resources.transform_state.children_cache_valid {
        validate_and_rebuild_children_cache(world);
    }
    let descendents = query_descendents(world, entity);
    despawn_entities_with_cache_cleanup(world, &descendents);
}

/// Spawn a default directional light positioned above the origin, with shadows enabled.
pub fn spawn_sun(world: &mut World) -> Entity {
    use crate::ecs::world::components;
    use crate::ecs::world::{
        GLOBAL_TRANSFORM, LIGHT, LOCAL_TRANSFORM, LOCAL_TRANSFORM_DIRTY, NAME,
    };

    let entity = spawn_entities(
        world,
        NAME | LOCAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY | GLOBAL_TRANSFORM | LIGHT,
        1,
    )[0];

    world
        .core
        .set_name(entity, components::Name("Sun".to_string()));
    world.core.set_local_transform(
        entity,
        components::LocalTransform {
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
    );
    world
        .core
        .set_local_transform_dirty(entity, components::LocalTransformDirty);
    world
        .core
        .set_global_transform(entity, components::GlobalTransform::default());
    world.core.set_light(
        entity,
        components::Light {
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
        },
    );

    entity
}

/// Spawn a directional light positioned above the origin without shadow casting.
pub fn spawn_sun_without_shadows(world: &mut World) -> Entity {
    use crate::ecs::world::components;
    use crate::ecs::world::{
        GLOBAL_TRANSFORM, LIGHT, LOCAL_TRANSFORM, LOCAL_TRANSFORM_DIRTY, NAME,
    };

    let entity = spawn_entities(
        world,
        NAME | LOCAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY | GLOBAL_TRANSFORM | LIGHT,
        1,
    )[0];

    world
        .core
        .set_name(entity, components::Name("Sun".to_string()));
    world.core.set_local_transform(
        entity,
        components::LocalTransform {
            translation: nalgebra_glm::Vec3::new(5.0, 10.0, 5.0),
            rotation: nalgebra_glm::quat_angle_axis(
                -std::f32::consts::FRAC_PI_4,
                &nalgebra_glm::Vec3::new(1.0, 0.0, 0.0),
            ),
            scale: nalgebra_glm::Vec3::new(1.0, 1.0, 1.0),
        },
    );
    world
        .core
        .set_local_transform_dirty(entity, components::LocalTransformDirty);
    world
        .core
        .set_global_transform(entity, components::GlobalTransform::default());
    world.core.set_light(
        entity,
        components::Light {
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
        },
    );

    entity
}

/// Spawn an entity that references `mesh_name` in the mesh cache, with default material, at `position` scaled by `scale`.
pub fn spawn_mesh_at(
    world: &mut World,
    mesh_name: &str,
    position: nalgebra_glm::Vec3,
    scale: nalgebra_glm::Vec3,
) -> Entity {
    use crate::ecs::world::components;
    use crate::ecs::world::{
        BOUNDING_VOLUME, CASTS_SHADOW, GLOBAL_TRANSFORM, LOCAL_TRANSFORM, LOCAL_TRANSFORM_DIRTY,
        MATERIAL_REF, NAME, RENDER_MESH, VISIBILITY,
    };

    let entity = spawn_entities(
        world,
        NAME | LOCAL_TRANSFORM
            | GLOBAL_TRANSFORM
            | LOCAL_TRANSFORM_DIRTY
            | RENDER_MESH
            | MATERIAL_REF
            | BOUNDING_VOLUME
            | CASTS_SHADOW
            | VISIBILITY,
        1,
    )[0];

    world
        .core
        .set_name(entity, components::Name(mesh_name.to_string()));
    world.core.set_local_transform(
        entity,
        components::LocalTransform {
            translation: position,
            scale,
            rotation: nalgebra_glm::Quat::identity(),
        },
    );
    world
        .core
        .set_global_transform(entity, components::GlobalTransform::default());
    world
        .core
        .set_local_transform_dirty(entity, components::LocalTransformDirty);
    let render_mesh = components::RenderMesh::new(mesh_name);

    if !world
        .resources
        .assets
        .mesh_cache
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
                &mut world.resources.assets.mesh_cache,
                mesh_name.to_string(),
                mesh,
            );
        }
    }

    if let Some(&index) = world
        .resources
        .assets
        .mesh_cache
        .registry
        .name_to_index
        .get(&render_mesh.name)
    {
        registry_add_reference(&mut world.resources.assets.mesh_cache.registry, index);
    }

    world.core.set_render_mesh(entity, render_mesh);
    world.resources.mesh_render_state.mark_entity_added(entity);

    if let Some(&index) = world
        .resources
        .assets
        .material_registry
        .registry
        .name_to_index
        .get("Default")
    {
        registry_add_reference(
            &mut world.resources.assets.material_registry.registry,
            index,
        );
    }
    world
        .core
        .set_material_ref(entity, components::MaterialRef::new("Default"));

    world.core.set_bounding_volume(
        entity,
        components::BoundingVolume::from_mesh_type(mesh_name),
    );
    world.core.set_casts_shadow(entity, components::CastsShadow);

    if let Some(visibility) = world.core.get_visibility_mut(entity) {
        visibility.visible = true;
    }

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
    use crate::ecs::world::components;
    use crate::ecs::world::{
        GLOBAL_TRANSFORM, LOCAL_TRANSFORM, LOCAL_TRANSFORM_DIRTY, NAME, TEXT, VISIBILITY,
    };

    let text_index = world.resources.text.cache.add_text(text);

    let entity = spawn_entities(
        world,
        NAME | LOCAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY | GLOBAL_TRANSFORM | TEXT | VISIBILITY,
        1,
    )[0];

    world
        .core
        .set_name(entity, components::Name(text.to_string()));
    world.core.set_local_transform(
        entity,
        components::LocalTransform {
            translation: position,
            rotation: nalgebra_glm::Quat::identity(),
            scale: nalgebra_glm::Vec3::new(1.0, 1.0, 1.0),
        },
    );
    world
        .core
        .set_local_transform_dirty(entity, components::LocalTransformDirty);
    world
        .core
        .set_global_transform(entity, components::GlobalTransform::default());
    world.core.set_text(
        entity,
        components::Text {
            text_index,
            properties,
            dirty: true,
            cached_mesh: None,
            billboard,
        },
    );
    world
        .core
        .set_visibility(entity, components::Visibility { visible: true });

    entity
}

const CLEANUP_INTERVAL_FRAMES: u64 = 300;

pub fn cleanup_unused_resources_system(world: &mut World) {
    let _span = tracing::info_span!("cleanup").entered();
    world.resources.cleanup.frame_counter += 1;

    if world.resources.cleanup.frame_counter >= CLEANUP_INTERVAL_FRAMES {
        world.resources.cleanup.frame_counter = 0;
        registry_remove_unused(&mut world.resources.assets.mesh_cache.registry);
        material_registry_remove_unused(&mut world.resources.assets.material_registry);
        texture_cache_remove_unused(&mut world.resources.texture_cache);
    }
}

pub fn set_material_with_textures(
    world: &mut World,
    entity: Entity,
    material: crate::ecs::material::components::Material,
) {
    let material_ref = world.core.get_material_ref(entity).cloned();

    if let Some(ref mat_ref) = material_ref {
        let old_textures: Vec<String> = registry_entry_by_name(
            &world.resources.assets.material_registry.registry,
            &mat_ref.name,
        )
        .map(|mat| mat.texture_names().map(str::to_string).collect())
        .unwrap_or_default();
        for texture in &old_textures {
            texture_cache_remove_reference(&mut world.resources.texture_cache, texture);
        }
    }

    for texture in material.texture_names() {
        texture_cache_add_reference(&mut world.resources.texture_cache, texture);
    }

    if let Some(ref mat_ref) = material_ref {
        if let Some(existing_mat) = registry_entry_by_name_mut(
            &mut world.resources.assets.material_registry.registry,
            &mat_ref.name,
        ) {
            *existing_mat = material;
        }
    } else {
        let material_name = format!("TexturedMaterial_{}", entity.id);
        material_registry_insert(
            &mut world.resources.assets.material_registry,
            material_name.clone(),
            material,
        );
        if let Some(&index) = world
            .resources
            .assets
            .material_registry
            .registry
            .name_to_index
            .get(&material_name)
        {
            registry_add_reference(
                &mut world.resources.assets.material_registry.registry,
                index,
            );
        }
        world.core.set_material_ref(
            entity,
            crate::ecs::material::components::MaterialRef::new(material_name),
        );
    }
}

pub fn load_procedural_textures(world: &mut World) {
    let default_sampler = crate::render::wgpu::texture_cache::SamplerSettings::DEFAULT;
    let color = crate::render::wgpu::texture_cache::TextureUsage::Color;

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
}

fn generate_checkerboard_texture() -> (Vec<u8>, u32, u32) {
    let width = 256;
    let height = 256;
    let checker_size = 32;

    let mut pixels = Vec::new();
    for y in 0..height {
        for x in 0..width {
            let checker_x = (x / checker_size) % 2;
            let checker_y = (y / checker_size) % 2;
            let is_white = (checker_x + checker_y) % 2 == 0;

            if is_white {
                pixels.extend_from_slice(&[255, 255, 255, 255]);
            } else {
                pixels.extend_from_slice(&[64, 64, 64, 255]);
            }
        }
    }

    (pixels, width, height)
}

fn generate_gradient_texture() -> (Vec<u8>, u32, u32) {
    let width = 256;
    let height = 256;

    let mut pixels = Vec::new();
    for y in 0..height {
        for x in 0..width {
            let r = (x * 255 / width) as u8;
            let g = (y * 255 / height) as u8;
            let b = 128u8;
            pixels.extend_from_slice(&[r, g, b, 255]);
        }
    }

    (pixels, width, height)
}

fn generate_uv_test_texture() -> (Vec<u8>, u32, u32) {
    let width = 256;
    let height = 256;

    let mut pixels = Vec::new();
    for y in 0..height {
        for x in 0..width {
            let u = x as f32 / width as f32;
            let v = y as f32 / height as f32;

            let r = (u * 255.0) as u8;
            let g = (v * 255.0) as u8;
            let b = ((1.0 - u) * (1.0 - v) * 255.0) as u8;

            pixels.extend_from_slice(&[r, g, b, 255]);
        }
    }

    (pixels, width, height)
}

pub fn load_hdr_skybox(world: &mut World, hdr_data: Vec<u8>) {
    queue_render_command(world, RenderCommand::LoadHdrSkybox { hdr_data });
}

pub fn load_hdr_skybox_from_path(world: &mut World, path: std::path::PathBuf) {
    queue_render_command(world, RenderCommand::LoadHdrSkyboxFromPath { path });
}

pub fn capture_procedural_atmosphere_ibl(world: &mut World, atmosphere: Atmosphere, time: f32) {
    queue_render_command(
        world,
        RenderCommand::CaptureProceduralAtmosphereIBL { atmosphere, time },
    );
}

pub fn capture_ibl_snapshots(world: &mut World, atmosphere: Atmosphere, hours: Vec<f32>) {
    queue_render_command(
        world,
        RenderCommand::CaptureIblSnapshots { atmosphere, hours },
    );
}

#[cfg(not(target_arch = "wasm32"))]
pub fn capture_screenshot(world: &mut World) {
    queue_render_command(
        world,
        RenderCommand::CaptureScreenshot {
            path: None,
            max_dimension: None,
        },
    );
}

#[cfg(not(target_arch = "wasm32"))]
pub fn capture_screenshot_to_path(world: &mut World, path: impl Into<std::path::PathBuf>) {
    queue_render_command(
        world,
        RenderCommand::CaptureScreenshot {
            path: Some(path.into()),
            max_dimension: None,
        },
    );
}

#[cfg(not(target_arch = "wasm32"))]
pub fn capture_thumbnail_to_path(
    world: &mut World,
    path: impl Into<std::path::PathBuf>,
    max_dimension: u32,
) {
    queue_render_command(
        world,
        RenderCommand::CaptureScreenshot {
            path: Some(path.into()),
            max_dimension: Some(max_dimension),
        },
    );
}

/// Spawn an entity that draws `count` instanced copies of `mesh_name` using per-instance transforms.
pub fn spawn_instanced_mesh(
    world: &mut World,
    mesh_name: &str,
    instances: Vec<crate::ecs::mesh::components::InstanceTransform>,
) -> Entity {
    use crate::ecs::world::components;
    use crate::ecs::world::{
        CASTS_SHADOW, GLOBAL_TRANSFORM, INSTANCED_MESH, LOCAL_TRANSFORM, LOCAL_TRANSFORM_DIRTY,
        MATERIAL_REF, NAME, VISIBILITY,
    };

    let entity = spawn_entities(
        world,
        NAME | LOCAL_TRANSFORM
            | GLOBAL_TRANSFORM
            | LOCAL_TRANSFORM_DIRTY
            | INSTANCED_MESH
            | MATERIAL_REF
            | VISIBILITY
            | CASTS_SHADOW,
        1,
    )[0];

    world
        .core
        .set_name(entity, components::Name(format!("Instanced {}", mesh_name)));
    world.core.set_local_transform(
        entity,
        components::LocalTransform {
            translation: nalgebra_glm::Vec3::new(0.0, 0.0, 0.0),
            scale: nalgebra_glm::Vec3::new(1.0, 1.0, 1.0),
            rotation: nalgebra_glm::Quat::identity(),
        },
    );
    world
        .core
        .set_global_transform(entity, components::GlobalTransform::default());
    world
        .core
        .set_local_transform_dirty(entity, components::LocalTransformDirty);

    world.core.set_instanced_mesh(
        entity,
        crate::ecs::mesh::components::InstancedMesh::with_instances(mesh_name, instances),
    );

    if let Some(&index) = world
        .resources
        .assets
        .material_registry
        .registry
        .name_to_index
        .get("Default")
    {
        registry_add_reference(
            &mut world.resources.assets.material_registry.registry,
            index,
        );
    }
    world
        .core
        .set_material_ref(entity, components::MaterialRef::new("Default"));

    world
        .core
        .set_visibility(entity, components::Visibility { visible: true });
    world.core.set_casts_shadow(entity, components::CastsShadow);

    entity
}

/// Spawn an instanced-mesh entity using `mesh_name` and the supplied material.
pub fn spawn_instanced_mesh_with_material(
    world: &mut World,
    mesh_name: &str,
    instances: Vec<crate::ecs::mesh::components::InstanceTransform>,
    material_name: &str,
) -> Entity {
    use crate::ecs::world::components;
    use crate::ecs::world::{
        CASTS_SHADOW, GLOBAL_TRANSFORM, INSTANCED_MESH, LOCAL_TRANSFORM, LOCAL_TRANSFORM_DIRTY,
        MATERIAL_REF, NAME, VISIBILITY,
    };

    let entity = spawn_entities(
        world,
        NAME | LOCAL_TRANSFORM
            | GLOBAL_TRANSFORM
            | LOCAL_TRANSFORM_DIRTY
            | INSTANCED_MESH
            | MATERIAL_REF
            | VISIBILITY
            | CASTS_SHADOW,
        1,
    )[0];

    world
        .core
        .set_name(entity, components::Name(format!("Instanced {}", mesh_name)));
    world.core.set_local_transform(
        entity,
        components::LocalTransform {
            translation: nalgebra_glm::Vec3::new(0.0, 0.0, 0.0),
            scale: nalgebra_glm::Vec3::new(1.0, 1.0, 1.0),
            rotation: nalgebra_glm::Quat::identity(),
        },
    );
    world
        .core
        .set_global_transform(entity, components::GlobalTransform::default());
    world
        .core
        .set_local_transform_dirty(entity, components::LocalTransformDirty);

    world.core.set_instanced_mesh(
        entity,
        crate::ecs::mesh::components::InstancedMesh::with_instances(mesh_name, instances),
    );

    if let Some(&index) = world
        .resources
        .assets
        .material_registry
        .registry
        .name_to_index
        .get(material_name)
    {
        registry_add_reference(
            &mut world.resources.assets.material_registry.registry,
            index,
        );
    }

    let texture_names: Vec<String> = registry_entry_by_name(
        &world.resources.assets.material_registry.registry,
        material_name,
    )
    .map(|mat| mat.texture_names().map(str::to_string).collect())
    .unwrap_or_default();
    for texture in &texture_names {
        texture_cache_add_reference(&mut world.resources.texture_cache, texture);
    }

    world
        .core
        .set_material_ref(entity, components::MaterialRef::new(material_name));

    world
        .core
        .set_visibility(entity, components::Visibility { visible: true });
    world.core.set_casts_shadow(entity, components::CastsShadow);

    entity
}

pub fn find_entity_by_name(world: &World, name: &str) -> Option<Entity> {
    world
        .core
        .query_entities(NAME)
        .find(|&entity| world.core.get_name(entity).is_some_and(|n| n.0 == name))
}

/// Register `material` in the registry under `name` and attach it to `entity` via [`MaterialRef`].
pub fn spawn_material(
    world: &mut World,
    entity: Entity,
    name: String,
    material: crate::ecs::material::components::Material,
) {
    material_registry_insert(
        &mut world.resources.assets.material_registry,
        name.clone(),
        material,
    );
    if let Some(&index) = world
        .resources
        .assets
        .material_registry
        .registry
        .name_to_index
        .get(&name)
    {
        registry_add_reference(
            &mut world.resources.assets.material_registry.registry,
            index,
        );
    }
    world.core.set_material_ref(entity, MaterialRef::new(name));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn process_commands_system_drains_ecs_and_leaves_render() {
        let mut world = World::default();
        let entity = world.spawn_count(1)[0];

        queue_ecs_command(&mut world, EcsCommand::DespawnRecursive { entity });
        queue_render_command(
            &mut world,
            RenderCommand::LoadHdrSkybox {
                hdr_data: vec![0u8; 4],
            },
        );

        process_commands_system(&mut world);

        assert!(
            world.resources.commands.ecs.is_empty(),
            "process_commands_system should drain the ecs queue"
        );
        assert_eq!(
            world.resources.commands.render.len(),
            1,
            "process_commands_system must not touch the render queue"
        );
        assert!(matches!(
            world.resources.commands.render[0],
            RenderCommand::LoadHdrSkybox { .. }
        ));
    }
}
