use crate::contract::{ChildOf, GlobalTransform, LocalTransform};
use nightshade_ecs::Entity;
use nightshade_ecs::system_param::Query;

pub type ParentLinks<'world, 'item> = Query<'world, (&'item nightshade_ecs::ChildOf,)>;

pub type GlobalTransforms<'world, 'item> = Query<'world, (&'item GlobalTransform,)>;

pub type Names<'world, 'item> = Query<'world, (&'item crate::contract::primitives::Name,)>;

pub fn find_entity_by_name(names: &mut Names<'_, '_>, name: &str) -> Option<Entity> {
    let mut found = None;
    names.for_each_mut(|entity, (entity_name,)| {
        if found.is_none() && entity_name.0 == name {
            found = Some(entity);
        }
    });
    found
}

pub fn query_children(
    hierarchy: &nightshade_ecs::HierarchyIndex,
    entity: nightshade_ecs::Entity,
) -> Vec<nightshade_ecs::Entity> {
    hierarchy.children(entity).to_vec()
}

pub fn query_descendants(
    hierarchy: &nightshade_ecs::HierarchyIndex,
    entity: nightshade_ecs::Entity,
) -> Vec<nightshade_ecs::Entity> {
    hierarchy.descendants(entity)
}

pub fn query_ancestors(
    parents: &mut ParentLinks<'_, '_>,
    entity: nightshade_ecs::Entity,
) -> Vec<nightshade_ecs::Entity> {
    let mut chain = vec![entity];
    let mut current = entity;
    loop {
        let mut parent = None;
        parents.for_entity(current, |(link,)| parent = Some(link.0));
        let Some(parent) = parent else {
            break;
        };
        if chain.contains(&parent) {
            break;
        }
        chain.push(parent);
        current = parent;
    }
    chain
}

pub fn is_ancestor_of(
    parents: &mut ParentLinks<'_, '_>,
    ancestor: nightshade_ecs::Entity,
    entity: nightshade_ecs::Entity,
) -> bool {
    query_ancestors(parents, entity)
        .into_iter()
        .skip(1)
        .any(|above| above == ancestor)
}

pub fn topmost(
    parents: &mut ParentLinks<'_, '_>,
    targets: &[nightshade_ecs::Entity],
) -> Vec<nightshade_ecs::Entity> {
    let named: std::collections::HashSet<nightshade_ecs::Entity> =
        targets.iter().copied().collect();
    targets
        .iter()
        .copied()
        .filter(|entity| {
            !query_ancestors(parents, *entity)
                .into_iter()
                .skip(1)
                .any(|above| named.contains(&above))
        })
        .collect()
}

pub fn local_transform_under(
    transforms: &mut GlobalTransforms<'_, '_>,
    parent: nightshade_ecs::Entity,
    child: nightshade_ecs::Entity,
) -> Option<LocalTransform> {
    let mut parent_world = None;
    transforms.for_entity(parent, |(global,)| parent_world = Some(global.0));
    let mut child_world = None;
    transforms.for_entity(child, |(global,)| child_world = Some(global.0));
    let inverse = parent_world?.try_inverse()?;
    Some(LocalTransform::from_matrix(&(inverse * child_world?)))
}

pub fn parent_world_matrix(
    parents: &mut ParentLinks<'_, '_>,
    transforms: &mut GlobalTransforms<'_, '_>,
    entity: nightshade_ecs::Entity,
) -> nalgebra_glm::Mat4 {
    let mut parent = None;
    parents.for_entity(entity, |(link,)| parent = Some(link.0));
    let Some(parent) = parent else {
        return nalgebra_glm::Mat4::identity();
    };
    let mut matrix = nalgebra_glm::Mat4::identity();
    transforms.for_entity(parent, |(global,)| matrix = global.0);
    matrix
}

pub fn chain_from_root_to_leaf(
    parents: &mut ParentLinks<'_, '_>,
    root: nightshade_ecs::Entity,
    leaf: nightshade_ecs::Entity,
) -> Vec<nightshade_ecs::Entity> {
    if leaf == root {
        return vec![root];
    }
    let mut chain = vec![leaf];
    let mut current = leaf;
    loop {
        let mut parent = None;
        parents.for_entity(current, |(ChildOf(link),)| parent = Some(*link));
        let Some(parent) = parent else {
            break;
        };
        chain.push(parent);
        if parent == root {
            break;
        }
        current = parent;
    }
    chain.reverse();
    chain
}

pub(crate) fn computed_global_transform(
    world: &nightshade_ecs::Storage,
    current: Entity,
) -> nalgebra_glm::Mat4 {
    let local_matrix = match world.get::<crate::contract::LocalTransform>(current) {
        Some(local_transform) => local_transform.as_matrix(),
        None => nalgebra_glm::Mat4::identity(),
    };

    match world
        .get::<nightshade_ecs::ChildOf>(current)
        .map(|child_of| child_of.0)
    {
        Some(parent_entity) => {
            let mut parent_matrix = world
                .get::<crate::contract::GlobalTransform>(parent_entity)
                .map(|global_transform| global_transform.0)
                .unwrap_or_else(nalgebra_glm::Mat4::identity);
            if world.has::<crate::contract::IgnoreParentScale>(current) {
                parent_matrix = remove_scale_from_matrix(parent_matrix);
            }
            parent_matrix * local_matrix
        }
        None => local_matrix,
    }
}

fn remove_scale_from_matrix(matrix: nalgebra_glm::Mat4) -> nalgebra_glm::Mat4 {
    let mut result = matrix;

    let right = nalgebra_glm::vec3(matrix[(0, 0)], matrix[(1, 0)], matrix[(2, 0)]);
    let up = nalgebra_glm::vec3(matrix[(0, 1)], matrix[(1, 1)], matrix[(2, 1)]);
    let forward = nalgebra_glm::vec3(matrix[(0, 2)], matrix[(1, 2)], matrix[(2, 2)]);

    let right_normalized = nalgebra_glm::normalize(&right);
    let up_normalized = nalgebra_glm::normalize(&up);
    let forward_normalized = nalgebra_glm::normalize(&forward);

    result[(0, 0)] = right_normalized.x;
    result[(1, 0)] = right_normalized.y;
    result[(2, 0)] = right_normalized.z;

    result[(0, 1)] = up_normalized.x;
    result[(1, 1)] = up_normalized.y;
    result[(2, 1)] = up_normalized.z;

    result[(0, 2)] = forward_normalized.x;
    result[(1, 2)] = forward_normalized.y;
    result[(2, 2)] = forward_normalized.z;

    result
}
