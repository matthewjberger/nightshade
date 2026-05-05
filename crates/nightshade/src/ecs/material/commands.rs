use crate::ecs::world::components::MaterialRef;
use crate::ecs::world::{MATERIAL_REF, MATERIAL_VARIANTS, World};
use std::collections::HashSet;

/// Switches every entity with [`MaterialVariants`] to the material that
/// covers the supplied variant name, or back to its default if
/// `variant_name` is `None`. Updates [`MaterialRef`] in place so the
/// next prepare pass re-resolves to the new material.
///
/// Returns the number of entities whose material was changed.
///
/// [`MaterialVariants`]: super::components::MaterialVariants
pub fn material_variant_apply(world: &mut World, variant_name: Option<&str>) -> usize {
    let mut updates: Vec<(freecs::Entity, String)> = Vec::new();
    world
        .core
        .query()
        .with(MATERIAL_VARIANTS | MATERIAL_REF)
        .iter(|entity, table, idx| {
            let variants = &table.material_variants[idx];
            let target_name = match variant_name {
                None => variants.default_material_name.clone(),
                Some(name) => variants.material_for_variant(name).to_string(),
            };
            let current_name = &table.material_ref[idx].name;
            if current_name != &target_name {
                updates.push((entity, target_name));
            }
        });

    let count = updates.len();
    for (entity, name) in updates {
        world.core.set_material_ref(entity, MaterialRef::new(name));
    }
    if count > 0 {
        world.resources.mesh_render_state.request_full_rebuild();
    }
    count
}

/// Returns the deduplicated variant names referenced by any currently-spawned
/// entity carrying a [`super::components::MaterialVariants`] component. Order
/// follows first observation while iterating entities and their mappings.
pub fn material_variant_names(world: &World) -> Vec<String> {
    let mut seen_set: HashSet<String> = HashSet::new();
    let mut seen: Vec<String> = Vec::new();
    world
        .core
        .query()
        .with(MATERIAL_VARIANTS)
        .iter(|_, table, idx| {
            let variants = &table.material_variants[idx];
            for mapping in &variants.mappings {
                for name in &mapping.variant_names {
                    if seen_set.insert(name.clone()) {
                        seen.push(name.clone());
                    }
                }
            }
        });
    seen
}
