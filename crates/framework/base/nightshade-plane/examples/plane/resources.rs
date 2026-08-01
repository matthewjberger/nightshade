use nightshade::prelude::Entity;
use nightshade_plane::Plane;

#[derive(Default, Clone, Copy)]
pub struct Walk {
    pub walker: Option<Entity>,
    pub shown: Plane,
}
