use nalgebra_glm::{Quat, Vec3};
use nightshade_ecs::Entity;
use nightshade_ecs::system_param::Commands;
use nightshade_render::commands::material_registry::material_registry_insert;
use nightshade_render::commands::mesh_state::mesh_state_mark_entity_added;
use nightshade_render::components::material::MaterialRef;
use nightshade_render::components::mesh::RenderMesh;
use nightshade_render::data::bounding_volume::BoundingVolume;
use nightshade_render::data::material::Material;

use nightshade_render::components::culling::CastsShadow;
use nightshade_scene::contract::primitives::Name;
use nightshade_scene::contract::{GlobalTransform, LocalTransform};

use nightshade_render::data::generational_registry::registry_add_reference;

pub fn spawn_mesh(
    commands: &mut Commands<'_>,
    mesh_name: &str,
    position: Vec3,
    scale: Vec3,
) -> Entity {
    let entity = commands.spawn((
        Name(mesh_name.to_string()),
        LocalTransform {
            translation: position,
            scale,
            rotation: Quat::identity(),
        },
        GlobalTransform::default(),
        RenderMesh::new(mesh_name),
        BoundingVolume::from_mesh_type(mesh_name),
        CastsShadow,
    ));

    let mesh_name = mesh_name.to_string();
    commands.queue(move |storage, resources| {
        nightshade_scene::commands::transform::propagate_subtree(
            resources.res_mut::<nightshade_ecs::HierarchyIndex>(),
            storage,
            entity,
        );
        mesh_state_mark_entity_added(
            resources.res_mut::<nightshade_render::resources::mesh_state::MeshRenderState>(),
            entity,
        );

        let material_name = format!("Mesh_{}", entity.id);
        let registry = resources
            .res_mut::<nightshade_render::resources::material_registry::MaterialRegistry>();
        let material_id = material_registry_insert(registry, material_name, Material::default());
        registry_add_reference(&mut registry.registry, material_id.index);
        storage.set(entity, MaterialRef::new(material_id));
        storage.set(entity, BoundingVolume::from_mesh_type(&mesh_name));
    });

    entity
}
