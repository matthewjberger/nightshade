//! The renderer composition. The renderer itself deliberately lives
//! outside the [`World`]: it is freecs-free, consumes one owned
//! `RenderInputs` per frame, and its lifecycle (window surface, resize,
//! present) belongs to the runner. This plugin declares that a renderer
//! exists and installs the world-side sync systems that feed it.

use crate::app::{App, Plugin};
use crate::ecs::world::CORE;
use crate::ecs::world::World;

/// Registers the scene, material, animation, and skinning sync systems
/// into the render stage, after every system that writes render-facing
/// state.
pub fn register_frame_systems(stages: &mut freecs::Stages<World>) {
    use crate::app::{Stage, push_frame_system};
    push_frame_system(
        stages,
        Stage::Render,
        crate::ecs::graphics::scene_sync::render_sync_system,
    );
    push_frame_system(stages, Stage::Render, render_sync_materials_system);
    push_frame_system(
        stages,
        Stage::Render,
        crate::ecs::graphics::animation_sync::render_sync_animation_system,
    );
    push_frame_system(
        stages,
        Stage::Render,
        crate::ecs::skin::systems::render_sync_skinning_system,
    );
}

/// Adds the renderer: the surface comes up (against a hidden window
/// without `WindowPlugin`, the headless capture path), and the sync
/// systems from [`register_frame_systems`] join the frame stages to feed
/// it. Without this plugin the engine does no render work at all. The
/// optional fields override the engine's vsync and frame rate defaults at
/// composition.
#[derive(Default)]
pub struct RenderPlugin {
    pub vsync: Option<bool>,
    pub frame_rate_limit: Option<f32>,
}

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.world
            .res_mut::<crate::ecs::window::resources::Window>()
            .create_renderer = true;
        if let Some(vsync) = self.vsync {
            app.world
                .res_mut::<crate::render::config::RenderSettings>()
                .vsync_enabled = vsync;
        }
        if let Some(limit) = self.frame_rate_limit {
            app.world
                .res_mut::<crate::render::config::RenderSettings>()
                .frame_rate_limit = Some(limit);
        }
        register_frame_systems(&mut app.stages);
    }
}

fn material_table_entry(
    registry: &crate::ecs::material::resources::MaterialRegistry,
    index: usize,
) -> crate::render::config::RenderMaterialEntry {
    match registry.registry.entries[index].as_ref() {
        Some(material) => crate::render::config::RenderMaterialEntry {
            material: material.clone(),
            texture_ids: registry.texture_ids.get(index).copied().unwrap_or_default(),
        },
        None => crate::render::config::RenderMaterialEntry::default(),
    }
}

