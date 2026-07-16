//! Entity picking from a screen position.
//!
//! Pick entities under the mouse cursor (a ray is cast from the camera through
//! the screen point internally):
//!
//! - [`pick_entities`]: All entities under a screen position (bounding box), nearest first
//! - [`pick_closest_entity`]: The nearest entity under a screen position
//! - `pick_closest_entity_trimesh`: Precise mesh intersection (requires `physics`)
//! - [`pick_entities_in_frustum`]: Filter a list of entities to those in the camera frustum
//! - [`PickingResult`]: Hit information (entity, distance, world_position)
//!
//! # Basic Picking (Bounding Boxes)
//!
//! Fast picking using axis-aligned bounding boxes:
//!
//! ```ignore
//! // Each frame, pick the entity under the cursor:
//! let mouse_pos = world.res::<crate::ecs::input::resources::Input>().mouse.position;
//! if let Some(result) = pick_closest_entity(world, mouse_pos) {
//!     println!("Hit entity {:?} at distance {}", result.entity, result.distance);
//!     self.selected = Some(result.entity);
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
//! if let Some(result) = pick_closest_entity_trimesh(world, mouse_pos) {
//!     let hit_position = result.world_position;  // exact world-space hit point
//! }
//! ```
//!
//! # Frustum Filtering
//!
//! Keep only the entities from a list that fall inside the camera frustum:
//!
//! ```ignore
//! // Filter a candidate list down to those inside the camera frustum:
//! let visible = pick_entities_in_frustum(world, &candidate_entities);
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
//!     max_distance: 1000.0,   // maximum ray distance
//!     ignore_invisible: true, // skip hidden entities
//! };
//! let results = pick_entities(world, mouse_pos, options);
//! ```
//!
//! # PickingResult Fields
//!
//! | Field | Description |
//! |-------|-------------|
//! | `entity` | Hit entity handle |
//! | `distance` | Distance from the camera |
//! | `world_position` | World-space hit point |
//!
//! [`pick_entities`]: queries::pick_entities
//! [`pick_closest_entity`]: queries::pick_closest_entity
//! [`pick_entities_in_frustum`]: queries::pick_entities_in_frustum
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
