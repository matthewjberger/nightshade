use crate::contract::CameraFrameInputs;
use crate::data::config::{RenderAreaLightShape, RenderLightType};
use crate::data::shadow::{SpotlightAtlasSlot, SpotlightShadowData};
use crate::queries::projection::{reverse_z_perspective, sphere_in_frustum, transform_forward};
use crate::resources::shadow::{PointShadowSlot, SpotlightAtlasAssignment};
use nightshade::ecs::Entity;
use std::collections::HashMap;

pub fn assign_spotlight_atlas(
    scene_world: &nightshade::ecs::World,
    active_camera: Option<&CameraFrameInputs>,
    use_camera_frustum: bool,
    atlas_size: u32,
    slots_per_row: u32,
    max_shadows: usize,
) -> SpotlightAtlasAssignment {
    let camera_matrices = active_camera.and_then(|camera| camera.matrices);
    let camera_position = camera_matrices
        .map(|matrices| matrices.camera_position)
        .unwrap_or_else(|| nalgebra_glm::vec3(0.0, 0.0, 0.0));

    let camera_frustum = if use_camera_frustum {
        camera_matrices.map(|matrices| {
            let view_projection: [[f32; 4]; 4] = (matrices.projection * matrices.view).into();
            crate::queries::projection::full_frustum_planes(&view_projection)
        })
    } else {
        None
    };

    let mut spotlight_candidates: Vec<(
        Entity,
        f32,
        crate::components::light::RenderLightData,
        nalgebra_glm::Mat4,
    )> = Vec::new();

    let lights: Vec<(
        Entity,
        crate::components::light::RenderLightData,
        nalgebra_glm::Mat4,
    )> = scene_world
        .query_ref::<(
            &crate::components::light::RenderLightData,
            &crate::components::render_world::Transform,
        )>()
        .iter()
        .map(|(entity, (light, transform))| (entity, light.clone(), transform.0))
        .collect();
    for (entity, light, transform) in &lights {
        let entity = *entity;
        if light.cast_shadows && light_casts_atlas_shadow(light) {
            let light_pos =
                nalgebra_glm::vec3(transform[(0, 3)], transform[(1, 3)], transform[(2, 3)]);
            let distance_sq = nalgebra_glm::length2(&(light_pos - camera_position));
            if light.shadow_distance > 0.0
                && distance_sq > light.shadow_distance * light.shadow_distance
            {
                continue;
            }
            if let Some(planes) = &camera_frustum
                && !sphere_in_frustum(&light_pos, light.range.max(1.0), planes)
            {
                continue;
            }
            spotlight_candidates.push((entity, distance_sq, light.clone(), *transform));
        }
    }

    spotlight_candidates.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

    let default_slot_size = atlas_size / slots_per_row;
    let mut shelf_x: u32 = 0;
    let mut shelf_y: u32 = 0;
    let mut shelf_height: u32 = 0;

    let mut slots = Vec::new();
    let mut data = Vec::new();
    let mut entity_to_index = HashMap::new();

    for (slot_index, (entity, _distance, light, transform)) in
        (0_u32..).zip(spotlight_candidates.iter().take(max_shadows))
    {
        let requested = if light.shadow_resolution > 0 {
            light.shadow_resolution.clamp(64, atlas_size)
        } else {
            default_slot_size
        };

        if shelf_x + requested > atlas_size {
            shelf_y += shelf_height;
            shelf_x = 0;
            shelf_height = 0;
        }
        if shelf_y + requested > atlas_size {
            break;
        }

        let light_position =
            nalgebra_glm::vec3(transform[(0, 3)], transform[(1, 3)], transform[(2, 3)]);
        let light_direction = transform_forward(transform);

        let up = if light_direction.y.abs() > 0.99 {
            nalgebra_glm::vec3(1.0, 0.0, 0.0)
        } else {
            nalgebra_glm::vec3(0.0, 1.0, 0.0)
        };

        let target = light_position + light_direction;
        let light_view = nalgebra_glm::look_at(&light_position, &target, &up);

        let fov = atlas_shadow_fov(light);
        let near = 0.1;
        let far = light.range.max(1.0);
        let light_projection = reverse_z_perspective(fov, 1.0, near, far);

        let view_projection = light_projection * light_view;

        let atlas_offset_x = shelf_x as f32 / atlas_size as f32;
        let atlas_offset_y = shelf_y as f32 / atlas_size as f32;
        let atlas_scale = requested as f32 / atlas_size as f32;

        entity_to_index.insert(*entity, slot_index as i32);

        slots.push(SpotlightAtlasSlot {
            entity: *entity,
            view_projection,
            atlas_x_pixels: shelf_x,
            atlas_y_pixels: shelf_y,
            atlas_size_pixels: requested,
            slot_index,
        });

        data.push(SpotlightShadowData {
            view_projection: view_projection.into(),
            atlas_offset: [atlas_offset_x, atlas_offset_y],
            atlas_scale: [atlas_scale, atlas_scale],
            bias: light.shadow_bias,
            padding: [0.0; 3],
        });

        shelf_x += requested;
        shelf_height = shelf_height.max(requested);
    }

    SpotlightAtlasAssignment {
        slots,
        data,
        entity_to_index,
        default_slot_size,
    }
}

