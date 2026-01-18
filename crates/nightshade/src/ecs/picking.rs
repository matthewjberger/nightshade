//! Entity picking via ray casting.
//!
//! Pick entities under the mouse cursor or from arbitrary rays:
//!
//! - [`pick_entities`]: Find all entities intersecting a ray (bounding box)
//! - [`pick_closest_entity`]: Find the nearest entity
//! - `pick_entities_trimesh`: Precise mesh intersection (requires `physics`)
//! - [`PickingRay`]: Ray definition for custom picking
//! - [`PickingResult`]: Hit information (entity, distance, position)
//!
//! # Basic Picking (Bounding Boxes)
//!
//! Fast picking using axis-aligned bounding boxes:
//!
//! ```ignore
//! fn on_mouse_input(&mut self, world: &mut World, button: MouseButton, state: KeyState) {
//!     if button == MouseButton::Left && state == KeyState::Pressed {
//!         let mouse_pos = world.resources.input.mouse.position;
//!         let ray = PickingRay::from_screen_position(world, mouse_pos);
//!
//!         if let Some(result) = pick_closest_entity(world, &ray, &PickingOptions::default()) {
//!             println!("Hit entity {:?} at distance {}", result.entity, result.distance);
//!             self.selected = Some(result.entity);
//!         }
//!     }
//! }
//! ```
//!
//! # Precise Trimesh Picking
//!
//! For accurate mesh intersection (requires `physics` feature):
//!
//! ```ignore
//! // Register entity for trimesh picking (do once after spawning)
//! register_entity_for_trimesh_picking(world, entity);
//!
//! // Or register entire hierarchy
//! register_entity_hierarchy_for_trimesh_picking(world, root_entity);
//!
//! // Pick with mesh precision
//! if let Some(result) = pick_closest_entity_trimesh(world, &ray) {
//!     let hit_position = result.position;  // Exact world position
//!     let hit_normal = result.normal;      // Surface normal at hit
//! }
//! ```
//!
//! # Frustum Picking (Selection Box)
//!
//! Select multiple entities within a screen rectangle:
//!
//! ```ignore
//! let entities = pick_entities_in_frustum(
//!     world,
//!     start_screen_pos,
//!     end_screen_pos,
//!     &PickingOptions::default(),
//! );
//! ```
//!
//! # Ground Position from Screen
//!
//! Project screen position onto the ground plane:
//!
//! ```ignore
//! if let Some(ground_pos) = get_ground_position_from_screen(world, mouse_pos, 0.0) {
//!     // ground_pos is Vec3 on the Y=0 plane
//! }
//! ```
//!
//! # Picking Options
//!
//! ```ignore
//! let options = PickingOptions {
//!     max_distance: 1000.0,           // Maximum ray distance
//!     include_transparent: false,     // Skip transparent materials
//!     layer_mask: u32::MAX,           // Filter by render layers
//! };
//! ```
//!
//! # PickingResult Fields
//!
//! | Field | Description |
//! |-------|-------------|
//! | `entity` | Hit entity handle |
//! | `distance` | Distance from ray origin |
//! | `position` | World-space hit point |
//! | `normal` | Surface normal (trimesh only) |
//!
//! [`pick_entities`]: queries::pick_entities
//! [`pick_closest_entity`]: queries::pick_closest_entity
//! [`PickingRay`]: queries::PickingRay
//! [`PickingResult`]: queries::PickingResult

#[cfg(feature = "physics")]
pub mod commands;
pub mod queries;
#[cfg(feature = "physics")]
pub mod resources;

#[cfg(feature = "physics")]
pub use commands::*;
pub use queries::*;
#[cfg(feature = "physics")]
pub use resources::*;
