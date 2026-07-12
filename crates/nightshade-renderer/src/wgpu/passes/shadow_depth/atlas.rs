//! Single source of truth for spotlight and rectangular area-light shadow atlas
//! slot assignment. A frame-schedule system computes the assignment once per
//! frame into `world.resources.shadow_atlas`. The shadow depth pass renders the
//! atlas from that stored result, and the mesh light collection writes its
//! sampling buffer and shadow indices from the same result, so the rendered
//! slots and the sampled slots are always the same data.

use std::collections::HashMap;

use super::types::SpotlightShadowData;
use freecs::Entity;

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
    pub entity: Entity,
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
    pub entity_to_index: HashMap<Entity, i32>,
    pub default_slot_size: u32,
}

/// Atlas texture side length, in pixels.
pub fn shadow_atlas_size() -> u32 {
    4096
}