pub fn assign_point_shadows(
    scene_world: &nightshade::ecs::World,
    camera_position: nalgebra_glm::Vec3,
    camera_frustum: Option<&[nalgebra_glm::Vec4; 6]>,
    slot_count: usize,
) -> Vec<PointShadowSlot> {
    let mut candidates: Vec<(
        Entity,
        f32,
        crate::components::light::RenderLightData,
        nalgebra_glm::Mat4,
    )> = Vec::new();
    let lights: Vec<(
        Entity,
        crate::components::light::RenderLightData,
        nalgebra_glm::Mat4,
    )> = scene_world
        .query_ref::<(
            &crate::components::light::RenderLightData,
            &crate::components::render_world::Transform,
        )>()
        .iter()
        .map(|(entity, (light, transform))| (entity, light.clone(), transform.0))
        .collect();
    for (entity, light, transform) in &lights {
        if !matches!(light.light_type, RenderLightType::Point) || !light.cast_shadows {
            continue;
        }
        let light_position =
            nalgebra_glm::vec3(transform[(0, 3)], transform[(1, 3)], transform[(2, 3)]);
        let distance_squared = nalgebra_glm::length2(&(light_position - camera_position));
        if light.shadow_distance > 0.0
            && distance_squared > light.shadow_distance * light.shadow_distance
        {
            continue;
        }
        if let Some(planes) = camera_frustum
            && !sphere_in_frustum(&light_position, light.range.max(1.0), planes)
        {
            continue;
        }
        candidates.push((*entity, distance_squared, light.clone(), *transform));
    }
    candidates.sort_by(|first, second| {
        first
            .1
            .partial_cmp(&second.1)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    candidates
        .into_iter()
        .take(slot_count)
        .enumerate()
        .map(
            |(slot_index, (entity, _, light, transform))| PointShadowSlot {
                entity,
                light,
                transform,
                slot_index: slot_index as u32,
            },
        )
        .collect()
}

pub fn light_casts_atlas_shadow(light: &crate::components::light::RenderLightData) -> bool {
    match light.light_type {
        RenderLightType::Spot => true,
        RenderLightType::Area => matches!(
            light.area_shape,
            RenderAreaLightShape::Rectangle | RenderAreaLightShape::Disk
        ),
        _ => false,
    }
}

pub fn atlas_shadow_fov(light: &crate::components::light::RenderLightData) -> f32 {
    match light.light_type {
        RenderLightType::Area => 2.4,
        _ => light.outer_cone_angle * 2.0,
    }
}
