//! Single source of truth for spotlight and rectangular area-light shadow atlas
//! slot assignment. The frame driver computes the assignment once per frame
//! into `RenderInputs::shadow_atlas`. The shadow depth pass renders the
//! atlas from that stored result, and the mesh light collection writes its
//! sampling buffer and shadow indices from the same result, so the rendered
//! slots and the sampled slots are always the same data.

use std::collections::HashMap;

use super::cascade::reverse_z_perspective;
use super::types::{MAX_SPOTLIGHT_SHADOWS, SPOTLIGHT_ATLAS_SLOTS_PER_ROW, SpotlightShadowData};
use crate::entity::RenderEntity;
use crate::wgpu::passes::geometry::projection::{
    atlas_shadow_fov, light_casts_atlas_shadow, transform_forward,
};
use crate::wgpu::render_configs::CameraFrameInputs;

pub fn sphere_in_frustum(center: nalgebra_glm::Vec3, radius: f32, planes: &[[f32; 4]; 6]) -> bool {
    for plane in planes {
        let distance = plane[0] * center.x + plane[1] * center.y + plane[2] * center.z + plane[3];
        if distance < -radius {
            return false;
        }
    }
    true
}

pub fn full_frustum_planes(view_projection: &[[f32; 4]; 4]) -> [[f32; 4]; 6] {
    let row = |index: usize| {
        [
            view_projection[0][index],
            view_projection[1][index],
            view_projection[2][index],
            view_projection[3][index],
        ]
    };
    let row_x = row(0);
    let row_y = row(1);
    let row_z = row(2);
    let row_w = row(3);
    let combine = |sign: f32, axis: [f32; 4]| {
        let plane = [
            row_w[0] + sign * axis[0],
            row_w[1] + sign * axis[1],
            row_w[2] + sign * axis[2],
            row_w[3] + sign * axis[3],
        ];
        let length = (plane[0] * plane[0] + plane[1] * plane[1] + plane[2] * plane[2])
            .sqrt()
            .max(1e-8);
        [
            plane[0] / length,
            plane[1] / length,
            plane[2] / length,
            plane[3] / length,
        ]
    };
    [
        combine(1.0, row_x),
        combine(-1.0, row_x),
        combine(1.0, row_y),
        combine(-1.0, row_y),
        combine(1.0, row_z),
        combine(-1.0, row_z),
    ]
}

/// One assigned atlas slot. The shadow depth pass renders an occluder depth map
/// into this slot, and the mesh pass samples it through the matching
/// [`SpotlightShadowData`] entry.
pub struct SpotlightAtlasSlot {
    pub entity: RenderEntity,
    pub view_projection: nalgebra_glm::Mat4,
    pub atlas_x_pixels: u32,
    pub atlas_y_pixels: u32,
    pub atlas_size_pixels: u32,
    pub bias: f32,
    pub slot_index: u32,
}

#[derive(Default)]
pub struct SpotlightAtlasAssignment {
    pub slots: Vec<SpotlightAtlasSlot>,
    pub data: Vec<SpotlightShadowData>,
    pub entity_to_index: HashMap<RenderEntity, i32>,
    pub default_slot_size: u32,
}

/// Atlas texture side length, in pixels.
pub fn shadow_atlas_size() -> u32 {
    4096
}

/// Gathers shadow-casting spotlights and rectangular or disk area lights, culls
/// those whose lit volume falls outside the camera frustum or shadow distance,
/// sorts by distance, and shelf-packs them into the atlas. The camera frustum
/// cull only applies when a single camera drives the view, so tiled multi-camera
/// layouts keep every candidate.
pub fn assign_spotlight_atlas(
    scene: &crate::config::RendererState,
    active_camera: Option<&CameraFrameInputs>,
    use_camera_frustum: bool,
    atlas_size: u32,
) -> SpotlightAtlasAssignment {
    let camera_matrices = active_camera.and_then(|camera| camera.matrices);
    let camera_position = camera_matrices
        .map(|matrices| matrices.camera_position)
        .unwrap_or_else(|| nalgebra_glm::vec3(0.0, 0.0, 0.0));

    let camera_frustum = if use_camera_frustum {
        camera_matrices.map(|matrices| {
            let view_projection: [[f32; 4]; 4] = (matrices.projection * matrices.view).into();
            full_frustum_planes(&view_projection)
        })
    } else {
        None
    };

    let mut spotlight_candidates: Vec<(
        RenderEntity,
        f32,
        crate::config::RenderLightData,
        nalgebra_glm::Mat4,
    )> = Vec::new();

    for render_light in &scene.render_lights.lights {
        let entity = render_light.entity;
        let light = &render_light.light;
        let transform = &render_light.transform;
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
                && !sphere_in_frustum(light_pos, light.range.max(1.0), planes)
            {
                continue;
            }
            spotlight_candidates.push((entity, distance_sq, light.clone(), *transform));
        }
    }

    spotlight_candidates.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

    let default_slot_size = atlas_size / SPOTLIGHT_ATLAS_SLOTS_PER_ROW;
    let mut shelf_x: u32 = 0;
    let mut shelf_y: u32 = 0;
    let mut shelf_height: u32 = 0;

    let mut slots = Vec::new();
    let mut data = Vec::new();
    let mut entity_to_index = HashMap::new();

    for (slot_index, (entity, _distance, light, transform)) in
        (0_u32..).zip(spotlight_candidates.iter().take(MAX_SPOTLIGHT_SHADOWS))
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
            bias: light.shadow_bias,
            slot_index,
        });

        data.push(SpotlightShadowData {
            view_projection: view_projection.into(),
            atlas_offset: [atlas_offset_x, atlas_offset_y],
            atlas_scale: [atlas_scale, atlas_scale],
            bias: light.shadow_bias,
            _padding: [0.0; 3],
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
