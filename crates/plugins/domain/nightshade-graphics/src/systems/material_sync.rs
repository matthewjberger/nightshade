use nightshade::ecs::system_param::ResMut;

fn material_table_entry(
    registry: &nightshade_render::resources::material_registry::MaterialRegistry,
    index: usize,
) -> nightshade_render::data::config::RenderMaterialEntry {
    match registry.registry.entries[index].as_ref() {
        Some(material) => nightshade_render::data::config::RenderMaterialEntry {
            material: material.clone(),
            texture_ids: registry.texture_ids.get(index).copied().unwrap_or_default(),
        },
        None => nightshade_render::data::config::RenderMaterialEntry::default(),
    }
}

pub fn render_sync_materials_system(
    mut registry: ResMut<nightshade_render::resources::material_registry::MaterialRegistry>,
    mut sync: ResMut<nightshade_render::resources::render_sync::RenderSceneSync>,
    mut renderer_state: ResMut<nightshade_render::data::config::RendererState>,
    storage: &mut nightshade::ecs::Storage,
) {
    let registry_version = registry.version;
    let registry_names_version = registry.names_version;
    let (rebuild_entity_maps, first_run) = {
        let sync = &mut sync;
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
    use nightshade_render::data::material::AlphaMode;

    let mut dirty_indices = std::mem::take(&mut registry.dirty_indices);
    dirty_indices.sort_unstable();
    dirty_indices.dedup();
    let mut classes_changed = first_run;

    {
        let registry = &*registry;
        let table = &mut renderer_state.render_materials;
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
                    nightshade_render::data::config::RenderMaterialEntry::default,
                );
            }
            for &index in &dirty_indices {
                let index = index as usize;
                if index < entry_count {
                    table.entries[index] = material_table_entry(registry, index);
                }
            }
        }

        let before = (
            table.material_ids.clone(),
            table.transparent_ids.clone(),
            table.mask_ids.clone(),
            table.double_sided_ids.clone(),
            table.transmissive_ids.clone(),
        );
        table.material_ids.clear();
        for (index, name_slot) in registry.registry.index_to_name.iter().enumerate() {
            if name_slot.is_some() && registry.registry.entries[index].is_some() {
                table.material_ids.insert(index as u32 + 1);
            }
        }

        table.transparent_ids.clear();
        table.mask_ids.clear();
        table.double_sided_ids.clear();
        table.transmissive_ids.clear();
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
            if material.transmission_factor > 0.0 {
                table.transmissive_ids.insert(material_id);
            }
        }
        let after = (
            &table.material_ids,
            &table.transparent_ids,
            &table.mask_ids,
            &table.double_sided_ids,
            &table.transmissive_ids,
        );
        if (&before.0, &before.1, &before.2, &before.3, &before.4) != after {
            classes_changed = true;
        }
    }

    {
        let table = &mut renderer_state.render_materials;
        table.generation = table.generation.wrapping_add(1);
        if classes_changed {
            table.class_generation = table.class_generation.wrapping_add(1);
        }
    }

    if !rebuild_entity_maps {
        return;
    }

    let mesh_entities: Vec<nightshade::ecs::Entity> = storage
        .query_entities::<(nightshade_render::components::mesh::RenderMesh,)>()
        .filter(|entity| {
            !storage.entity_has_components::<(nightshade_render::components::skin::Skin,)>(*entity)
        })
        .collect();
    let skinned_entities: Vec<nightshade::ecs::Entity> = storage
        .query_entities::<(
            nightshade_render::components::skin::Skin,
            nightshade_render::components::mesh::RenderMesh,
        )>()
        .collect();
    let instanced_entities: Vec<nightshade::ecs::Entity> = storage
        .query_entities::<(nightshade_render::components::mesh::InstancedMesh,)>()
        .collect();

    let registry = &*registry;
    for &entity in mesh_entities
        .iter()
        .chain(skinned_entities.iter())
        .chain(instanced_entities.iter())
    {
        use nightshade_render::data::generational_registry::registry_entry;
        let slot = storage
            .get::<nightshade_render::components::material::MaterialRef>(entity)
            .and_then(|material_ref| {
                registry_entry(
                    &registry.registry,
                    material_ref.id.index,
                    material_ref.id.generation,
                )
                .map(|_| material_ref.id.index + 1)
            })
            .unwrap_or(0);
        storage.set(
            entity,
            nightshade_render::components::render_world::MaterialId(slot),
        );
    }
}
