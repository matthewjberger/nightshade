use crate::ecs::material::components::MaterialRef;
use crate::ecs::material::resources::{material_registry_insert, material_registry_remove_unused};
#[cfg(feature = "physics")]
use crate::ecs::physics::resources::{physics_world_add_collider, physics_world_add_rigid_body};
use crate::ecs::world::{Entity, NAME, World};
use crate::render::generational_registry::registry_entry_by_name;
use crate::render::mesh_cache::{mesh_cache_insert, mesh_cache_remove_unused};
use crate::render::procedural_meshes::{
    create_cone_mesh, create_cube_mesh, create_cylinder_mesh, create_plane_mesh,
    create_sphere_mesh, create_subdivided_plane_mesh, create_torus_mesh,
};
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
        material: Box<crate::render::material::Material>,
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
    /// Upload a 3D color grading lookup table. `data` is RGBA8 for a
    /// 16x16x16 table with red varying fastest then green then blue
    /// (4096 texels, 16384 bytes). Set `color_lut_weight` on the color
    /// grading settings to blend it in.
    SetColorLut {
        data: Vec<u8>,
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
        world.ecs.worlds[CORE].add_components(entity, core_mask);
    }
    entities
}

/// Spawn `count` entities in the retained-UI archetype with the supplied component mask.
pub fn spawn_entities_ui(world: &mut World, ui_mask: u64, count: usize) -> Vec<Entity> {
    let entities = world.spawn_count(count);
    for &entity in &entities {
        world.ecs.worlds[UI].add_components(entity, ui_mask);
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
    #[cfg(not(target_arch = "wasm32"))]
    {
        world.resources.window.cursor_locked = locked;
    }
}

/// Reconcile [`Window::cursor_locked`] with the browser's actual pointer-lock
/// state. On the web the browser, not the app, owns the lock: pressing Escape
/// exits it and the keydown is never delivered, so [`set_cursor_locked`] cannot
/// be the source of truth. Run this once per frame before app systems so a
/// user-initiated release is visible to game logic.
#[cfg(target_arch = "wasm32")]
pub fn sync_cursor_lock_state(world: &mut World) {
    let locked = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.pointer_lock_element())
        .is_some();
    world.resources.window.cursor_locked = locked;
}

/// Show or hide the OS cursor.
pub fn set_cursor_visible(world: &mut World, visible: bool) {
    if let Some(window_handle) = &world.resources.window.handle {
        window_handle.set_cursor_visible(visible);
    }
}

/// Set the global time scale that the frame's delta time is multiplied by:
/// `0.5` for slow motion, `2.0` for fast forward, `1.0` for real time. Negative
/// values clamp to zero. Systems that read the scaled delta time follow it;
/// unscaled work reads `raw_delta_time` instead.
pub fn set_time_scale(world: &mut World, scale: f32) {
    world.resources.window.timing.time_speed = scale.max(0.0);
}

/// The current global time scale.
pub fn time_scale(world: &World) -> f32 {
    world.resources.window.timing.time_speed
}

/// Pause game time, so the scaled delta time reports zero until resumed. The
/// time scale is preserved, so resuming returns to the previous speed.
pub fn pause(world: &mut World) {
    world.resources.window.timing.paused = true;
}

/// Resume game time after a pause.
pub fn unpause(world: &mut World) {
    world.resources.window.timing.paused = false;
}

/// Set whether game time is paused.
pub fn set_paused(world: &mut World, paused: bool) {
    world.resources.window.timing.paused = paused;
}

/// Whether game time is currently paused.
pub fn is_paused(world: &World) -> bool {
    world.resources.window.timing.paused
}

/// Insert a material into the registry, ref-count it, and attach it to `entity` via [`MaterialRef`].
pub fn register_material(
    world: &mut World,
    entity: Entity,
    name: String,
    material: crate::render::material::Material,
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
    world.ecs.worlds[CORE].set(entity, MaterialRef::new(name));
    world.resources.mesh_render_state.mark_entity_added(entity);
}

