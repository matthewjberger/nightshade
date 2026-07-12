//! GPU-based picking for precise world position and normal sampling.
//!
//! Read back depth buffer data to get exact 3D world positions from screen coordinates.
//! This is used for features like placing decals at cursor position or terrain painting.
//!
//! - [`GpuPicking`]: Resource managing pick requests and results
//! - [`GpuPickRequest`]: Screen coordinates to sample
//! - [`GpuPickResult`]: World position, normal, depth, and entity id

use crate::ecs::world::Vec2;
use crate::render::config::ViewportRect;

#[derive(Debug, Clone, Copy)]
pub struct GpuPickRequest {
    pub screen_x: u32,
    pub screen_y: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct GpuPickResult {
    pub world_position: nalgebra_glm::Vec3,
    pub world_normal: nalgebra_glm::Vec3,
    pub depth: f32,
    pub entity_id: Option<u32>,
}

#[derive(Default)]
pub struct GpuPicking {
    pub pending_request: Option<GpuPickRequest>,
    pub result: Option<GpuPickResult>,
    pub depth_sample_buffer: Vec<f32>,
    pub entity_id_sample_buffer: Vec<u32>,
    pub sample_width: u32,
    pub sample_height: u32,
    pub sample_center_x: u32,
    pub sample_center_y: u32,
    previous_position: Option<nalgebra_glm::Vec3>,
    previous_normal: Option<nalgebra_glm::Vec3>,
}

impl GpuPicking {
    pub fn request_pick(&mut self, screen_x: u32, screen_y: u32) {
        self.pending_request = Some(GpuPickRequest { screen_x, screen_y });
        self.result = None;
    }

    pub fn take_result(&mut self) -> Option<GpuPickResult> {
        self.result.take()
    }

    pub fn has_pending_request(&self) -> bool {
        self.pending_request.is_some()
    }

    pub fn take_pending_request(&mut self) -> Option<GpuPickRequest> {
        self.pending_request.take()
    }

    pub fn set_depth_samples(
        &mut self,
        depth_samples: Vec<f32>,
        entity_id_samples: Vec<u32>,
        width: u32,
        height: u32,
        center_x: u32,
        center_y: u32,
    ) {
        self.depth_sample_buffer = depth_samples;
        self.entity_id_sample_buffer = entity_id_samples;
        self.sample_width = width;
        self.sample_height = height;
        self.sample_center_x = center_x;
        self.sample_center_y = center_y;
    }

