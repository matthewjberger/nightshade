use nalgebra_glm;
use nightshade::ecs::Entity;
use nightshade::ecs::system_param::Query;
use nightshade_render::data::generational_registry::registry_entry_by_name;
use rapier3d::prelude::*;

pub type RenderMeshes<'world, 'item> =
    Query<'world, (&'item nightshade_render::components::mesh::RenderMesh,)>;

pub type GlobalTransforms<'world, 'item> =
    Query<'world, (&'item nightshade_scene::contract::GlobalTransform,)>;

fn decompose(
    global_transform: &nightshade_scene::contract::GlobalTransform,
) -> (Isometry<Real>, nalgebra_glm::Vec3) {
    let translation = global_transform.0.column(3).xyz();
    let rotation_matrix = global_transform.0.fixed_view::<3, 3>(0, 0);
    let scale = nalgebra_glm::vec3(
        rotation_matrix.column(0).magnitude(),
        rotation_matrix.column(1).magnitude(),
        rotation_matrix.column(2).magnitude(),
    );

    let rotation_normalized = nalgebra::Matrix3::new(
        rotation_matrix[(0, 0)] / scale.x,
        rotation_matrix[(0, 1)] / scale.y,
        rotation_matrix[(0, 2)] / scale.z,
        rotation_matrix[(1, 0)] / scale.x,
        rotation_matrix[(1, 1)] / scale.y,
        rotation_matrix[(1, 2)] / scale.z,
        rotation_matrix[(2, 0)] / scale.x,
        rotation_matrix[(2, 1)] / scale.y,
        rotation_matrix[(2, 2)] / scale.z,
    );

    let rotation = nalgebra::UnitQuaternion::from_rotation_matrix(
        &nalgebra::Rotation3::from_matrix(&rotation_normalized),
    );

    let position = Isometry::from_parts(
        nalgebra::Translation3::new(translation.x, translation.y, translation.z),
        rotation,
    );
    (position, scale)
}

pub fn register_entity_for_trimesh_picking(
    picking_world: &mut crate::resources::PickingWorld,
    mesh_cache: &nightshade_render::resources::mesh_cache::MeshCache,
    meshes: &mut RenderMeshes<'_, '_>,
    transforms: &mut GlobalTransforms<'_, '_>,
    entity: Entity,
) -> bool {
    let mut mesh_name = None;
    meshes.for_entity(entity, |(render_mesh,)| {
        mesh_name = Some(render_mesh.name.clone())
    });
    let Some(mesh_name) = mesh_name else {
        return false;
    };

    let Some(mesh) = registry_entry_by_name(&mesh_cache.registry, &mesh_name) else {
        return false;
    };

    let mut global_transform = None;
    transforms.for_entity(entity, |(global,)| global_transform = Some(*global));
    let Some(global_transform) = global_transform else {
        return false;
    };

    let vertices: Vec<Point<Real>> = mesh
        .vertices
        .iter()
        .map(|vertex| Point::new(vertex.position[0], vertex.position[1], vertex.position[2]))
        .collect();

    let indices: Vec<[u32; 3]> = mesh
        .indices
        .chunks(3)
        .filter_map(|chunk| {
            if chunk.len() == 3 {
                Some([chunk[0], chunk[1], chunk[2]])
            } else {
                None
            }
        })
        .collect();

    if vertices.is_empty() || indices.is_empty() {
        return false;
    }

    let (position, scale) = decompose(&global_transform);

    let scaled_vertices: Vec<Point<Real>> = vertices
        .iter()
        .map(|vertex| Point::new(vertex.x * scale.x, vertex.y * scale.y, vertex.z * scale.z))
        .collect();

    picking_world
        .add_trimesh_collider(entity, scaled_vertices, indices, position)
        .is_some()
}

pub fn unregister_entity_from_picking(
    picking_world: &mut crate::resources::PickingWorld,
    entity: Entity,
) {
    picking_world.remove_entity(entity);
}

pub fn update_picking_transform(
    picking_world: &mut crate::resources::PickingWorld,
    transforms: &mut GlobalTransforms<'_, '_>,
    entity: Entity,
) {
    let mut global_transform = None;
    transforms.for_entity(entity, |(global,)| global_transform = Some(*global));
    let Some(global_transform) = global_transform else {
        return;
    };

    let (position, _) = decompose(&global_transform);
    picking_world.update_collider_position(entity, position);
}

pub fn register_entity_hierarchy_for_trimesh_picking(
    picking_world: &mut crate::resources::PickingWorld,
    mesh_cache: &nightshade_render::resources::mesh_cache::MeshCache,
    hierarchy: &nightshade::ecs::HierarchyIndex,
    meshes: &mut RenderMeshes<'_, '_>,
    transforms: &mut GlobalTransforms<'_, '_>,
    root_entity: Entity,
) {
    let mut entities_to_register = Vec::new();
    collect_mesh_entities(hierarchy, meshes, root_entity, &mut entities_to_register);

    for entity in entities_to_register {
        register_entity_for_trimesh_picking(picking_world, mesh_cache, meshes, transforms, entity);
    }
}

fn collect_mesh_entities(
    hierarchy: &nightshade::ecs::HierarchyIndex,
    meshes: &mut RenderMeshes<'_, '_>,
    entity: Entity,
    entities: &mut Vec<Entity>,
) {
    let mut has_mesh = false;
    meshes.for_entity(entity, |_| has_mesh = true);
    if has_mesh {
        entities.push(entity);
    }

    for child in hierarchy.children(entity).to_vec() {
        collect_mesh_entities(hierarchy, meshes, child, entities);
    }
}