#[cfg(feature = "physics")]
pub fn spawn_physics_body(world: &mut World, entity: Entity) {
    let rigid_body_comp = world.ecs.worlds[CORE]
        .get::<crate::ecs::physics::components::RigidBodyComponent>(entity)
        .cloned();
    let collider_comp = world.ecs.worlds[CORE]
        .get::<crate::ecs::physics::components::ColliderComponent>(entity)
        .cloned();

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

        if let Some(rigid_body_mut) =
            world.ecs.worlds[CORE]
                .get_mut::<crate::ecs::physics::components::RigidBodyComponent>(entity)
        {
            rigid_body_mut.handle = Some(
                crate::ecs::physics::types::rigid_body_handle_from_rapier(rapier_handle),
            );
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
    new_material: crate::render::material::Material,
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

    crate::ecs::material::resources::material_registry_mutate(
        &mut world.resources.assets.material_registry,
        &name,
        |existing| *existing = new_material,
    );

    let entities_to_dirty: Vec<Entity> = world.ecs.worlds[CORE]
        .query_entities(MATERIAL_REF)
        .filter(|&entity| {
            world.ecs.worlds[CORE]
                .get::<crate::ecs::material::components::MaterialRef>(entity)
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
        if let Some(guid) = world.ecs.worlds[CORE]
            .get::<crate::ecs::primitives::Guid>(entity)
            .copied()
        {
            world.resources.entities.guid_index.remove(&guid.0);
        }

        if let Some(parent_component) =
            world.ecs.worlds[CORE].get::<crate::ecs::transform::components::Parent>(entity)
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

        if let Some(render_mesh) =
            world.ecs.worlds[CORE].get::<crate::ecs::mesh::components::RenderMesh>(entity)
        {
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

        if let Some(instanced_mesh) =
            world.ecs.worlds[CORE].get::<crate::ecs::mesh::components::InstancedMesh>(entity)
        {
            if !is_runtime {
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
            world
                .resources
                .mesh_render_state
                .mark_instanced_meshes_changed();
        }

        if !is_runtime
            && let Some(material_ref) =
                world.ecs.worlds[CORE].get::<crate::ecs::material::components::MaterialRef>(entity)
        {
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
/// Sets an entity's local transform, a default global transform, and the
/// dirty flag. These three writes are performed together by every spawn helper.
pub(crate) fn setup_entity_transforms(
    world: &mut World,
    entity: Entity,
    local_transform: crate::ecs::world::components::LocalTransform,
) {
    world.ecs.worlds[CORE].set(entity, local_transform);
    world.ecs.worlds[CORE].set(
        entity,
        crate::ecs::world::components::GlobalTransform::default(),
    );
}

pub fn spawn_sun(world: &mut World) -> Entity {
    use crate::ecs::world::components;
    use crate::ecs::world::{GLOBAL_TRANSFORM, LIGHT, LOCAL_TRANSFORM, NAME};

    let entity = spawn_entities(world, NAME | LOCAL_TRANSFORM | GLOBAL_TRANSFORM | LIGHT, 1)[0];

    world.ecs.worlds[CORE].set(entity, components::Name("Sun".to_string()));
    setup_entity_transforms(
        world,
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
    world.ecs.worlds[CORE].set(
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
            ..Default::default()
        },
    );

    entity
}

/// Spawn a directional light positioned above the origin without shadow casting.
pub fn spawn_sun_without_shadows(world: &mut World) -> Entity {
    use crate::ecs::world::components;
    use crate::ecs::world::{GLOBAL_TRANSFORM, LIGHT, LOCAL_TRANSFORM, NAME};

    let entity = spawn_entities(world, NAME | LOCAL_TRANSFORM | GLOBAL_TRANSFORM | LIGHT, 1)[0];

    world.ecs.worlds[CORE].set(entity, components::Name("Sun".to_string()));
    setup_entity_transforms(
        world,
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
    world.ecs.worlds[CORE].set(
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
            ..Default::default()
        },
    );

    entity
}

/// Spawn a positioned light entity carrying a default [`components::Light`] for the caller to configure.
pub fn spawn_light_entity(world: &mut World, position: nalgebra_glm::Vec3, name: &str) -> Entity {
    use crate::ecs::world::components;
    use crate::ecs::world::{GLOBAL_TRANSFORM, LIGHT, LOCAL_TRANSFORM, NAME};

    let entity = spawn_entities(world, NAME | LOCAL_TRANSFORM | GLOBAL_TRANSFORM | LIGHT, 1)[0];
    world.ecs.worlds[CORE].set(entity, components::Name(name.to_string()));
    setup_entity_transforms(
        world,
        entity,
        components::LocalTransform {
            translation: position,
            rotation: nalgebra_glm::quat_identity(),
            scale: nalgebra_glm::Vec3::new(1.0, 1.0, 1.0),
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
        BOUNDING_VOLUME, CASTS_SHADOW, GLOBAL_TRANSFORM, LOCAL_TRANSFORM, MATERIAL_REF, NAME,
        RENDER_MESH, VISIBILITY,
    };

    let entity = spawn_entities(
        world,
        NAME | LOCAL_TRANSFORM
            | GLOBAL_TRANSFORM
            | RENDER_MESH
            | MATERIAL_REF
            | BOUNDING_VOLUME
            | CASTS_SHADOW
            | VISIBILITY,
        1,
    )[0];

    world.ecs.worlds[CORE].set(entity, components::Name(mesh_name.to_string()));
    setup_entity_transforms(
        world,
        entity,
        components::LocalTransform {
            translation: position,
            scale,
            rotation: nalgebra_glm::Quat::identity(),
        },
    );
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

    world.ecs.worlds[CORE].set(entity, render_mesh);
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
    world.ecs.worlds[CORE].set(entity, components::MaterialRef::new("Default"));

    world.ecs.worlds[CORE].set(
        entity,
        components::BoundingVolume::from_mesh_type(mesh_name),
    );
    world.ecs.worlds[CORE].set(entity, components::CastsShadow);

    if let Some(visibility) =
        world.ecs.worlds[CORE].get_mut::<crate::ecs::primitives::Visibility>(entity)
    {
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
    use crate::ecs::world::{GLOBAL_TRANSFORM, LOCAL_TRANSFORM, NAME, TEXT, VISIBILITY};

    let text_index = world.resources.text.cache.add_text(text);

    let entity = spawn_entities(
        world,
        NAME | LOCAL_TRANSFORM | GLOBAL_TRANSFORM | TEXT | VISIBILITY,
        1,
    )[0];

    world.ecs.worlds[CORE].set(entity, components::Name(text.to_string()));
    world.ecs.worlds[CORE].set(
        entity,
        components::LocalTransform {
            translation: position,
            rotation: nalgebra_glm::Quat::identity(),
            scale: nalgebra_glm::Vec3::new(1.0, 1.0, 1.0),
        },
    );
    world.ecs.worlds[CORE].set(entity, components::GlobalTransform::default());
    world.ecs.worlds[CORE].set(
        entity,
        components::Text {
            text_index,
            properties,
            dirty: true,
            cached_mesh: None,
            billboard,
        },
    );
    world.ecs.worlds[CORE].set(entity, components::Visibility { visible: true });

    entity
}

const CLEANUP_INTERVAL_FRAMES: u64 = 300;

pub fn cleanup_unused_resources_system(world: &mut World) {
    let _span = tracing::info_span!("cleanup").entered();
    world.resources.cleanup.frame_counter += 1;

    if world.resources.cleanup.frame_counter >= CLEANUP_INTERVAL_FRAMES {
        world.resources.cleanup.frame_counter = 0;
        mesh_cache_remove_unused(&mut world.resources.assets.mesh_cache);
        material_registry_remove_unused(&mut world.resources.assets.material_registry);
        texture_cache_remove_unused(&mut world.resources.texture_cache);
    }
}

pub fn set_material_with_textures(
    world: &mut World,
    entity: Entity,
    material: crate::render::material::Material,
) {
    let material_ref = world.ecs.worlds[CORE]
        .get::<crate::ecs::material::components::MaterialRef>(entity)
        .cloned();

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
        crate::ecs::material::resources::material_registry_mutate(
            &mut world.resources.assets.material_registry,
            &mat_ref.name,
            |existing_mat| *existing_mat = material,
        );
        world
            .resources
            .mesh_render_state
            .mark_material_dirty(entity);
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
        world.ecs.worlds[CORE].set(
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

    for name in ["checkerboard", "gradient", "uv_test"] {
        crate::render::wgpu::texture_cache::texture_cache_protect(
            &mut world.resources.texture_cache,
            name.to_string(),
        );
    }
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

    world.ecs.worlds[CORE].set(entity, components::Name(format!("Instanced {}", mesh_name)));
    world.ecs.worlds[CORE].set(
        entity,
        components::LocalTransform {
            translation: nalgebra_glm::Vec3::new(0.0, 0.0, 0.0),
            scale: nalgebra_glm::Vec3::new(1.0, 1.0, 1.0),
            rotation: nalgebra_glm::Quat::identity(),
        },
    );
    world.ecs.worlds[CORE].set(entity, components::GlobalTransform::default());

    world.ecs.worlds[CORE].set(
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
    world.ecs.worlds[CORE].set(entity, components::MaterialRef::new("Default"));

    world.ecs.worlds[CORE].set(entity, components::Visibility { visible: true });
    world.ecs.worlds[CORE].set(entity, components::CastsShadow);

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

    world.ecs.worlds[CORE].set(entity, components::Name(format!("Instanced {}", mesh_name)));
    world.ecs.worlds[CORE].set(
        entity,
        components::LocalTransform {
            translation: nalgebra_glm::Vec3::new(0.0, 0.0, 0.0),
            scale: nalgebra_glm::Vec3::new(1.0, 1.0, 1.0),
            rotation: nalgebra_glm::Quat::identity(),
        },
    );
    world.ecs.worlds[CORE].set(entity, components::GlobalTransform::default());

    world.ecs.worlds[CORE].set(
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

    world.ecs.worlds[CORE].set(entity, components::MaterialRef::new(material_name));

    world.ecs.worlds[CORE].set(entity, components::Visibility { visible: true });
    world.ecs.worlds[CORE].set(entity, components::CastsShadow);

    entity
}

pub fn find_entity_by_name(world: &World, name: &str) -> Option<Entity> {
    world.ecs.worlds[CORE].query_entities(NAME).find(|&entity| {
        world.ecs.worlds[CORE]
            .get::<crate::ecs::primitives::Name>(entity)
            .is_some_and(|n| n.0 == name)
    })
}

/// Register `material` in the registry under `name` and attach it to `entity` via [`MaterialRef`].
pub fn spawn_material(
    world: &mut World,
    entity: Entity,
    name: String,
    material: crate::render::material::Material,
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
    world.ecs.worlds[CORE].set(entity, MaterialRef::new(name));
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
