use crate::components::{Resting, Wandering};
use crate::resources::WanderSettings;
use nalgebra_glm::Vec3;
use nightshade_plane::Plane;
use nightshade_plane::queries::across;

pub fn arrived(
    plane: Plane,
    position: Vec3,
    wandering: &Wandering,
    settings: &WanderSettings,
) -> bool {
    across(plane, position, wandering.0) <= settings.arrival
}

pub fn resting(resting: &Resting) -> bool {
    resting.0 > 0.0
}
