use nightshade::prelude::*;
use nightshade_render::commands::material_registry::material_registry_find_or_insert;
use nightshade_render::commands::mesh_state::mesh_state_mark_material_dirty;
use nightshade_render::components::material::MaterialRef;
use nightshade_render::data::generational_registry::registry_add_reference;
use nightshade_render::data::material::Material;
use nightshade_render::resources::material_registry::MaterialRegistry;
use nightshade_render::resources::mesh_state::MeshRenderState;

pub fn attach(
    commands: &mut Commands,
    entity: Entity,
    name: impl Into<String>,
    material: Material,
) {
    let name = name.into();
    commands.queue(move |storage, resources| {
        let registry = resources.res_mut::<MaterialRegistry>();
        let id = material_registry_find_or_insert(registry, name, material);
        registry_add_reference(&mut registry.registry, id.index);
        storage.set(entity, MaterialRef::new(id));
        mesh_state_mark_material_dirty(resources.res_mut::<MeshRenderState>(), entity);
    });
}
