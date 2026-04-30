//! Deferred command system for GPU-dependent operations.
//!
//! Some operations (texture uploads, skybox loading, screenshots) require GPU access
//! and cannot execute immediately. Queue them with [`WorldCommand`] for processing
//! during the render phase.
//!
//! # Queuing Commands
//!
//! ```ignore
//! // Queue a command for deferred execution
//! world.queue_command(WorldCommand::LoadTexture {
//!     name: "my_texture".to_string(),
//!     rgba_data: image_bytes,
//!     width: 512,
//!     height: 512,
//! });
//! ```
//!
//! # Available Commands
//!
//! | Command | Description |
//! |---------|-------------|
//! | `DespawnRecursive` | Delete entity and all children |
//! | `UploadSpriteTexture` | Upload texture to sprite atlas slot |
//! | `LoadTexture` | Load texture into cache by name |
//! | `LoadHdrSkybox` | Load HDR skybox from bytes |
//! | `LoadHdrSkyboxFromPath` | Load HDR skybox from file path |
//! | `CaptureProceduralAtmosphereIBL` | Generate IBL from procedural sky |
//! | `CaptureScreenshot` | Save screenshot to file |
//!
//! # Example: Dynamic Texture Loading
//!
//! ```ignore
//! fn load_custom_texture(&mut self, world: &mut World, image_data: &[u8]) {
//!     // Decode image (using image crate)
//!     let img = image::load_from_memory(image_data).unwrap().to_rgba8();
//!     let (width, height) = img.dimensions();
//!
//!     world.queue_command(WorldCommand::LoadTexture {
//!         name: "custom_texture".to_string(),
//!         rgba_data: img.into_raw(),
//!         width,
//!         height,
//!     });
//!
//!     // Texture will be available in texture cache next frame
//! }
//! ```
//!
//! # Example: Screenshot
//!
//! ```ignore
//! fn capture_screenshot(&mut self, world: &mut World) {
//!     world.queue_command(WorldCommand::CaptureScreenshot {
//!         path: Some("screenshot.png".into()),
//!     });
//! }
//! ```

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

pub type CommandQueue = Vec<WorldCommand>;

/// Deferred commands for operations requiring GPU access.
///
/// Queue commands via `world.queue_command(cmd)`. They execute during the render phase.
#[derive(Debug, Clone)]
pub enum WorldCommand {
    DespawnRecursive {
        entity: Entity,
    },
    UploadSpriteTexture {
        slot: u32,
        rgba_data: Vec<u8>,
        width: u32,
        height: u32,
    },
    LoadTexture {
        name: String,
        rgba_data: Vec<u8>,
        width: u32,
        height: u32,
        usage: crate::render::wgpu::texture_cache::TextureUsage,
        sampler: crate::render::wgpu::texture_cache::SamplerSettings,
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
    },
    ReloadTexture {
        name: String,
        rgba_data: Vec<u8>,
        width: u32,
        height: u32,
    },
    ReloadMaterial {
        name: String,
        material: Box<crate::ecs::material::components::Material>,
    },
}

impl World {
    pub fn spawn_entities(&mut self, core_mask: u64, count: usize) -> Vec<Entity> {
        let entities = self.spawn_count(count);
        for &entity in &entities {
            self.core.add_components(entity, core_mask);
        }
        entities
    }

    pub fn spawn_entities_ui(&mut self, ui_mask: u64, count: usize) -> Vec<Entity> {
        let entities = self.spawn_count(count);
        for &entity in &entities {
            self.ui.add_components(entity, ui_mask);
        }
        entities
    }

    pub fn queue_command(&mut self, command: WorldCommand) {
        self.resources.command_queue.push(command);
    }

