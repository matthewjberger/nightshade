use crate::data::queues::EcsCommand;
use nightshade_ecs::Entity;
use nightshade_ecs::system_param::{Commands, Query, ResMut};
use nightshade_render::commands::material_registry::material_registry_insert;
use nightshade_render::commands::mesh_state::{
    mesh_state_mark_entity_added, mesh_state_mark_entity_removed, mesh_state_mark_material_dirty,
};
use nightshade_render::commands::primitives::register_primitive_mesh;
use nightshade_render::commands::texture_cache::{
    texture_cache_acquire, texture_cache_release_entity,
};
use nightshade_render::components::material::MaterialRef;
use nightshade_render::data::generational_registry::{
    registry_add_reference, registry_entry_by_name, registry_name_hash, registry_remove_reference,
};
use nightshade_render::resources::texture_cache::TextureOwner;

const CLEANUP_INTERVAL_FRAMES: u64 = 10;
const CLEANUP_SWEEP_RUNS: usize = 30;
const CLEANUP_SWEEP_MINIMUM: usize = 4;
const CLEANUP_SWEEP_MAXIMUM: usize = 64;

fn sweep_budget(len: usize) -> usize {
    len.div_ceil(CLEANUP_SWEEP_RUNS)
        .clamp(CLEANUP_SWEEP_MINIMUM, CLEANUP_SWEEP_MAXIMUM)
}

pub fn register_material(
    storage: &mut nightshade_ecs::Storage,
    resources: &mut nightshade_ecs::ResourceMap,
    entity: Entity,
    name: String,
    material: nightshade_render::data::material::Material,
) {
    let registry =
        resources.res_mut::<nightshade_render::resources::material_registry::MaterialRegistry>();
    let material_id = material_registry_insert(registry, name, material);
    registry_add_reference(&mut registry.registry, material_id.index);
    storage.set(entity, MaterialRef::new(material_id));
    mesh_state_mark_entity_added(
        resources.res_mut::<nightshade_render::resources::mesh_state::MeshRenderState>(),
        entity,
    );
}

pub struct EntityCaches<'a> {
    pub entities: &'a mut nightshade_scene::resources::entity_registry::EntityRegistry,
    pub meshes: &'a mut nightshade_render::resources::mesh_cache::MeshCache,
    pub mesh_state: &'a mut nightshade_render::resources::mesh_state::MeshRenderState,
    pub textures: &'a mut nightshade_render::resources::texture_cache::TextureCache,
    pub materials: &'a mut nightshade_render::resources::material_registry::MaterialRegistry,
    pub hierarchy: &'a mut nightshade_ecs::HierarchyIndex,
}

impl<W: nightshade_ecs::ResourceHost> nightshade_ecs::system_param::ExtractParam<W>
    for EntityCaches<'_>
{
    type State = ();
    type Owned = ();
    type Item<'item> = EntityCaches<'item>;

    fn init() -> Self::State {}

    fn resources() -> Vec<nightshade_ecs::system_param::ResourceNeed> {
        use nightshade_ecs::system_param::ResourceNeed;
        vec![
            ResourceNeed::write::<nightshade_scene::resources::entity_registry::EntityRegistry>(),
            ResourceNeed::write::<nightshade_render::resources::mesh_cache::MeshCache>(),
            ResourceNeed::write::<nightshade_render::resources::mesh_state::MeshRenderState>(),
            ResourceNeed::write::<nightshade_render::resources::texture_cache::TextureCache>(),
            ResourceNeed::write::<nightshade_render::resources::material_registry::MaterialRegistry>(
            ),
            ResourceNeed::write::<nightshade_ecs::HierarchyIndex>(),
        ]
    }

    fn extract(_state: &mut (), _host: &W) {}

    fn build<'a, I: Iterator<Item = Option<nightshade_ecs::ResourceSlot<'a>>>>(
        _owned: &'a mut (),
        slots: &mut I,
        _resources: &'a nightshade_ecs::ResourceMap,
    ) -> EntityCaches<'a> {
        EntityCaches {
            entities: cache_slot(slots),
            meshes: cache_slot(slots),
            mesh_state: cache_slot(slots),
            textures: cache_slot(slots),
            materials: cache_slot(slots),
            hierarchy: cache_slot(slots),
        }
    }

    fn apply(_state: &mut (), _owned: (), _host: &W) {}

    fn access() -> nightshade_ecs::SystemAccess {
        use nightshade_ecs::{Access, SystemAccess};
        use std::any::TypeId;
        SystemAccess::empty()
            .with_resource(
                TypeId::of::<nightshade_scene::resources::entity_registry::EntityRegistry>(),
                Access::Write,
            )
            .with_resource(
                TypeId::of::<nightshade_render::resources::mesh_cache::MeshCache>(),
                Access::Write,
            )
            .with_resource(
                TypeId::of::<nightshade_render::resources::mesh_state::MeshRenderState>(),
                Access::Write,
            )
            .with_resource(
                TypeId::of::<nightshade_render::resources::texture_cache::TextureCache>(),
                Access::Write,
            )
            .with_resource(
                TypeId::of::<nightshade_render::resources::material_registry::MaterialRegistry>(),
                Access::Write,
            )
            .with_resource(
                TypeId::of::<nightshade_ecs::HierarchyIndex>(),
                Access::Write,
            )
    }
}

