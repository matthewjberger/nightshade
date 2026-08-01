use crate::data::asset_id::MaterialId;
use crate::data::generational_registry::registry_lookup_index;
use crate::data::material::Material;
use crate::resources::material_registry::MaterialRegistry;

pub fn material_registry_resolve(registry: &MaterialRegistry, name: &str) -> Option<MaterialId> {
    registry_lookup_index(&registry.registry, name)
        .map(|(index, generation)| MaterialId::new(index, generation))
}

pub fn material_registry_name(registry: &MaterialRegistry, id: MaterialId) -> Option<&str> {
    crate::data::generational_registry::registry_name_for(&registry.registry, id.index)
}

pub fn material_registry_iter(
    registry: &MaterialRegistry,
) -> impl Iterator<Item = (&String, &Material)> {
    registry
        .registry
        .index_to_name
        .iter()
        .enumerate()
        .filter_map(|(index, name_opt)| {
            name_opt
                .as_ref()
                .zip(registry.registry.entries[index].as_ref())
        })
}