    pub fn set_cursor_locked(&mut self, locked: bool) {
        if let Some(window_handle) = &self.resources.window.handle {
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

    pub fn set_cursor_visible(&mut self, visible: bool) {
        if let Some(window_handle) = &self.resources.window.handle {
            window_handle.set_cursor_visible(visible);
        }
    }

    pub fn register_material(
        &mut self,
        entity: Entity,
        name: String,
        material: crate::ecs::material::components::Material,
    ) {
        material_registry_insert(
            &mut self.resources.material_registry,
            name.clone(),
            material,
        );
        if let Some(&index) = self
            .resources
            .material_registry
            .registry
            .name_to_index
            .get(&name)
        {
            self.resources
                .material_registry
                .registry
                .add_reference(index);
        }
        self.core.set_material_ref(entity, MaterialRef::new(name));
        self.resources.mesh_render_state.mark_entity_added(entity);
    }

    #[cfg(feature = "physics")]
    pub fn spawn_physics_body(&mut self, entity: Entity) {
        let rigid_body_comp = self.core.get_rigid_body(entity).cloned();
        let collider_comp = self.core.get_collider(entity).cloned();

        if let Some(rigid_body_comp) = rigid_body_comp {
            let rapier_body = rigid_body_comp.to_rapier_rigid_body();
            let rapier_handle = self.resources.physics.add_rigid_body(rapier_body);

            if let Some(collider_comp) = collider_comp {
                let rapier_collider = collider_comp.to_rapier_collider();
                self.resources
                    .physics
                    .add_collider(rapier_collider, rapier_handle);
            }

            if let Some(rigid_body_mut) = self.core.get_rigid_body_mut(entity) {
                rigid_body_mut.handle = Some(rapier_handle.into());
            }

            self.resources
                .physics
                .handle_to_entity
                .insert(rapier_handle, entity);
        }
    }
}

pub fn process_commands_system(world: &mut World) {
    let commands = world.resources.command_queue.drain(..).collect::<Vec<_>>();
    let _span = tracing::info_span!("commands", count = commands.len()).entered();
    let mut sprite_texture_commands = Vec::new();

    for command in commands {
        match &command {
            WorldCommand::UploadSpriteTexture { .. }
            | WorldCommand::LoadTexture { .. }
            | WorldCommand::LoadHdrSkybox { .. }
            | WorldCommand::LoadHdrSkyboxFromPath { .. }
            | WorldCommand::CaptureProceduralAtmosphereIBL { .. }
            | WorldCommand::CaptureIblSnapshots { .. }
            | WorldCommand::CaptureScreenshot { .. }
            | WorldCommand::ReloadTexture { .. } => {
                sprite_texture_commands.push(command);
            }
            _ => {
                process_command(world, command);
            }
        }
    }

    world
        .resources
        .command_queue
        .extend(sprite_texture_commands);
}

fn process_command(world: &mut World, command: WorldCommand) {
    match command {
        WorldCommand::DespawnRecursive { entity } => {
            if !world.resources.children_cache_valid {
                world.validate_and_rebuild_children_cache();
            }
            let descendents = query_descendents(world, entity);
            despawn_entities_with_cache_cleanup(world, &descendents);
        }
        WorldCommand::UploadSpriteTexture { .. } => {}
        WorldCommand::LoadTexture { .. } => {}
        WorldCommand::LoadHdrSkybox { .. } => {}
        WorldCommand::LoadHdrSkyboxFromPath { .. } => {}
        WorldCommand::CaptureProceduralAtmosphereIBL { .. } => {}
        WorldCommand::CaptureIblSnapshots { .. } => {}
        WorldCommand::CaptureScreenshot { .. } => {}
        WorldCommand::ReloadTexture { .. } => {}
        WorldCommand::ReloadMaterial { name, material } => {
            reload_material_command(world, name, *material);
        }
    }
}

fn reload_material_command(
    world: &mut World,
    name: String,
    new_material: crate::ecs::material::components::Material,
) {
    use crate::ecs::world::MATERIAL_REF;

    let old_textures = registry_entry_by_name(&world.resources.material_registry.registry, &name)
        .map(|mat| (mat.base_texture.clone(), mat.emissive_texture.clone()));

    if let Some((old_base, old_emissive)) = &old_textures {
        if let Some(base_texture) = old_base {
            texture_cache_remove_reference(&mut world.resources.texture_cache, base_texture);
        }
        if let Some(emissive_texture) = old_emissive {
            texture_cache_remove_reference(&mut world.resources.texture_cache, emissive_texture);
        }
    }

    if let Some(base_texture) = &new_material.base_texture {
        texture_cache_add_reference(&mut world.resources.texture_cache, base_texture);
    }
    if let Some(emissive_texture) = &new_material.emissive_texture {
        texture_cache_add_reference(&mut world.resources.texture_cache, emissive_texture);
    }

    if let Some(existing) =
        registry_entry_by_name_mut(&mut world.resources.material_registry.registry, &name)
    {
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

pub fn despawn_entities_with_cache_cleanup(world: &mut World, entities: &[Entity]) {
    let mut has_parent_entities = false;
    let is_runtime = world.resources.is_runtime;

    for &entity in entities {
        world.resources.children_cache.remove(&entity);
        world.resources.entity_tags.remove(&entity);
        world.resources.entity_metadata.remove(&entity);

        if let Some(parent_component) = world.core.get_parent(entity)
            && let Some(parent_entity) = parent_component.0
            && let Some(children) = world.resources.children_cache.get_mut(&parent_entity)
        {
            children.retain(|&e| e != entity);
            has_parent_entities = true;
        }

        if let Some(render_mesh) = world.core.get_render_mesh(entity) {
            if !is_runtime {
                let mesh_name = render_mesh.name.clone();
                if let Some(&index) = world
                    .resources
                    .mesh_cache
                    .registry
                    .name_to_index
                    .get(&mesh_name)
                {
                    world.resources.mesh_cache.registry.remove_reference(index);
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
                    .mesh_cache
                    .registry
                    .name_to_index
                    .get(&mesh_name)
                {
                    world.resources.mesh_cache.registry.remove_reference(index);
                }
            }

            if let Some(material_ref) = world.core.get_material_ref(entity) {
                let material_name = material_ref.name.clone();
                if let Some(material) = registry_entry_by_name(
                    &world.resources.material_registry.registry,
                    &material_name,
                ) {
                    if let Some(base_texture) = &material.base_texture {
                        texture_cache_remove_reference(
                            &mut world.resources.texture_cache,
                            base_texture,
                        );
                    }
                    if let Some(emissive_texture) = &material.emissive_texture {
                        texture_cache_remove_reference(
                            &mut world.resources.texture_cache,
                            emissive_texture,
                        );
                    }
                }
                if let Some(&index) = world
                    .resources
                    .material_registry
                    .registry
                    .name_to_index
                    .get(&material_name)
                {
                    world
                        .resources
                        .material_registry
                        .registry
                        .remove_reference(index);
                }
            }
        }
    }

    if has_parent_entities {
        world.resources.children_cache_valid = false;
    }

    world.despawn_entities(entities);
}

pub fn despawn_recursive_immediate(world: &mut World, entity: Entity) {
    if !world.resources.children_cache_valid {
        world.validate_and_rebuild_children_cache();
    }
    let descendents = query_descendents(world, entity);
    despawn_entities_with_cache_cleanup(world, &descendents);
}

pub fn spawn_sun(world: &mut World) -> Entity {
    use crate::ecs::world::components;
    use crate::ecs::world::{
        GLOBAL_TRANSFORM, LIGHT, LOCAL_TRANSFORM, LOCAL_TRANSFORM_DIRTY, NAME,
    };

    let entity = world.spawn_entities(
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
        },
    );

    entity
}

pub fn spawn_sun_without_shadows(world: &mut World) -> Entity {
    use crate::ecs::world::components;
    use crate::ecs::world::{
        GLOBAL_TRANSFORM, LIGHT, LOCAL_TRANSFORM, LOCAL_TRANSFORM_DIRTY, NAME,
    };

    let entity = world.spawn_entities(
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
        },
    );

    entity
}

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

    let entity = world.spawn_entities(
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
            mesh_cache_insert(&mut world.resources.mesh_cache, mesh_name.to_string(), mesh);
        }
    }

    if let Some(&index) = world
        .resources
        .mesh_cache
        .registry
        .name_to_index
        .get(&render_mesh.name)
    {
        world.resources.mesh_cache.registry.add_reference(index);
    }

    world.core.set_render_mesh(entity, render_mesh);
    world.resources.mesh_render_state.mark_entity_added(entity);

    if let Some(&index) = world
        .resources
        .material_registry
        .registry
        .name_to_index
        .get("Default")
    {
        world
            .resources
            .material_registry
            .registry
            .add_reference(index);
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

pub fn spawn_cube_at(world: &mut World, position: nalgebra_glm::Vec3) -> Entity {
    spawn_mesh_at(
        world,
        "Cube",
        position,
        nalgebra_glm::Vec3::new(1.0, 1.0, 1.0),
    )
}

pub fn spawn_sphere_at(world: &mut World, position: nalgebra_glm::Vec3) -> Entity {
    spawn_mesh_at(
        world,
        "Sphere",
        position,
        nalgebra_glm::Vec3::new(1.0, 1.0, 1.0),
    )
}

pub fn spawn_cylinder_at(world: &mut World, position: nalgebra_glm::Vec3) -> Entity {
    spawn_mesh_at(
        world,
        "Cylinder",
        position,
        nalgebra_glm::Vec3::new(1.0, 1.0, 1.0),
    )
}

pub fn spawn_torus_at(world: &mut World, position: nalgebra_glm::Vec3) -> Entity {
    spawn_mesh_at(
        world,
        "Torus",
        position,
        nalgebra_glm::Vec3::new(1.0, 1.0, 1.0),
    )
}

pub fn spawn_cone_at(world: &mut World, position: nalgebra_glm::Vec3) -> Entity {
    spawn_mesh_at(
        world,
        "Cone",
        position,
        nalgebra_glm::Vec3::new(1.0, 1.0, 1.0),
    )
}

pub fn spawn_plane_at(world: &mut World, position: nalgebra_glm::Vec3) -> Entity {
    spawn_mesh_at(
        world,
        "Plane",
        position,
        nalgebra_glm::Vec3::new(1.0, 1.0, 1.0),
    )
}

pub fn spawn_water_plane_at(world: &mut World, position: nalgebra_glm::Vec3) -> Entity {
    use crate::ecs::mesh::components::create_subdivided_plane_mesh;
    use crate::ecs::water::Water;
    use crate::ecs::world::components;
    use crate::ecs::world::{
        GLOBAL_TRANSFORM, LOCAL_TRANSFORM, LOCAL_TRANSFORM_DIRTY, NAME, RENDER_MESH, VISIBILITY,
        WATER,
    };

    let entity = world.spawn_entities(
        NAME | LOCAL_TRANSFORM
            | GLOBAL_TRANSFORM
            | LOCAL_TRANSFORM_DIRTY
            | RENDER_MESH
            | VISIBILITY
            | WATER,
        1,
    )[0];

    world
        .core
        .set_name(entity, components::Name("Water Plane".to_string()));
    world.core.set_local_transform(
        entity,
        components::LocalTransform {
            translation: position,
            scale: nalgebra_glm::Vec3::new(10.0, 1.0, 10.0),
            rotation: nalgebra_glm::Quat::identity(),
        },
    );
    world
        .core
        .set_global_transform(entity, components::GlobalTransform::default());
    world
        .core
        .set_local_transform_dirty(entity, components::LocalTransformDirty);

    let mesh_name = "WaterPlane";
    if !world
        .resources
        .mesh_cache
        .registry
        .name_to_index
        .contains_key(mesh_name)
    {
        let mesh = create_subdivided_plane_mesh(20.0, 1);
        mesh_cache_insert(&mut world.resources.mesh_cache, mesh_name.to_string(), mesh);
    }

    if let Some(&index) = world
        .resources
        .mesh_cache
        .registry
        .name_to_index
        .get(mesh_name)
    {
        world.resources.mesh_cache.registry.add_reference(index);
    }

    world
        .core
        .set_render_mesh(entity, components::RenderMesh::new(mesh_name));
    world.resources.mesh_render_state.mark_entity_added(entity);

    world.core.set_water(entity, Water::default());

    if let Some(visibility) = world.core.get_visibility_mut(entity) {
        visibility.visible = true;
    }

    entity
}

pub fn spawn_3d_text_with_properties(
    world: &mut World,
    text: &str,
    position: nalgebra_glm::Vec3,
    properties: crate::ecs::text::components::TextProperties,
) -> Entity {
    spawn_3d_text_impl(world, text, position, properties, false)
}

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

    let text_index = world.resources.text_cache.add_text(text);

    let entity = world.spawn_entities(
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
            font_index: 0,
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
    world.resources.cleanup_frame_counter += 1;

    if world.resources.cleanup_frame_counter >= CLEANUP_INTERVAL_FRAMES {
        world.resources.cleanup_frame_counter = 0;
        world.resources.mesh_cache.registry.remove_unused();
        material_registry_remove_unused(&mut world.resources.material_registry);
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
        let old_textures =
            registry_entry_by_name(&world.resources.material_registry.registry, &mat_ref.name)
                .map(|mat| (mat.base_texture.clone(), mat.emissive_texture.clone()));

        if let Some((old_base, old_emissive)) = old_textures {
            if let Some(base_texture) = old_base {
                texture_cache_remove_reference(&mut world.resources.texture_cache, &base_texture);
            }
            if let Some(emissive_texture) = old_emissive {
                texture_cache_remove_reference(
                    &mut world.resources.texture_cache,
                    &emissive_texture,
                );
            }
        }
    }

    if let Some(base_texture) = &material.base_texture {
        texture_cache_add_reference(&mut world.resources.texture_cache, base_texture);
    }
    if let Some(emissive_texture) = &material.emissive_texture {
        texture_cache_add_reference(&mut world.resources.texture_cache, emissive_texture);
    }

    if let Some(ref mat_ref) = material_ref {
        if let Some(existing_mat) = registry_entry_by_name_mut(
            &mut world.resources.material_registry.registry,
            &mat_ref.name,
        ) {
            *existing_mat = material;
        }
    } else {
        let material_name = format!("TexturedMaterial_{}", entity.id);
        material_registry_insert(
            &mut world.resources.material_registry,
            material_name.clone(),
            material,
        );
        if let Some(&index) = world
            .resources
            .material_registry
            .registry
            .name_to_index
            .get(&material_name)
        {
            world
                .resources
                .material_registry
                .registry
                .add_reference(index);
        }
        world.core.set_material_ref(
            entity,
            crate::ecs::material::components::MaterialRef::new(material_name),
        );
    }
}

pub fn load_procedural_textures(world: &mut World) {
    let default_sampler = crate::render::wgpu::texture_cache::SamplerSettings::DEFAULT;

    let checkerboard = generate_checkerboard_texture();
    world.queue_command(WorldCommand::LoadTexture {
        name: "checkerboard".to_string(),
        rgba_data: checkerboard.0,
        width: checkerboard.1,
        height: checkerboard.2,
        usage: crate::render::wgpu::texture_cache::TextureUsage::Color,
        sampler: default_sampler,
    });

    let gradient = generate_gradient_texture();
    world.queue_command(WorldCommand::LoadTexture {
        name: "gradient".to_string(),
        rgba_data: gradient.0,
        width: gradient.1,
        height: gradient.2,
        usage: crate::render::wgpu::texture_cache::TextureUsage::Color,
        sampler: default_sampler,
    });

    let uv_test = generate_uv_test_texture();
    world.queue_command(WorldCommand::LoadTexture {
        name: "uv_test".to_string(),
        rgba_data: uv_test.0,
        width: uv_test.1,
        height: uv_test.2,
        usage: crate::render::wgpu::texture_cache::TextureUsage::Color,
        sampler: default_sampler,
    });
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
    world.queue_command(WorldCommand::LoadHdrSkybox { hdr_data });
}

pub fn load_hdr_skybox_from_path(world: &mut World, path: std::path::PathBuf) {
    world.queue_command(WorldCommand::LoadHdrSkyboxFromPath { path });
}

pub fn capture_procedural_atmosphere_ibl(world: &mut World, atmosphere: Atmosphere, time: f32) {
    world.queue_command(WorldCommand::CaptureProceduralAtmosphereIBL { atmosphere, time });
}

pub fn capture_ibl_snapshots(world: &mut World, atmosphere: Atmosphere, hours: Vec<f32>) {
    world.queue_command(WorldCommand::CaptureIblSnapshots { atmosphere, hours });
}

#[cfg(not(target_arch = "wasm32"))]
pub fn capture_screenshot(world: &mut World) {
    world.queue_command(WorldCommand::CaptureScreenshot { path: None });
}

#[cfg(not(target_arch = "wasm32"))]
pub fn capture_screenshot_to_path(world: &mut World, path: impl Into<std::path::PathBuf>) {
    world.queue_command(WorldCommand::CaptureScreenshot {
        path: Some(path.into()),
    });
}

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

    let entity = world.spawn_entities(
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
        .material_registry
        .registry
        .name_to_index
        .get("Default")
    {
        world
            .resources
            .material_registry
            .registry
            .add_reference(index);
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

    let entity = world.spawn_entities(
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
        .material_registry
        .registry
        .name_to_index
        .get(material_name)
    {
        world
            .resources
            .material_registry
            .registry
            .add_reference(index);
    }

    if let Some(mat) =
        registry_entry_by_name(&world.resources.material_registry.registry, material_name)
    {
        if let Some(base_texture) = &mat.base_texture {
            texture_cache_add_reference(&mut world.resources.texture_cache, base_texture);
        }
        if let Some(emissive_texture) = &mat.emissive_texture {
            texture_cache_add_reference(&mut world.resources.texture_cache, emissive_texture);
        }
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

pub fn spawn_material(
    world: &mut World,
    entity: Entity,
    name: String,
    material: crate::ecs::material::components::Material,
) {
    material_registry_insert(
        &mut world.resources.material_registry,
        name.clone(),
        material,
    );
    if let Some(&index) = world
        .resources
        .material_registry
        .registry
        .name_to_index
        .get(&name)
    {
        world
            .resources
            .material_registry
            .registry
            .add_reference(index);
    }
    world.core.set_material_ref(entity, MaterialRef::new(name));
}