fn cache_slot<'a, T: Send + Sync + 'static>(
    slots: &mut impl Iterator<Item = Option<nightshade_ecs::ResourceSlot<'a>>>,
) -> &'a mut T {
    slots
        .next()
        .expect("the runner borrows one slot for each cache a despawn names")
        .expect("an entity cache names no resource it can do without")
        .into_mut::<T>()
}

pub fn entity_caches(resources: &mut nightshade_ecs::ResourceMap) -> EntityCaches<'_> {
    use std::any::TypeId;
    let [entities, meshes, mesh_state, textures, materials, hierarchy] = resources
        .res_disjoint_mut([
            TypeId::of::<nightshade_scene::resources::entity_registry::EntityRegistry>(),
            TypeId::of::<nightshade_render::resources::mesh_cache::MeshCache>(),
            TypeId::of::<nightshade_render::resources::mesh_state::MeshRenderState>(),
            TypeId::of::<nightshade_render::resources::texture_cache::TextureCache>(),
            TypeId::of::<nightshade_render::resources::material_registry::MaterialRegistry>(),
            TypeId::of::<nightshade_ecs::HierarchyIndex>(),
        ]);
    EntityCaches {
        entities: entities.downcast_mut().expect("EntityRegistry"),
        meshes: meshes.downcast_mut().expect("MeshCache"),
        mesh_state: mesh_state.downcast_mut().expect("MeshRenderState"),
        textures: textures.downcast_mut().expect("TextureCache"),
        materials: materials.downcast_mut().expect("MaterialRegistry"),
        hierarchy: hierarchy.downcast_mut().expect("HierarchyIndex"),
    }
}

pub fn process_commands_system(
    mut queues: ResMut<crate::resources::queues::CommandQueues>,
    mut entities: ResMut<nightshade_scene::resources::entity_registry::EntityRegistry>,
    mut meshes: ResMut<nightshade_render::resources::mesh_cache::MeshCache>,
    mut mesh_state: ResMut<nightshade_render::resources::mesh_state::MeshRenderState>,
    mut textures: ResMut<nightshade_render::resources::texture_cache::TextureCache>,
    mut materials: ResMut<nightshade_render::resources::material_registry::MaterialRegistry>,
    mut hierarchy: ResMut<nightshade_ecs::HierarchyIndex>,
    storage: &mut nightshade_ecs::Storage,
) {
    let commands = std::mem::take(&mut queues.ecs);
    let mut caches = EntityCaches {
        entities: &mut entities,
        meshes: &mut meshes,
        mesh_state: &mut mesh_state,
        textures: &mut textures,
        materials: &mut materials,
        hierarchy: &mut hierarchy,
    };
    for command in commands {
        match command {
            EcsCommand::DespawnRecursive { entity } => {
                despawn_recursive_now(storage, &mut caches, entity);
            }
            EcsCommand::ReloadMaterial { name, material } => {
                reload_material_now(storage, &mut caches, name, *material);
            }
        }
    }
}

