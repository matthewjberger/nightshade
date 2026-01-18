/// Marker component to exclude parent scale from transform inheritance.
///
/// When present, the entity's [`super::GlobalTransform`] will not include
/// the scale from its parent hierarchy. Useful for UI elements or objects
/// that should maintain consistent size regardless of parent scaling.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct IgnoreParentScale;