/// Resolves the scene's materials into `renderer_state.render_materials`,
/// with ids equal to the registry index offset by one, so an id never moves
/// when other materials are added or removed (id 0 is the built-in default).
/// Render passes read this instead of walking the material registry and each
/// `MaterialRef` themselves; each pass still converts the resolved materials
/// to GPU form with its own bindless texture-layer map. The table persists
/// across frames: registry edits refresh only the entries whose indices the
/// registry marked dirty, and the entity maps rebuild only when a
/// `MaterialRef` or the renderable entity set changed.
pub fn render_sync_materials_system(world: &mut World) {
    let registry_version = world
        .res::<crate::ecs::asset_state::AssetState>()
        .material_registry
        .version;
    let registry_names_version = world
        .res::<crate::ecs::asset_state::AssetState>()
        .material_registry
        .names_version;
    let (rebuild_entity_maps, first_run) = {
        let sync = &mut world.res_mut::<crate::ecs::graphics::scene_sync::RenderSceneSync>();
        let entities_dirty = std::mem::take(&mut sync.materials_dirty);
        let version_dirty = sync.materials_version_seen != registry_version;
        let names_dirty = sync.materials_names_version_seen != registry_names_version;
        let first_run = !sync.materials_initialized;
        if !entities_dirty && !version_dirty && !first_run {
            return;
        }
        sync.materials_initialized = true;
        sync.materials_version_seen = registry_version;
        sync.materials_names_version_seen = registry_names_version;
        (entities_dirty || names_dirty || first_run, first_run)
    };
    let _span = tracing::info_span!("render_sync_materials").entered();
    use crate::render::material::AlphaMode;

    let mut dirty_indices = std::mem::take(
        &mut world
            .res_mut::<crate::ecs::asset_state::AssetState>()
            .material_registry
            .dirty_indices,
    );
    dirty_indices.sort_unstable();
    dirty_indices.dedup();

    let mut scene = std::mem::take(world.res_mut::<crate::render::config::RendererState>());
    {
        let registry = &world
            .res::<crate::ecs::asset_state::AssetState>()
            .material_registry;
        let table = &mut scene.render_materials;
        let entry_count = registry.registry.entries.len();

        if first_run || table.entries.len() > entry_count {
            table.entries.clear();
            table.entries.reserve(entry_count);
            for index in 0..entry_count {
                table.entries.push(material_table_entry(registry, index));
            }
        } else {
            if table.entries.len() < entry_count {
                table.entries.resize_with(
                    entry_count,
                    crate::render::config::RenderMaterialEntry::default,
                );
            }
            for &index in &dirty_indices {
                let index = index as usize;
                if index < entry_count {
                    table.entries[index] = material_table_entry(registry, index);
                }
            }
        }

        table.material_ids.clear();
        for (index, name_slot) in registry.registry.index_to_name.iter().enumerate() {
            if name_slot.is_some() && registry.registry.entries[index].is_some() {
                table.material_ids.insert(index as u32 + 1);
            }
        }

        table.transparent_ids.clear();
        table.mask_ids.clear();
        table.double_sided_ids.clear();
        for (index, entry) in table.entries.iter().enumerate() {
            let material_id = index as u32 + 1;
            let material = &entry.material;
            if material.alpha_mode == AlphaMode::Blend || material.transmission_factor > 0.0 {
                table.transparent_ids.insert(material_id);
            }
            if matches!(material.alpha_mode, AlphaMode::Mask | AlphaMode::Dither) {
                table.mask_ids.insert(material_id);
            }
            if material.double_sided {
                table.double_sided_ids.insert(material_id);
            }
        }
    }
    *world.res_mut::<crate::render::config::RendererState>() = scene;

    {
        let table = &mut world
            .res_mut::<crate::render::config::RendererState>()
            .render_materials;
        table.generation = table.generation.wrapping_add(1);
    }

    if !rebuild_entity_maps {
        return;
    }

    let mesh_entities: Vec<crate::ecs::world::Entity> = world.ecs.worlds[CORE]
        .query_entities(crate::ecs::world::RENDER_MESH)
        .filter(|entity| {
            !world.ecs.worlds[CORE].entity_has_components(*entity, crate::ecs::world::SKIN)
        })
        .collect();
    let skinned_entities: Vec<crate::ecs::world::Entity> = world.ecs.worlds[CORE]
        .query_entities(crate::ecs::world::SKIN | crate::ecs::world::RENDER_MESH)
        .collect();
    let instanced_entities: Vec<crate::ecs::world::Entity> = world.ecs.worlds[CORE]
        .query_entities(crate::ecs::world::INSTANCED_MESH)
        .collect();
    // Meshlet geometry names its material like anything else and has to be
    // resolved here too. An entity this misses gets no id at all, which reads
    // downstream as the built-in default, so it renders grey however carefully
    // its material was authored.
    #[cfg(feature = "meshlet")]
    let meshlet_entities: Vec<crate::ecs::world::Entity> = world.ecs.worlds[CORE]
        .query_entities(crate::ecs::world::MESHLET_MESH)
        .collect();
    #[cfg(not(feature = "meshlet"))]
    let meshlet_entities: Vec<crate::ecs::world::Entity> = Vec::new();

    let registry = &world
        .res::<crate::ecs::asset_state::AssetState>()
        .material_registry;
    let mut entity_to_id: std::collections::HashMap<crate::ecs::world::Entity, u32> =
        std::collections::HashMap::new();
    for &entity in mesh_entities
        .iter()
        .chain(skinned_entities.iter())
        .chain(instanced_entities.iter())
        .chain(meshlet_entities.iter())
    {
        use crate::render::generational_registry::{registry_entry, registry_lookup_index};
        let id = match world.get::<crate::ecs::material::components::MaterialRef>(entity) {
            Some(material_ref) => material_ref
                .id
                .and_then(|id| {
                    registry_entry(&registry.registry, id.index, id.generation)
                        .map(|_| id.index + 1)
                })
                .or_else(|| {
                    registry_lookup_index(&registry.registry, &material_ref.name).and_then(
                        |(index, _generation)| {
                            registry.registry.entries[index as usize]
                                .as_ref()
                                .map(|_| index + 1)
                        },
                    )
                })
                .unwrap_or(0),
            None => 0,
        };
        entity_to_id.insert(entity, id);
    }

    for (&entity, &id) in &entity_to_id {
        world.set(entity, crate::render::render_world::MaterialId(id));
    }
}