fn reload_material_now(
    storage: &mut nightshade_ecs::Storage,
    caches: &mut EntityCaches<'_>,
    name: String,
    new_material: nightshade_render::data::material::Material,
) {
    let new_textures: Vec<String> = new_material.texture_names().map(str::to_string).collect();

    nightshade_render::commands::material_registry::material_registry_mutate(
        caches.materials,
        &name,
        |existing| *existing = new_material,
    );

    let Some(material_id) =
        nightshade_render::queries::material_registry::material_registry_resolve(
            caches.materials,
            &name,
        )
    else {
        return;
    };

    let referencing_entities: Vec<Entity> = storage
        .query_entities::<(nightshade_render::components::material::MaterialRef,)>()
        .filter(|&entity| {
            storage
                .get::<nightshade_render::components::material::MaterialRef>(entity)
                .is_some_and(|mat_ref| mat_ref.id == material_id)
        })
        .collect();

    for entity in referencing_entities {
        texture_cache_acquire(
            caches.textures,
            TextureOwner::EntityMaterial(entity),
            new_textures.clone(),
        );
        mesh_state_mark_material_dirty(caches.mesh_state, entity);
    }
}

pub fn despawn_entities_with_cache_cleanup(
    storage: &mut nightshade_ecs::Storage,
    resources: &mut nightshade_ecs::ResourceMap,
    entities: &[Entity],
) {
    let mut caches = entity_caches(resources);
    despawn_entities_now(storage, &mut caches, entities);
}

pub fn despawn_entities_now(
    storage: &mut nightshade_ecs::Storage,
    caches: &mut EntityCaches<'_>,
    entities: &[Entity],
) {
    for &entity in entities {
        caches.entities.tags.remove(&entity);
        if let Some(guid) = storage
            .get::<nightshade_scene::contract::primitives::Guid>(entity)
            .copied()
        {
            caches.entities.guid_index.remove(&guid.0);
        }

        if let Some(render_mesh) =
            storage.get::<nightshade_render::components::mesh::RenderMesh>(entity)
        {
            let mesh_name = render_mesh.name.clone();
            if let Some(&index) = caches.meshes.registry.name_to_index.get(&mesh_name) {
                registry_remove_reference(&mut caches.meshes.registry, index);
            }
            mesh_state_mark_entity_removed(caches.mesh_state, entity);
        }

        if let Some(instanced_mesh) =
            storage.get::<nightshade_render::components::mesh::InstancedMesh>(entity)
        {
            let mesh_name = instanced_mesh.mesh_name.clone();
            if let Some(&index) = caches.meshes.registry.name_to_index.get(&mesh_name) {
                registry_remove_reference(&mut caches.meshes.registry, index);
            }
            caches.mesh_state.instanced_meshes_changed = true;
        }

        texture_cache_release_entity(caches.textures, entity);

        if let Some(material_ref) =
            storage.get::<nightshade_render::components::material::MaterialRef>(entity)
        {
            let index = material_ref.id.index;
            registry_remove_reference(&mut caches.materials.registry, index);
        }
    }

    storage.despawn_entities(entities);
}

pub fn despawn_recursive_immediate(
    storage: &mut nightshade_ecs::Storage,
    resources: &mut nightshade_ecs::ResourceMap,
    entity: Entity,
) {
    let mut caches = entity_caches(resources);
    despawn_recursive_now(storage, &mut caches, entity);
}

pub fn despawn_recursive_now(
    storage: &mut nightshade_ecs::Storage,
    caches: &mut EntityCaches<'_>,
    entity: Entity,
) {
    caches.hierarchy.sync(storage);
    let mut targets = caches.hierarchy.descendants(entity);
    targets.insert(0, entity);
    despawn_entities_now(storage, caches, &targets);
}

pub fn spawn_sun(commands: &mut Commands) -> Entity {
    use nightshade_render::components;
    commands.spawn((
        components::Name("Sun".to_string()),
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
        components::GlobalTransform::default(),
    ))
}

pub fn spawn_sun_without_shadows(commands: &mut Commands) -> Entity {
    use nightshade_render::components;
    commands.spawn((
        components::Name("Sun".to_string()),
        components::LocalTransform {
            translation: nalgebra_glm::Vec3::new(5.0, 10.0, 5.0),
            rotation: nalgebra_glm::quat_angle_axis(
                -std::f32::consts::FRAC_PI_4,
                &nalgebra_glm::Vec3::new(1.0, 0.0, 0.0),
            ),
            scale: nalgebra_glm::Vec3::new(1.0, 1.0, 1.0),
        },
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
        components::GlobalTransform::default(),
    ))
}

