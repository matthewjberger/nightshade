use crate::components::{Body, Heading};
use crate::resources::Hull;
use nightshade::ecs::{HierarchyIndex, Storage};
use nightshade::prelude::*;
use nightshade_render::data::bounding_volume::BoundingVolume;
use nightshade_scene::prelude::LocalTransform;

pub fn measure(hierarchy: Res<HierarchyIndex>, storage: &mut Storage) {
    let roots: Vec<Entity> = storage
        .query_ref::<(&Heading,)>()
        .without::<Body>()
        .iter()
        .map(|(entity, _)| entity)
        .collect();
    let mut bodies = Vec::new();
    for root in roots {
        let mut minimum = Vec2::new(f32::MAX, f32::MAX);
        let mut maximum = Vec2::new(f32::MIN, f32::MIN);
        let mut found = false;
        let mut stack: Vec<(Entity, Mat4)> = hierarchy
            .children(root)
            .iter()
            .map(|child| (*child, Mat4::identity()))
            .collect();
        while let Some((node, above)) = stack.pop() {
            let Some(local) = storage.get::<LocalTransform>(node) else {
                continue;
            };
            let into_root = above * local.as_matrix();
            for child in hierarchy.children(node) {
                stack.push((*child, into_root));
            }
            let Some(volume) = storage.get::<BoundingVolume>(node) else {
                continue;
            };
            let boxed = volume.obb.transform(&into_root);
            for corner in boxed.get_corners() {
                minimum = Vec2::new(minimum.x.min(corner.x), minimum.y.min(corner.z));
                maximum = Vec2::new(maximum.x.max(corner.x), maximum.y.max(corner.z));
                found = true;
            }
        }
        if found {
            bodies.push((
                root,
                Hull {
                    center: (minimum + maximum) * 0.5,
                    heading: Vec2::new(0.0, 1.0),
                    half: (maximum - minimum) * 0.5,
                },
            ));
        }
    }
    for (root, hull) in bodies {
        storage.set(root, Body(hull));
    }
}
