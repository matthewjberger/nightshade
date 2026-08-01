use nightshade_render::components::material::MaterialRef;
use nightshade_render::data::asset_id::MaterialId;
use nightshade_render::queries::material_registry::material_registry_resolve;

pub fn material_variant_apply(
    variants: &mut MaterialVariantRows<'_, '_>,
    registry: &nightshade_render::resources::material_registry::MaterialRegistry,
    commands: &mut nightshade_ecs::system_param::Commands<'_>,
    variant_name: Option<&str>,
) -> usize {
    let mut updates: Vec<(nightshade_ecs::Entity, MaterialId)> = Vec::new();
    variants.for_each_mut(|entity, (variants, material_ref)| {
        let target_name = match variant_name {
            None => variants.default_material_name.as_str(),
            Some(name) => variants.material_for_variant(name),
        };
        let Some(target_id) = material_registry_resolve(registry, target_name) else {
            return;
        };
        if material_ref.id != target_id {
            updates.push((entity, target_id));
        }
    });

    let count = updates.len();
    for (entity, id) in updates {
        commands.set(entity, MaterialRef::new(id));
    }
    if count > 0 {
        commands.queue(|_, resources| {
            resources
                .res_mut::<nightshade_render::resources::mesh_state::MeshRenderState>()
                .full_rebuild_needed = true;
        });
    }
    count
}

pub type MaterialVariantRows<'world, 'item> = nightshade_ecs::system_param::Query<
    'world,
    (
        &'item nightshade_render::components::material::MaterialVariants,
        &'item nightshade_render::components::material::MaterialRef,
    ),
>;
