use crate::ecs::world::{Entity, World};

pub type CommandQueue = Vec<WorldCommand>;

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
    },
    LoadHdrSkybox {
        hdr_data: Vec<u8>,
    },
    LoadHdrSkyboxFromPath {
        path: std::path::PathBuf,
    },
}

impl World {
    pub fn queue_command(&mut self, command: WorldCommand) {
        self.resources.command_queue.push(command);
    }
}

pub fn process_commands_system(world: &mut World) {
    let commands = world.resources.command_queue.drain(..).collect::<Vec<_>>();
    let mut sprite_texture_commands = Vec::new();

    for command in commands {
        match &command {
            WorldCommand::UploadSpriteTexture { .. }
            | WorldCommand::LoadTexture { .. }
            | WorldCommand::LoadHdrSkybox { .. }
            | WorldCommand::LoadHdrSkyboxFromPath { .. } => {
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

fn despawn_entities_with_cache_cleanup(world: &mut World, entities: &[Entity]) {
    let mut has_parent_entities = false;

    for &entity in entities {
        world.resources.children_cache.remove(&entity);

        if let Some(parent_component) = world.get_parent(entity)
            && let Some(parent_entity) = parent_component.0
            && let Some(children) = world.resources.children_cache.get_mut(&parent_entity)
        {
            children.retain(|&e| e != entity);
            has_parent_entities = true;
        }

        if let Some(render_mesh) = world.get_render_mesh(entity) {
            let mesh_name = render_mesh.name.clone();
            world.resources.mesh_cache.remove_reference(&mesh_name);
        }

        if let Some(material) = world.get_material(entity) {
            let base_texture = material.base_texture.clone();
            let emissive_texture = material.emissive_texture.clone();
            let material_id = material.cache_id.clone();

            if let Some(base_texture) = base_texture {
                world
                    .resources
                    .texture_cache
                    .remove_reference(&base_texture);
            }
            if let Some(emissive_texture) = emissive_texture {
                world
                    .resources
                    .texture_cache
                    .remove_reference(&emissive_texture);
            }
            if let Some(material_id) = material_id {
                world
                    .resources
                    .material_cache
                    .remove_reference(&material_id);
            }
        }
    }

    if has_parent_entities {
        world.resources.children_cache_valid = false;
    }

    world.despawn_entities(entities);

    let removed_meshes = world.resources.mesh_cache.remove_unused();
    if !removed_meshes.is_empty() {
        tracing::debug!(
            "Immediately cleaned up {} unused meshes after despawn",
            removed_meshes.len()
        );
    }

    let removed_materials = world.resources.material_cache.remove_unused();
    if !removed_materials.is_empty() {
        tracing::debug!(
            "Immediately cleaned up {} unused materials after despawn",
            removed_materials.len()
        );
    }

    let removed_textures = world.resources.texture_cache.remove_unused();
    if !removed_textures.is_empty() {
        tracing::debug!(
            "Immediately cleaned up {} unused textures after despawn",
            removed_textures.len()
        );
    }
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

    world.set_name(entity, components::Name("Sun".to_string()));
    world.set_local_transform(
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
    world.set_local_transform_dirty(entity, components::LocalTransformDirty);
    world.set_global_transform(entity, components::GlobalTransform::default());
    world.set_light(
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

    world.set_name(entity, components::Name("Sun".to_string()));
    world.set_local_transform(
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
    world.set_local_transform_dirty(entity, components::LocalTransformDirty);
    world.set_global_transform(entity, components::GlobalTransform::default());
    world.set_light(
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
        MATERIAL, NAME, RENDER_MESH,
    };

    let entity = world.spawn_entities(
        NAME | LOCAL_TRANSFORM
            | GLOBAL_TRANSFORM
            | LOCAL_TRANSFORM_DIRTY
            | RENDER_MESH
            | MATERIAL
            | BOUNDING_VOLUME
            | CASTS_SHADOW,
        1,
    )[0];

    world.set_name(entity, components::Name(mesh_name.to_string()));
    world.set_local_transform(
        entity,
        components::LocalTransform {
            translation: position,
            scale,
            rotation: nalgebra_glm::Quat::identity(),
        },
    );
    world.set_global_transform(entity, components::GlobalTransform::default());
    world.set_local_transform_dirty(entity, components::LocalTransformDirty);
    let render_mesh = components::RenderMesh {
        name: mesh_name.to_string(),
    };

    if world.resources.mesh_cache.contains(&render_mesh.name) {
        world.resources.mesh_cache.add_reference(&render_mesh.name);
    }

    world.set_render_mesh(entity, render_mesh);

    let mut material = components::Material::default();
    let material_id = world.resources.material_cache.insert_or_get_id(&material);
    world.resources.material_cache.add_reference(&material_id);
    material.cache_id = Some(material_id);

    world.set_material(entity, material);
    world.set_bounding_volume(
        entity,
        components::BoundingVolume::from_mesh_type(mesh_name),
    );
    world.set_casts_shadow(entity, components::CastsShadow);

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

    world.set_name(entity, components::Name(text.to_string()));
    world.set_local_transform(
        entity,
        components::LocalTransform {
            translation: position,
            rotation: nalgebra_glm::Quat::identity(),
            scale: nalgebra_glm::Vec3::new(1.0, 1.0, 1.0),
        },
    );
    world.set_local_transform_dirty(entity, components::LocalTransformDirty);
    world.set_global_transform(entity, components::GlobalTransform::default());
    world.set_text(
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
    world.set_visibility(entity, components::Visibility { visible: true });

    entity
}

const CLEANUP_INTERVAL_FRAMES: u64 = 300;

pub fn cleanup_unused_resources_system(world: &mut World) {
    world.resources.cleanup_frame_counter += 1;

    if world.resources.cleanup_frame_counter >= CLEANUP_INTERVAL_FRAMES {
        world.resources.cleanup_frame_counter = 0;

        let removed_meshes = world.resources.mesh_cache.remove_unused();
        if !removed_meshes.is_empty() {
            tracing::info!(
                "Cleaned up {} unused meshes: {:?}",
                removed_meshes.len(),
                removed_meshes
            );
        }

        let removed_materials = world.resources.material_cache.remove_unused();
        if !removed_materials.is_empty() {
            tracing::info!(
                "Cleaned up {} unused materials: {:?}",
                removed_materials.len(),
                removed_materials
            );
        }

        let removed_textures = world.resources.texture_cache.remove_unused();
        if !removed_textures.is_empty() {
            tracing::info!(
                "Cleaned up {} unused textures: {:?}",
                removed_textures.len(),
                removed_textures
            );
        }
    }
}

pub fn set_material_with_textures(
    world: &mut World,
    entity: Entity,
    material: crate::ecs::material::components::Material,
) {
    let old_textures = world
        .get_material(entity)
        .map(|mat| (mat.base_texture.clone(), mat.emissive_texture.clone()));

    if let Some((old_base, old_emissive)) = old_textures {
        if let Some(base_texture) = old_base {
            world
                .resources
                .texture_cache
                .remove_reference(&base_texture);
        }
        if let Some(emissive_texture) = old_emissive {
            world
                .resources
                .texture_cache
                .remove_reference(&emissive_texture);
        }
    }

    if let Some(base_texture) = &material.base_texture {
        world.resources.texture_cache.add_reference(base_texture);
    }
    if let Some(emissive_texture) = &material.emissive_texture {
        world
            .resources
            .texture_cache
            .add_reference(emissive_texture);
    }

    world.set_material(entity, material);
}

pub fn load_procedural_textures(world: &mut World) {
    let checkerboard = generate_checkerboard_texture();
    world.queue_command(WorldCommand::LoadTexture {
        name: "checkerboard".to_string(),
        rgba_data: checkerboard.0,
        width: checkerboard.1,
        height: checkerboard.2,
    });

    let gradient = generate_gradient_texture();
    world.queue_command(WorldCommand::LoadTexture {
        name: "gradient".to_string(),
        rgba_data: gradient.0,
        width: gradient.1,
        height: gradient.2,
    });

    let uv_test = generate_uv_test_texture();
    world.queue_command(WorldCommand::LoadTexture {
        name: "uv_test".to_string(),
        rgba_data: uv_test.0,
        width: uv_test.1,
        height: uv_test.2,
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

pub fn spawn_instanced_mesh(
    world: &mut World,
    mesh_name: &str,
    instances: Vec<crate::ecs::mesh::components::InstanceTransform>,
) -> Entity {
    use crate::ecs::world::components;
    use crate::ecs::world::{
        CASTS_SHADOW, GLOBAL_TRANSFORM, INSTANCED_MESH, LOCAL_TRANSFORM, LOCAL_TRANSFORM_DIRTY,
        MATERIAL, NAME, VISIBILITY,
    };

    let entity = world.spawn_entities(
        NAME | LOCAL_TRANSFORM
            | GLOBAL_TRANSFORM
            | LOCAL_TRANSFORM_DIRTY
            | INSTANCED_MESH
            | MATERIAL
            | VISIBILITY
            | CASTS_SHADOW,
        1,
    )[0];

    world.set_name(entity, components::Name(format!("Instanced {}", mesh_name)));
    world.set_local_transform(
        entity,
        components::LocalTransform {
            translation: nalgebra_glm::Vec3::new(0.0, 0.0, 0.0),
            scale: nalgebra_glm::Vec3::new(1.0, 1.0, 1.0),
            rotation: nalgebra_glm::Quat::identity(),
        },
    );
    world.set_global_transform(entity, components::GlobalTransform::default());
    world.set_local_transform_dirty(entity, components::LocalTransformDirty);

    world.set_instanced_mesh(
        entity,
        crate::ecs::mesh::components::InstancedMesh::with_instances(mesh_name, instances),
    );

    let mut material = components::Material::default();
    let material_id = world.resources.material_cache.insert_or_get_id(&material);
    world.resources.material_cache.add_reference(&material_id);
    material.cache_id = Some(material_id);
    world.set_material(entity, material);

    world.set_visibility(entity, components::Visibility { visible: true });
    world.set_casts_shadow(entity, components::CastsShadow);

    entity
}

pub fn spawn_instanced_mesh_with_material(
    world: &mut World,
    mesh_name: &str,
    instances: Vec<crate::ecs::mesh::components::InstanceTransform>,
    material: crate::ecs::material::components::Material,
) -> Entity {
    use crate::ecs::world::components;
    use crate::ecs::world::{
        CASTS_SHADOW, GLOBAL_TRANSFORM, INSTANCED_MESH, LOCAL_TRANSFORM, LOCAL_TRANSFORM_DIRTY,
        MATERIAL, NAME, VISIBILITY,
    };

    let entity = world.spawn_entities(
        NAME | LOCAL_TRANSFORM
            | GLOBAL_TRANSFORM
            | LOCAL_TRANSFORM_DIRTY
            | INSTANCED_MESH
            | MATERIAL
            | VISIBILITY
            | CASTS_SHADOW,
        1,
    )[0];

    world.set_name(entity, components::Name(format!("Instanced {}", mesh_name)));
    world.set_local_transform(
        entity,
        components::LocalTransform {
            translation: nalgebra_glm::Vec3::new(0.0, 0.0, 0.0),
            scale: nalgebra_glm::Vec3::new(1.0, 1.0, 1.0),
            rotation: nalgebra_glm::Quat::identity(),
        },
    );
    world.set_global_transform(entity, components::GlobalTransform::default());
    world.set_local_transform_dirty(entity, components::LocalTransformDirty);

    world.set_instanced_mesh(
        entity,
        crate::ecs::mesh::components::InstancedMesh::with_instances(mesh_name, instances),
    );

    let mut material = material;
    let material_id = world.resources.material_cache.insert_or_get_id(&material);
    world.resources.material_cache.add_reference(&material_id);
    material.cache_id = Some(material_id);
    world.set_material(entity, material);

    world.set_visibility(entity, components::Visibility { visible: true });
    world.set_casts_shadow(entity, components::CastsShadow);

    entity
}
