//! Camera-related resources.

/// The entity whose camera drives the primary view each frame. `None` renders
/// no scene camera.
#[derive(Default)]
pub struct ActiveCamera(pub Option<crate::ecs::world::Entity>);
