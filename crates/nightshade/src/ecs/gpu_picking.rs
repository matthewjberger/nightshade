//! GPU-based picking for precise world position and normal sampling.
//!
//! Read back depth buffer data to get exact 3D world positions from screen coordinates.
//! This is used for features like placing decals at cursor position or terrain painting.
//!
//! - [`GpuPicking`]: Resource managing pick requests and results
//! - [`GpuPickRequest`]: Screen coordinates to sample
//! - [`GpuPickResult`]: World position, normal, depth, and entity ID
//!
//! Unlike ray-based picking, GPU picking reads the actual rendered depth buffer,
//! giving pixel-perfect results that match what's visible on screen.
//!
//! # Basic Usage
//!
//! ```ignore
//! fn on_mouse_input(&mut self, world: &mut World, button: MouseButton, state: KeyState) {
//!     if button == MouseButton::Left && state == KeyState::Pressed {
//!         let mouse = world.resources.input.mouse.position;
//!         world.resources.gpu_picking.request_pick(mouse.x as u32, mouse.y as u32);
//!     }
//! }
//!
//! fn run_systems(&mut self, world: &mut World) {
//!     // Check for pick result (available next frame after request)
//!     if let Some(result) = world.resources.gpu_picking.take_result() {
//!         let hit_position = result.world_position;
//!         let surface_normal = result.world_normal;
//!         let depth = result.depth;
//!
//!         // Place a decal at the hit position
//!         spawn_decal_at(world, hit_position, surface_normal);
//!
//!         // Or check which entity was clicked
//!         if let Some(entity_id) = result.entity_id {
//!             println!("Clicked entity ID: {}", entity_id);
//!         }
//!     }
//! }
//! ```
//!
//! # Continuous Picking (Hover/Drag)
//!
//! For real-time cursor tracking:
//!
//! ```ignore
//! fn run_systems(&mut self, world: &mut World) {
//!     let mouse = world.resources.input.mouse.position;
//!
//!     // Request pick every frame for continuous tracking
//!     world.resources.gpu_picking.request_pick(mouse.x as u32, mouse.y as u32);
//!
//!     // Results are smoothed automatically to reduce jitter
//!     if let Some(result) = world.resources.gpu_picking.take_result() {
//!         // Update preview decal position
//!         self.preview_position = result.world_position;
//!         self.preview_normal = result.world_normal;
//!     }
//! }
//! ```
//!
//! # GpuPickResult Fields
//!
//! | Field | Type | Description |
//! |-------|------|-------------|
//! | `world_position` | Vec3 | 3D world coordinates at cursor |
//! | `world_normal` | Vec3 | Surface normal (computed from depth gradients) |
//! | `depth` | f32 | Raw depth buffer value (0-1, reverse-Z) |
//! | `entity_id` | `Option<u32>` | Entity ID if entity ID buffer is enabled |
//!
//! # How It Works
//!
//! 1. Call `request_pick(x, y)` with screen coordinates
//! 2. Renderer samples a small region of the depth buffer around that point
//! 3. `compute_result()` reconstructs world position from depth + inverse view-projection
//! 4. Normal is computed from neighboring depth samples
//! 5. Results are smoothed across frames to reduce jitter
//!
//! # Performance Notes
//!
//! - Only one pick request is processed per frame
//! - Depth buffer readback has ~1 frame latency
//! - Position/normal smoothing reduces noise from depth precision limits

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
