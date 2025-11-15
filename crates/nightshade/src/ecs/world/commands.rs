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
            WorldCommand::UploadSpriteTexture { .. } | WorldCommand::LoadTexture { .. } => {
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
            let descendents = query_descendents(world, entity);
            despawn_entities_with_cache_cleanup(world, &descendents);
        }
        WorldCommand::UploadSpriteTexture { .. } => {}
        WorldCommand::LoadTexture { .. } => {}
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
        log::debug!(
            "Immediately cleaned up {} unused meshes after despawn",
            removed_meshes.len()
        );
    }

    let removed_materials = world.resources.material_cache.remove_unused();
    if !removed_materials.is_empty() {
        log::debug!(
            "Immediately cleaned up {} unused materials after despawn",
            removed_materials.len()
        );
    }

    let removed_textures = world.resources.texture_cache.remove_unused();
    if !removed_textures.is_empty() {
        log::debug!(
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
                -0.5,
                &nalgebra_glm::Vec3::new(1.0, 0.0, 0.5).normalize(),
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
        BOUNDING_VOLUME, GLOBAL_TRANSFORM, LOCAL_TRANSFORM, LOCAL_TRANSFORM_DIRTY, MATERIAL, NAME,
        RENDER_MESH,
    };

    let entity = world.spawn_entities(
        NAME | LOCAL_TRANSFORM
            | GLOBAL_TRANSFORM
            | LOCAL_TRANSFORM_DIRTY
            | RENDER_MESH
            | MATERIAL
            | BOUNDING_VOLUME,
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
            log::info!(
                "Cleaned up {} unused meshes: {:?}",
                removed_meshes.len(),
                removed_meshes
            );
        }

        let removed_materials = world.resources.material_cache.remove_unused();
        if !removed_materials.is_empty() {
            log::info!(
                "Cleaned up {} unused materials: {:?}",
                removed_materials.len(),
                removed_materials
            );
        }

        let removed_textures = world.resources.texture_cache.remove_unused();
        if !removed_textures.is_empty() {
            log::info!(
                "Cleaned up {} unused textures: {:?}",
                removed_textures.len(),
                removed_textures
            );
        }
    }
}
