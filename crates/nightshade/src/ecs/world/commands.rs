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
        .query_entities(crate::ecs::world::PARENT)
        .filter(|child_entity| {
            world
                .get_parent(*child_entity)
                .is_some_and(|parent| parent.0 == Some(entity))
        })
        .collect()
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
    for &entity in entities {
        world.resources.children_cache.remove(&entity);

        if let Some(parent_component) = world.get_parent(entity)
            && let Some(parent_entity) = parent_component.0
            && let Some(children) = world.resources.children_cache.get_mut(&parent_entity)
        {
            children.retain(|&e| e != entity);
        }
    }

    world.despawn_entities(entities);
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
    world.set_render_mesh(
        entity,
        components::RenderMesh {
            name: mesh_name.to_string(),
        },
    );
    world.set_material(entity, components::Material::default());
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