    pub fn compute_result(
        &mut self,
        inverse_view_proj: &nalgebra_glm::Mat4,
        viewport_width: f32,
        viewport_height: f32,
    ) {
        if self.depth_sample_buffer.is_empty() || self.sample_width == 0 || self.sample_height == 0
        {
            return;
        }

        let center_index =
            ((self.sample_height / 2) * self.sample_width + (self.sample_width / 2)) as usize;
        if center_index >= self.depth_sample_buffer.len() {
            return;
        }

        let center_depth = self.depth_sample_buffer[center_index];
        if center_depth <= 0.0 {
            self.result = Some(GpuPickResult {
                world_position: nalgebra_glm::Vec3::zeros(),
                world_normal: nalgebra_glm::Vec3::new(0.0, 1.0, 0.0),
                depth: 0.0,
                entity_id: None,
            });
            self.previous_position = None;
            self.previous_normal = None;
            return;
        }

        let ndc_x = (2.0 * self.sample_center_x as f32) / viewport_width - 1.0;
        let ndc_y = 1.0 - (2.0 * self.sample_center_y as f32) / viewport_height;

        let clip_pos = nalgebra_glm::Vec4::new(ndc_x, ndc_y, center_depth, 1.0);
        let world_pos = inverse_view_proj * clip_pos;
        let world_position = world_pos.xyz() / world_pos.w;

        let mut normal = nalgebra_glm::Vec3::new(0.0, 1.0, 0.0);

        if self.sample_width >= 3 && self.sample_height >= 3 {
            let get_world_pos = |dx: i32, dy: i32| -> Option<nalgebra_glm::Vec3> {
                let sx = (self.sample_width as i32 / 2 + dx) as usize;
                let sy = (self.sample_height as i32 / 2 + dy) as usize;
                let idx = sy * self.sample_width as usize + sx;
                if idx >= self.depth_sample_buffer.len() {
                    return None;
                }
                let d = self.depth_sample_buffer[idx];
                if d <= 0.0 {
                    return None;
                }

                let px = self.sample_center_x as i32 + dx;
                let py = self.sample_center_y as i32 + dy;
                let nx = (2.0 * px as f32) / viewport_width - 1.0;
                let ny = 1.0 - (2.0 * py as f32) / viewport_height;
                let clip = nalgebra_glm::Vec4::new(nx, ny, d, 1.0);
                let wp = inverse_view_proj * clip;
                Some(wp.xyz() / wp.w)
            };

            if let (Some(left), Some(right), Some(up), Some(down)) = (
                get_world_pos(-1, 0),
                get_world_pos(1, 0),
                get_world_pos(0, -1),
                get_world_pos(0, 1),
            ) {
                let dx = right - left;
                let dy = down - up;
                let n = nalgebra_glm::cross(&dx, &dy);
                let len = nalgebra_glm::length(&n);
                if len > 1e-6 {
                    normal = n / len;
                }
            }
        }

        let smoothing_factor = 0.3;

        let smoothed_position = if let Some(prev_pos) = self.previous_position {
            let distance = nalgebra_glm::distance(&prev_pos, &world_position);
            if distance < 2.0 {
                nalgebra_glm::lerp(&prev_pos, &world_position, smoothing_factor)
            } else {
                world_position
            }
        } else {
            world_position
        };

        let smoothed_normal = if let Some(prev_normal) = self.previous_normal {
            let dot = nalgebra_glm::dot(&prev_normal, &normal);
            if dot > 0.0 {
                let lerped = nalgebra_glm::lerp(&prev_normal, &normal, smoothing_factor);
                nalgebra_glm::normalize(&lerped)
            } else {
                normal
            }
        } else {
            normal
        };

        self.previous_position = Some(smoothed_position);
        self.previous_normal = Some(smoothed_normal);

        let entity_id = if center_index < self.entity_id_sample_buffer.len() {
            let id = self.entity_id_sample_buffer[center_index];
            if id > 0 { Some(id) } else { None }
        } else {
            None
        };

        self.result = Some(GpuPickResult {
            world_position: smoothed_position,
            world_normal: smoothed_normal,
            depth: center_depth,
            entity_id,
        });
    }
}

/// Map a tile-local cursor position to surface (input texture) pixel coordinates,
/// honoring the same center-crop UV transform that `viewport_compose` applies when
/// the tile aspect differs from the surface aspect.
///
/// Returns `None` if the cursor is outside `viewport_rect`.
pub fn surface_pick_coords(
    mouse_pos: Vec2,
    viewport_rect: Option<ViewportRect>,
    surface_size: (u32, u32),
) -> Option<(u32, u32)> {
    let Some(rect) = viewport_rect else {
        return Some((mouse_pos.x.max(0.0) as u32, mouse_pos.y.max(0.0) as u32));
    };

    let local = rect.to_local(mouse_pos);
    if local.x < 0.0 || local.y < 0.0 || local.x >= rect.width || local.y >= rect.height {
        return None;
    }

    let surface_width = surface_size.0 as f32;
    let surface_height = surface_size.1 as f32;
    if surface_width <= 0.0 || surface_height <= 0.0 || rect.width <= 0.0 || rect.height <= 0.0 {
        return None;
    }

    let texture_aspect = surface_width / surface_height;
    let rect_aspect = rect.width / rect.height;
    let (uv_scale_x, uv_scale_y) = if texture_aspect > rect_aspect {
        (rect_aspect / texture_aspect, 1.0)
    } else if texture_aspect < rect_aspect {
        (1.0, texture_aspect / rect_aspect)
    } else {
        (1.0, 1.0)
    };

    let tile_uv_x = local.x / rect.width;
    let tile_uv_y = local.y / rect.height;
    let input_uv_x = (tile_uv_x - 0.5) * uv_scale_x + 0.5;
    let input_uv_y = (tile_uv_y - 0.5) * uv_scale_y + 0.5;

    let pick_x = (input_uv_x * surface_width)
        .clamp(0.0, surface_width - 1.0)
        .round() as u32;
    let pick_y = (input_uv_y * surface_height)
        .clamp(0.0, surface_height - 1.0)
        .round() as u32;
    Some((pick_x, pick_y))
}