pub fn spawn_mesh_at(
    commands: &mut Commands,
    mesh_name: &str,
    position: nalgebra_glm::Vec3,
    scale: nalgebra_glm::Vec3,
) -> Entity {
    use nightshade_render::components;

    let local_transform = components::LocalTransform {
        translation: position,
        scale,
        rotation: nalgebra_glm::Quat::identity(),
    };
    let global_transform = components::GlobalTransform(local_transform.as_matrix());

    let entity = commands.spawn((
        components::Name(mesh_name.to_string()),
        local_transform,
        components::RenderMesh::new(mesh_name),
        components::MaterialRef::default(),
        components::BoundingVolume::from_mesh_type(mesh_name),
        components::CastsShadow,
        nightshade_scene::contract::primitives::Visibility { visible: true },
        global_transform,
    ));

    let mesh_name = mesh_name.to_string();
    commands.queue(move |_, resources| {
        register_primitive_mesh(resources, &mesh_name);
        mesh_state_mark_entity_added(
            resources.res_mut::<nightshade_render::resources::mesh_state::MeshRenderState>(),
            entity,
        );
    });

    entity
}

pub fn spawn_mesh_with_material(
    commands: &mut Commands,
    mesh_name: &str,
    position: nalgebra_glm::Vec3,
    scale: nalgebra_glm::Vec3,
    material_name: &str,
) -> Entity {
    let entity = spawn_mesh_at(commands, mesh_name, position, scale);
    set_mesh_material(commands, entity, material_name);
    entity
}

pub fn set_mesh_material(commands: &mut Commands, entity: Entity, material_name: &str) {
    let material_name = material_name.to_string();
    commands.queue(move |storage, resources| {
        let registry = resources
            .res_mut::<nightshade_render::resources::material_registry::MaterialRegistry>();
        let Some(material_id) =
            nightshade_render::queries::material_registry::material_registry_resolve(
                registry,
                material_name.as_str(),
            )
        else {
            return;
        };
        registry_add_reference(&mut registry.registry, material_id.index);
        let texture_names: Vec<String> = registry_entry_by_name(
            &resources
                .res::<nightshade_render::resources::material_registry::MaterialRegistry>()
                .registry,
            material_name.as_str(),
        )
        .map(|material| material.texture_names().map(str::to_string).collect())
        .unwrap_or_default();
        texture_cache_acquire(
            resources.res_mut::<nightshade_render::resources::texture_cache::TextureCache>(),
            TextureOwner::EntityMaterial(entity),
            texture_names,
        );
        storage.set(entity, MaterialRef::new(material_id));
    });
}

pub fn spawn_cube_at(commands: &mut Commands, position: nalgebra_glm::Vec3) -> Entity {
    spawn_mesh_at(
        commands,
        "Cube",
        position,
        nalgebra_glm::Vec3::new(1.0, 1.0, 1.0),
    )
}

pub fn spawn_sphere_at(commands: &mut Commands, position: nalgebra_glm::Vec3) -> Entity {
    spawn_mesh_at(
        commands,
        "Sphere",
        position,
        nalgebra_glm::Vec3::new(1.0, 1.0, 1.0),
    )
}

pub fn spawn_cylinder_at(commands: &mut Commands, position: nalgebra_glm::Vec3) -> Entity {
    spawn_mesh_at(
        commands,
        "Cylinder",
        position,
        nalgebra_glm::Vec3::new(1.0, 1.0, 1.0),
    )
}

pub fn spawn_torus_at(commands: &mut Commands, position: nalgebra_glm::Vec3) -> Entity {
    spawn_mesh_at(
        commands,
        "Torus",
        position,
        nalgebra_glm::Vec3::new(1.0, 1.0, 1.0),
    )
}

pub fn spawn_cone_at(commands: &mut Commands, position: nalgebra_glm::Vec3) -> Entity {
    spawn_mesh_at(
        commands,
        "Cone",
        position,
        nalgebra_glm::Vec3::new(1.0, 1.0, 1.0),
    )
}

