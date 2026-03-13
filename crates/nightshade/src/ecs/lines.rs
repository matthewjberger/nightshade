//! Debug line rendering for visualization and gizmos.
//!
//! Draw lines in 3D space for debugging, visualization, and editor gizmos:
//!
//! - [`Lines`]: Component holding a collection of lines to render
//! - [`Line`]: Single line segment with start, end, and color
//!
//! Lines are rendered with depth testing and support up to 1 million segments.
//!
//! # Basic Line Drawing
//!
//! Add lines to an entity's `Lines` component:
//!
//! ```ignore
//! use nightshade::ecs::lines::{Lines, Line};
//!
//! // Create a lines entity
//! let debug_lines = world.spawn_entities(LINES, 1)[0];
//!
//! // Get the lines component and add lines
//! if let Some(lines) = world.core.get_lines_mut(debug_lines) {
//!     lines.push(Line {
//!         start: Vec3::new(0.0, 0.0, 0.0),
//!         end: Vec3::new(1.0, 1.0, 1.0),
//!         color: Vec4::new(1.0, 0.0, 0.0, 1.0),  // Red
//!     });
//! }
//! ```
//!
//! # Drawing Shapes
//!
//! Common debug shapes can be drawn with helper patterns:
//!
//! ```ignore
//! fn draw_debug_shapes(world: &mut World, lines_entity: Entity) {
//!     let Some(lines) = world.core.get_lines_mut(lines_entity) else { return };
//!
//!     // Draw a box/cube wireframe
//!     let min = Vec3::new(-1.0, 0.0, -1.0);
//!     let max = Vec3::new(1.0, 2.0, 1.0);
//!     let color = Vec4::new(0.0, 1.0, 0.0, 1.0);  // Green
//!
//!     // Bottom face
//!     lines.push(Line { start: vec3(min.x, min.y, min.z), end: vec3(max.x, min.y, min.z), color });
//!     lines.push(Line { start: vec3(max.x, min.y, min.z), end: vec3(max.x, min.y, max.z), color });
//!     lines.push(Line { start: vec3(max.x, min.y, max.z), end: vec3(min.x, min.y, max.z), color });
//!     lines.push(Line { start: vec3(min.x, min.y, max.z), end: vec3(min.x, min.y, min.z), color });
//!
//!     // Top face
//!     lines.push(Line { start: vec3(min.x, max.y, min.z), end: vec3(max.x, max.y, min.z), color });
//!     lines.push(Line { start: vec3(max.x, max.y, min.z), end: vec3(max.x, max.y, max.z), color });
//!     lines.push(Line { start: vec3(max.x, max.y, max.z), end: vec3(min.x, max.y, max.z), color });
//!     lines.push(Line { start: vec3(min.x, max.y, max.z), end: vec3(min.x, max.y, min.z), color });
//!
//!     // Vertical edges
//!     lines.push(Line { start: vec3(min.x, min.y, min.z), end: vec3(min.x, max.y, min.z), color });
//!     lines.push(Line { start: vec3(max.x, min.y, min.z), end: vec3(max.x, max.y, min.z), color });
//!     lines.push(Line { start: vec3(max.x, min.y, max.z), end: vec3(max.x, max.y, max.z), color });
//!     lines.push(Line { start: vec3(min.x, min.y, max.z), end: vec3(min.x, max.y, max.z), color });
//!
//!     // Draw a circle (approximated with segments)
//!     let center = Vec3::new(0.0, 0.0, 0.0);
//!     let radius = 2.0;
//!     let segments = 32;
//!     let circle_color = Vec4::new(0.0, 0.0, 1.0, 1.0);  // Blue
//!
//!     for i in 0..segments {
//!         let angle1 = (i as f32 / segments as f32) * std::f32::consts::TAU;
//!         let angle2 = ((i + 1) as f32 / segments as f32) * std::f32::consts::TAU;
//!         lines.push(Line {
//!             start: center + Vec3::new(angle1.cos() * radius, 0.0, angle1.sin() * radius),
//!             end: center + Vec3::new(angle2.cos() * radius, 0.0, angle2.sin() * radius),
//!             color: circle_color,
//!         });
//!     }
//!
//!     // Draw coordinate axes
//!     lines.push(Line {
//!         start: Vec3::zeros(),
//!         end: Vec3::new(1.0, 0.0, 0.0),
//!         color: Vec4::new(1.0, 0.0, 0.0, 1.0),  // X = Red
//!     });
//!     lines.push(Line {
//!         start: Vec3::zeros(),
//!         end: Vec3::new(0.0, 1.0, 0.0),
//!         color: Vec4::new(0.0, 1.0, 0.0, 1.0),  // Y = Green
//!     });
//!     lines.push(Line {
//!         start: Vec3::zeros(),
//!         end: Vec3::new(0.0, 0.0, 1.0),
//!         color: Vec4::new(0.0, 0.0, 1.0, 1.0),  // Z = Blue
//!     });
//! }
//! ```
//!
//! # Clearing Lines Each Frame
//!
//! For dynamic debug visualization, clear and redraw each frame:
//!
//! ```ignore
//! fn run_systems(&mut self, world: &mut World) {
//!     if let Some(lines) = world.core.get_lines_mut(self.debug_lines) {
//!         lines.clear();
//!
//!         // Draw current frame's debug info
//!         for entity in world.core.query_entities(RIGID_BODY | LOCAL_TRANSFORM) {
//!             if let Some(transform) = world.core.get_local_transform(entity) {
//!                 // Draw velocity vector
//!                 if let Some(rb) = world.core.get_rigid_body(entity) {
//!                     lines.push(Line {
//!                         start: transform.translation,
//!                         end: transform.translation + rb.linear_velocity,
//!                         color: Vec4::new(1.0, 1.0, 0.0, 1.0),
//!                     });
//!                 }
//!             }
//!         }
//!     }
//! }
//! ```
//!
//! # Line Properties
//!
//! | Field | Type | Description |
//! |-------|------|-------------|
//! | `start` | Vec3 | World-space start position |
//! | `end` | Vec3 | World-space end position |
//! | `color` | Vec4 | RGBA color (linear, 0-1 range) |
//!
//! # Performance Notes
//!
//! - Lines are batched and rendered in a single draw call
//! - Maximum of 1 million lines per `Lines` component
//! - Call `clear()` before adding new lines to avoid accumulation
//! - Lines component is marked dirty automatically when modified

pub mod components;

pub use components::*;
