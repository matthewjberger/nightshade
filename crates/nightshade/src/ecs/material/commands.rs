use crate::ecs::world::components::MaterialRef;
use crate::ecs::world::{MATERIAL_REF, MATERIAL_VARIANTS, World};

/// Switches every entity with [`MaterialVariants`] to the material that
/// covers the supplied variant name, or back to its default if
/// `variant_name` is `None`. Updates [`MaterialRef`] in place so the
/// next prepare pass re-resolves to the new material.
///
/// Returns the number of entities whose material was changed.
///
/// [`MaterialVariants`]: super::components::MaterialVariants
pub fn material_variant_apply(world: &mut World, variant_name: Option<&str>) -> usize {
    let entities: Vec<_> = world
        .core
        .query_entities(MATERIAL_VARIANTS | MATERIAL_REF)
        .collect();

    let mut updates: Vec<(freecs::Entity, String)> = Vec::with_capacity(entities.len());
    for entity in entities {
        let Some(variants) = world.core.get_material_variants(entity) else {
            continue;
        };
        let target_name = match variant_name {
            None => variants.default_material_name.clone(),
            Some(name) => variants.material_for_variant(name).to_string(),
        };
        let current_name = world
            .core
            .get_material_ref(entity)
            .map(|r| r.name.clone())
            .unwrap_or_default();
        if current_name != target_name {
            updates.push((entity, target_name));
        }
    }

    let count = updates.len();
    for (entity, name) in updates {
        world.core.set_material_ref(entity, MaterialRef::new(name));
    }
    if count > 0 {
        world.resources.mesh_render_state.request_full_rebuild();
    }
    count
}

/// Returns the deduplicated variant names exposed by any currently-loaded
/// prefab in [`crate::ecs::prefab::resources::PrefabCache`]. Order is the
/// first observation across cached prefabs.
#[cfg(feature = "assets")]
pub fn material_variant_names(world: &World) -> Vec<String> {
    let mut seen: Vec<String> = Vec::new();
    for cached in world.resources.prefab_cache.prefabs.values() {
        for name in &cached.prefab.material_variants {
            if !seen.iter().any(|existing| existing == name) {
                seen.push(name.clone());
            }
        }
    }
    seen
}

#[cfg(not(feature = "assets"))]
pub fn material_variant_names(_world: &World) -> Vec<String> {
    Vec::new()
}