pub fn spawn_plane_at(commands: &mut Commands, position: nalgebra_glm::Vec3) -> Entity {
    spawn_mesh_at(
        commands,
        "Plane",
        position,
        nalgebra_glm::Vec3::new(1.0, 1.0, 1.0),
    )
}

use std::collections::HashSet;

pub fn cleanup_unused_resources_system(
    mut meshes: ResMut<nightshade_render::resources::mesh_cache::MeshCache>,
    mut materials: ResMut<nightshade_render::resources::material_registry::MaterialRegistry>,
    mut cleanup: ResMut<nightshade_render::resources::cleanup::CleanupState>,
    drawn: Query<(&nightshade_render::components::mesh::RenderMesh,)>,
    instanced: Query<(&nightshade_render::components::mesh::InstancedMesh,)>,
    referenced: Query<(&nightshade_render::components::material::MaterialRef,)>,
) {
    cleanup.sweep_frame = cleanup.sweep_frame.wrapping_add(1);
    if !cleanup.sweep_frame.is_multiple_of(CLEANUP_INTERVAL_FRAMES) {
        return;
    }
    let mut live_meshes: HashSet<u64> = HashSet::new();
    drawn.for_each(|_entity, (mesh,)| {
        live_meshes.insert(registry_name_hash(&mesh.name));
    });
    instanced.for_each(|_entity, (mesh,)| {
        live_meshes.insert(registry_name_hash(&mesh.mesh_name));
    });

    let mut live_materials: HashSet<u32> = HashSet::new();
    referenced.for_each(|_entity, (material,)| {
        live_materials.insert(material.id.index);
    });

    let mesh_len = meshes.registry.entries.len();
    let (_, next_mesh_cursor) = nightshade_render::commands::mesh_cache::mesh_cache_reclaim_range(
        &mut meshes,
        &live_meshes,
        cleanup.mesh_cursor,
        sweep_budget(mesh_len),
    );
    cleanup.mesh_cursor = next_mesh_cursor;

    let material_len = materials.registry.entries.len();
    cleanup.material_cursor =
        nightshade_render::commands::material_registry::material_registry_reclaim_range(
            &mut materials,
            &live_materials,
            cleanup.material_cursor,
            sweep_budget(material_len),
        );
}

pub fn release_dead_texture_owners_system(
    mut textures: ResMut<nightshade_render::resources::texture_cache::TextureCache>,
    mut cleanup: ResMut<nightshade_render::resources::cleanup::CleanupState>,
    storage: &mut nightshade_ecs::Storage,
) {
    cleanup.owner_frame = cleanup.owner_frame.wrapping_add(1);
    if !(cleanup.owner_frame + CLEANUP_INTERVAL_FRAMES / 2).is_multiple_of(CLEANUP_INTERVAL_FRAMES)
    {
        return;
    }
    nightshade_render::commands::texture_cache::texture_cache_release_dead_entities(
        &mut textures,
        |entity| storage.is_alive(entity),
    );
}

pub fn set_material_with_textures(
    storage: &mut nightshade_ecs::Storage,
    resources: &mut nightshade_ecs::ResourceMap,
    entity: Entity,
    material: nightshade_render::data::material::Material,
) {
    let material_ref = storage
        .get::<nightshade_render::components::material::MaterialRef>(entity)
        .cloned();

    let texture_names: Vec<String> = material.texture_names().map(str::to_string).collect();
    texture_cache_acquire(
        resources.res_mut::<nightshade_render::resources::texture_cache::TextureCache>(),
        TextureOwner::EntityMaterial(entity),
        texture_names,
    );

    if let Some(ref mat_ref) = material_ref {
        nightshade_render::commands::material_registry::material_registry_mutate_id(
            resources
                .res_mut::<nightshade_render::resources::material_registry::MaterialRegistry>(),
            mat_ref.id,
            |existing_mat| *existing_mat = material,
        );
        mesh_state_mark_material_dirty(
            resources.res_mut::<nightshade_render::resources::mesh_state::MeshRenderState>(),
            entity,
        );
    } else {
        let material_name = format!("TexturedMaterial_{}", entity.id);
        let registry = resources
            .res_mut::<nightshade_render::resources::material_registry::MaterialRegistry>();
        let material_id = material_registry_insert(registry, material_name, material);
        registry_add_reference(&mut registry.registry, material_id.index);
        storage.set(
            entity,
            nightshade_render::components::material::MaterialRef::new(material_id),
        );
    }
}

