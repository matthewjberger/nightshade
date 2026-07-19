//! Entity selection and highlight state.
//!
//! [`Selection`] is the engine resource behind the selection outline and the
//! selected-entity bounding volume. Apps and editor tools write it directly;
//! the render sync resolves it into the outline id list the renderer consumes.

use nightshade_ecs::Entity;

/// The selected entities and how they are highlighted. Held at
/// `world.res::<crate::ecs::graphics::selection::Selection>()`.
#[derive(Clone)]
pub struct Selection {
    /// Enables the selection outline post-process.
    pub outline_enabled: bool,
    /// The color the selection outline is drawn with.
    pub outline_color: [f32; 4],
    /// The primary selected entity: bounding-volume highlight, gizmo target,
    /// and outline seed.
    pub active_entity: Option<Entity>,
    /// Additional entities included in the selection outline alongside
    /// `active_entity` and their descendants.
    pub entities: Vec<Entity>,
}

impl Default for Selection {
    fn default() -> Self {
        Self {
            outline_enabled: false,
            outline_color: [1.0, 0.45, 0.0, 1.0],
            active_entity: None,
            entities: Vec::new(),
        }
    }
}
