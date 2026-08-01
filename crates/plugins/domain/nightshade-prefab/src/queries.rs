use nightshade::ecs::ChildOf;
use nightshade::ecs::system_param::Query;

pub type PrefabSources<'world, 'item> = Query<'world, (&'item crate::contract::PrefabSource,)>;

pub fn find_group_root(
    sources: &mut PrefabSources<'_, '_>,
    parents: &mut nightshade_scene::queries::ParentLinks<'_, '_>,
    entity: nightshade::ecs::Entity,
) -> nightshade::ecs::Entity {
    let mut current = entity;
    let mut highest_prefab_root = None;
    loop {
        let mut is_prefab_root = false;
        sources.for_entity(current, |_| is_prefab_root = true);
        if is_prefab_root {
            highest_prefab_root = Some(current);
        }
        let mut parent = None;
        parents.for_entity(current, |(ChildOf(link),)| parent = Some(*link));
        match parent {
            Some(parent) => current = parent,
            None => break,
        }
    }
    highest_prefab_root.unwrap_or(current)
}

pub fn prefab_bounds(
    prefab: &crate::contract::Prefab,
    meshes: &std::collections::HashMap<String, nightshade_render::data::geometry::Mesh>,
) -> Option<(nalgebra_glm::Vec3, nalgebra_glm::Vec3)> {
    let mut lowest = nalgebra_glm::Vec3::new(f32::MAX, f32::MAX, f32::MAX);
    let mut highest = nalgebra_glm::Vec3::new(f32::MIN, f32::MIN, f32::MIN);
    for node in &prefab.root_nodes {
        stretch_over(
            node,
            &nalgebra_glm::Mat4::identity(),
            meshes,
            &mut lowest,
            &mut highest,
        );
    }
    (lowest.x <= highest.x).then_some((lowest, highest))
}

pub fn prefab_reach(
    prefab: &crate::contract::Prefab,
    meshes: &std::collections::HashMap<String, nightshade_render::data::geometry::Mesh>,
) -> (nalgebra_glm::Vec3, f32) {
    let Some((lowest, highest)) = prefab_bounds(prefab, meshes) else {
        return (nalgebra_glm::Vec3::zeros(), 1.0);
    };
    let middle = (lowest + highest) * 0.5;
    let reach = nalgebra_glm::length(&(highest - middle)).max(1.0e-4);
    (middle, reach)
}

fn stretch_over(
    node: &crate::contract::PrefabNode,
    above: &nalgebra_glm::Mat4,
    meshes: &std::collections::HashMap<String, nightshade_render::data::geometry::Mesh>,
    lowest: &mut nalgebra_glm::Vec3,
    highest: &mut nalgebra_glm::Vec3,
) {
    let here = above * node.local_transform.as_matrix();
    if let Some(render_mesh) = node.components.render_mesh.as_ref()
        && let Some(mesh) = meshes.get(&render_mesh.name)
    {
        for vertex in &mesh.vertices {
            let stood = here
                * nalgebra_glm::vec4(
                    vertex.position[0],
                    vertex.position[1],
                    vertex.position[2],
                    1.0,
                );
            let stood = nalgebra_glm::vec3(stood.x, stood.y, stood.z);
            *lowest = nalgebra_glm::min2(lowest, &stood);
            *highest = nalgebra_glm::max2(highest, &stood);
        }
    }
    for child in &node.children {
        stretch_over(child, &here, meshes, lowest, highest);
    }
}