pub fn spawn_instanced_mesh(
    storage: &mut nightshade_ecs::Storage,
    resources: &mut nightshade_ecs::ResourceMap,
    mesh_name: &str,
    instances: Vec<nightshade_render::components::mesh::InstanceTransform>,
) -> Entity {
    use nightshade_render::components;

    let entity = storage.spawn_entities::<(
        components::Name,
        components::LocalTransform,
        components::GlobalTransform,
        components::InstancedMesh,
        components::MaterialRef,
        components::Visibility,
        components::CastsShadow,
    )>(1)[0];

    storage.set(entity, components::Name(format!("Instanced {}", mesh_name)));
    storage.set(
        entity,
        components::LocalTransform {
            translation: nalgebra_glm::Vec3::new(0.0, 0.0, 0.0),
            scale: nalgebra_glm::Vec3::new(1.0, 1.0, 1.0),
            rotation: nalgebra_glm::Quat::identity(),
        },
    );
    storage.set(entity, components::GlobalTransform::default());

    storage.set(
        entity,
        nightshade_render::components::mesh::InstancedMesh::with_instances(mesh_name, instances),
    );

    if let Some(&index) = resources
        .res::<nightshade_render::resources::material_registry::MaterialRegistry>()
        .registry
        .name_to_index
        .get("Default")
    {
        registry_add_reference(
            &mut resources
                .res_mut::<nightshade_render::resources::material_registry::MaterialRegistry>()
                .registry,
            index,
        );
    }
    storage.set(entity, components::MaterialRef::default());

    storage.set(entity, components::Visibility { visible: true });
    storage.set(entity, components::CastsShadow);

    entity
}

pub fn spawn_instanced_mesh_with_material(
    storage: &mut nightshade_ecs::Storage,
    resources: &mut nightshade_ecs::ResourceMap,
    mesh_name: &str,
    instances: Vec<nightshade_render::components::mesh::InstanceTransform>,
    material_id: nightshade_render::data::asset_id::MaterialId,
) -> Entity {
    use nightshade_render::components;

    let entity = storage.spawn_entities::<(
        components::Name,
        components::LocalTransform,
        components::GlobalTransform,
        components::InstancedMesh,
        components::MaterialRef,
        components::Visibility,
        components::CastsShadow,
    )>(1)[0];

    storage.set(entity, components::Name(format!("Instanced {}", mesh_name)));
    storage.set(
        entity,
        components::LocalTransform {
            translation: nalgebra_glm::Vec3::new(0.0, 0.0, 0.0),
            scale: nalgebra_glm::Vec3::new(1.0, 1.0, 1.0),
            rotation: nalgebra_glm::Quat::identity(),
        },
    );
    storage.set(entity, components::GlobalTransform::default());

    storage.set(
        entity,
        nightshade_render::components::mesh::InstancedMesh::with_instances(mesh_name, instances),
    );

    registry_add_reference(
        &mut resources
            .res_mut::<nightshade_render::resources::material_registry::MaterialRegistry>()
            .registry,
        material_id.index,
    );

    let texture_names: Vec<String> =
        nightshade_render::data::generational_registry::registry_entry(
            &resources
                .res::<nightshade_render::resources::material_registry::MaterialRegistry>()
                .registry,
            material_id.index,
            material_id.generation,
        )
        .map(|mat| mat.texture_names().map(str::to_string).collect())
        .unwrap_or_default();
    texture_cache_acquire(
        resources.res_mut::<nightshade_render::resources::texture_cache::TextureCache>(),
        TextureOwner::EntityMaterial(entity),
        texture_names,
    );

    storage.set(entity, components::MaterialRef::new(material_id));

    storage.set(entity, components::Visibility { visible: true });
    storage.set(entity, components::CastsShadow);

    entity
}

pub fn spawn_material(
    storage: &mut nightshade_ecs::Storage,
    materials: &mut nightshade_render::resources::material_registry::MaterialRegistry,
    entity: Entity,
    name: String,
    material: nightshade_render::data::material::Material,
) {
    let material_id = material_registry_insert(materials, name, material);
    registry_add_reference(&mut materials.registry, material_id.index);
    storage.set(entity, MaterialRef::new(material_id));
}
