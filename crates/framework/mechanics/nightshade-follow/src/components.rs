use nalgebra_glm::Vec3;
use nightshade_ecs::Entity;

#[derive(Default, Clone, Copy)]
pub struct Following(pub Entity);

#[derive(Default, Clone, Copy)]
pub struct FollowOffset(pub Vec3);
