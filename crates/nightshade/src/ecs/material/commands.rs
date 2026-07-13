use crate::ecs::world::World;
use crate::ecs::world::components::MaterialRef;

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
    for (entity, (variants, material_ref)) in world
        .query_ref::<(
            &super::components::MaterialVariants,
            &super::components::MaterialRef,
        )>()
        .iter()
    {
        let target_name = match variant_name {
            None => variants.default_material_name.clone(),
            Some(name) => variants.material_for_variant(name).to_string(),
        };
        if material_ref.name != target_name {
            updates.push((entity, target_name));
        }
    }

    let count = updates.len();
    for (entity, name) in updates {
        world.set(entity, MaterialRef::new(name));
    }
    if count > 0 {
        world.resources.mesh_render_state.request_full_rebuild();
    }
    count
}
