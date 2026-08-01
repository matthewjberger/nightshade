use nalgebra_glm::Vec3;

#[derive(Clone, Copy)]
pub struct Exposure(pub f32);

impl Default for Exposure {
    fn default() -> Self {
        Self(1.0)
    }
}

#[derive(Default, Clone, Copy)]
pub struct Hearing(pub f32);

#[derive(Default, Clone, Copy)]
pub struct Sight(pub f32);

#[derive(Default, Clone, Copy)]
pub struct Alert(pub f32);

#[derive(Default, Clone, Copy)]
pub struct LastSeen(pub Vec3);
