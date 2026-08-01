use nightshade::prelude::*;
use nightshade_camera::components::Camera;
use nightshade_camera_controller::contract::PanOrbitCamera;
use nightshade_graphics::prelude::{Light, LightType};
use nightshade_render::data::bounding_volume::BoundingVolume;
use nightshade_scene::contract::GlobalTransform;

use crate::components::Deleted;
use crate::data::REACH;
use crate::resources::ViewerResources;

pub fn selection_bounds(
    viewer: &ViewerResources,
    hierarchy: &nightshade::ecs::HierarchyIndex,
    storage: &nightshade::ecs::Storage,
    selected: Entity,
) -> Option<(Vec3, Vec3)> {
    let mut min = Vec3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY);
    let mut max = Vec3::new(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY);
    let mut found = false;
    let mut roots = vec![selected];
    roots.extend(viewer.selection.others.iter().copied());

    for root in roots {
        let mut entities = hierarchy.descendants(root);
        entities.push(root);
        for entity in entities {
            if storage.has::<Deleted>(entity) {
                continue;
            }
            match box_corners(storage, entity) {
                Some(corners) => {
                    for corner in corners {
                        min = nalgebra_glm::min2(&min, &corner);
                        max = nalgebra_glm::max2(&max, &corner);
                    }
                    found = true;
                }
                None => {
                    if let Some(global) = storage.get::<GlobalTransform>(entity) {
                        let position = global.translation();
                        let reach = marker_reach(storage, entity);
                        let offset = Vec3::new(reach, reach, reach);
                        min = nalgebra_glm::min2(&min, &(position - offset));
                        max = nalgebra_glm::max2(&max, &(position + offset));
                        found = true;
                    }
                }
            }
        }
    }
    found.then_some((min, max))
}

fn marker_reach(storage: &nightshade::ecs::Storage, entity: Entity) -> f32 {
    if let Some(light) = storage.get::<Light>(entity) {
        return match light.light_type {
            LightType::Directional => REACH,
            _ => light.range.max(0.5),
        };
    }
    if storage.has::<Camera>(entity) {
        return 1.0;
    }
    0.0
}

pub fn model_bounds(
    viewer: &ViewerResources,
    storage: &nightshade::ecs::Storage,
) -> Option<(Vec3, Vec3)> {
    let mut min = Vec3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY);
    let mut max = Vec3::new(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY);
    let mut found = false;

    for &entity in &viewer.model.entities {
        if storage.has::<Deleted>(entity) {
            continue;
        }
        let Some(global) = storage.get::<GlobalTransform>(entity) else {
            continue;
        };
        let position = global.translation();
        let world_scale = global.0.column(0).xyz().norm();
        let radius = storage
            .get::<BoundingVolume>(entity)
            .map(|volume| volume.sphere_radius * world_scale)
            .unwrap_or(0.0);
        let offset = Vec3::new(radius, radius, radius);
        min = nalgebra_glm::min2(&min, &(position - offset));
        max = nalgebra_glm::max2(&max, &(position + offset));
        found = true;
    }

    found.then_some((min, max))
}

pub fn box_corners(storage: &nightshade::ecs::Storage, entity: Entity) -> Option<[Vec3; 8]> {
    let global = storage.get::<GlobalTransform>(entity)?.0;
    let volume = storage.get::<BoundingVolume>(entity)?;
    let center = volume.obb.center;
    let half = volume.obb.half_extents;
    let orientation = volume.obb.orientation;
    let mut corners = [Vec3::zeros(); 8];
    for (index, corner) in corners.iter_mut().enumerate() {
        let sign = Vec3::new(
            if index & 1 == 0 { -1.0 } else { 1.0 },
            if index & 2 == 0 { -1.0 } else { 1.0 },
            if index & 4 == 0 { -1.0 } else { 1.0 },
        );
        let local =
            center + nalgebra_glm::quat_rotate_vec3(&orientation, &sign.component_mul(&half));
        let moved = global * nalgebra_glm::vec4(local.x, local.y, local.z, 1.0);
        *corner = Vec3::new(moved.x, moved.y, moved.z);
    }
    Some(corners)
}

pub fn orbit_focus(camera: Option<Entity>, storage: &nightshade::ecs::Storage) -> Vec3 {
    camera
        .and_then(|camera| storage.get::<PanOrbitCamera>(camera))
        .map(|orbit| orbit.target_focus)
        .unwrap_or_else(Vec3::zeros)
}
