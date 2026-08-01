use nightshade::prelude::*;
use nightshade_graphics::prelude::material_registry_insert;
use nightshade_render::components::material::MaterialRef;
use nightshade_render::data::material::Material;
use nightshade_render::resources::material_registry::MaterialRegistry;

pub fn dress(commands: &mut Commands<'_>, entity: Entity, material: Material) {
    let name = format!("coat {}", entity.id);
    commands.queue(move |storage, resources| {
        let id = material_registry_insert(
            resources.res_mut::<MaterialRegistry>(),
            name.clone(),
            material.clone(),
        );
        storage.set(entity, MaterialRef::new(id));
    });
}
