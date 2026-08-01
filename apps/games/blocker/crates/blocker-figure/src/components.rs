use crate::data::joints::Joint;
use nalgebra_glm::{Vec2, Vec3};
use nightshade::ecs::Entity;

#[derive(Default, Clone, Copy, Debug)]
pub struct Gait(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Pace(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Limb(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Bone(pub Joint);

#[derive(Default, Clone, Copy, Debug)]
pub struct Soar(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Head(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Tilt(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Regard(pub Vec3);

#[derive(Default, Clone, Copy, Debug)]
pub struct Plate(pub Entity);

#[derive(Default, Clone, Copy, Debug)]
pub struct Look(pub usize);

#[derive(Clone, Copy, Debug)]
pub struct Fit(pub f32);

impl Default for Fit {
    fn default() -> Self {
        Self(1.0)
    }
}

#[derive(Default, Clone, Copy, Debug)]
pub struct Eye(pub Vec3);

#[derive(Clone, Copy, Debug)]
pub struct Gaze(pub Vec3);

impl Default for Gaze {
    fn default() -> Self {
        Self(Vec3::new(0.0, 0.0, -1.0))
    }
}

#[derive(Default, Clone, Copy, Debug)]
pub struct Home(pub Vec2);

#[derive(Default, Clone, Copy, Debug)]
pub struct Heading(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Owner(pub Entity);
