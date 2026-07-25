//! Camera-related resources.

/// The entity whose camera drives the primary view each frame. `None` renders
/// no scene camera.
#[derive(Default)]
pub struct ActiveCamera(pub Option<crate::ecs::world::Entity>);

/// An optional resolver that shortens the third-person camera distance so
/// the view does not clip through the world. The physics plugin installs
/// one; without it the camera keeps its full distance. The resolver
/// receives the world, the focus point, the ideal camera position, the
/// collision radius, and the desired distance, and returns the distance to
/// use. Kept opaque so the camera domain does not depend on the physics
/// plugin. A plain function pointer rather than a boxed closure: the value is
/// `Copy`, so reading it costs nothing and callers need no take-and-restore.
type CameraCollisionResolver =
    fn(&crate::ecs::world::World, nalgebra_glm::Vec3, nalgebra_glm::Vec3, f32, f32) -> f32;

#[derive(Default)]
pub struct CameraCollision {
    pub resolver: Option<CameraCollisionResolver>,
}
