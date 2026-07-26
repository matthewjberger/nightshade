use crate::ecs::world::World;
use crate::render::generational_registry::registry_entry_by_name;
use nightshade_ecs::Entity;
use rapier3d::prelude::*;

pub fn register_entity_for_trimesh_picking(world: &mut World, entity: Entity) -> bool {
    let render_mesh = match world.get::<crate::ecs::mesh::components::RenderMesh>(entity) {
        Some(rm) => rm.clone(),
        None => return false,
    };

    let mesh = match registry_entry_by_name(
        &world.res::<crate::render::mesh_cache::MeshCache>().registry,
        &render_mesh.name,
    ) {
        Some(m) => m,
        None => return false,
    };

    let global_transform =
        match world.get::<crate::ecs::transform::components::GlobalTransform>(entity) {
            Some(gt) => *gt,
            None => return false,
        };

    let vertices: Vec<Point<Real>> = mesh
        .vertices
        .iter()
        .map(|v| Point::new(v.position[0], v.position[1], v.position[2]))
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

    let translation = global_transform.0.column(3).xyz();
    let rotation_matrix = global_transform.0.fixed_view::<3, 3>(0, 0);
    let scale = nalgebra_glm::vec3(
        rotation_matrix.column(0).magnitude(),
        rotation_matrix.column(1).magnitude(),
        rotation_matrix.column(2).magnitude(),
    );

    let scaled_vertices: Vec<Point<Real>> = vertices
        .iter()
        .map(|v| Point::new(v.x * scale.x, v.y * scale.y, v.z * scale.z))
        .collect();

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

    world
        .plugin_resource_mut::<crate::plugins::physics::picking::resources::PickingWorld>()
        .add_trimesh_collider(entity, scaled_vertices, indices, position)
        .is_some()
}

pub fn unregister_entity_from_picking(world: &mut World, entity: Entity) {
    world
        .plugin_resource_mut::<crate::plugins::physics::picking::resources::PickingWorld>()
        .remove_entity(entity);
}

pub fn update_picking_transform(world: &mut World, entity: Entity) {
    let global_transform =
        match world.get::<crate::ecs::transform::components::GlobalTransform>(entity) {
            Some(gt) => *gt,
            None => return,
        };

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

    world
        .plugin_resource_mut::<crate::plugins::physics::picking::resources::PickingWorld>()
        .update_collider_position(entity, position);
}

pub fn register_entity_hierarchy_for_trimesh_picking(world: &mut World, root_entity: Entity) {
    let mut entities_to_register = Vec::new();
    collect_mesh_entities(world, root_entity, &mut entities_to_register);

    for entity in entities_to_register {
        register_entity_for_trimesh_picking(world, entity);
    }
}

fn collect_mesh_entities(world: &World, entity: Entity, entities: &mut Vec<Entity>) {
    if world
        .get::<crate::ecs::mesh::components::RenderMesh>(entity)
        .is_some()
    {
        entities.push(entity);
    }

    for child in world
        .res::<nightshade_ecs::dynamic::HierarchyIndex>()
        .children(entity)
        .to_vec()
    {
        collect_mesh_entities(world, child, entities);
    }
}
