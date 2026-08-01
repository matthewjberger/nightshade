use nightshade::prelude::Vec3;

#[derive(Clone, Copy, Debug, Default)]
pub struct Offset(pub f32);

#[derive(Clone, Copy, Debug, Default)]
pub struct Yaw(pub f32);

#[derive(Clone, Copy, Debug, Default)]
pub struct Home(pub Vec3);

#[derive(Clone, Copy, Debug, Default)]
pub struct Reach(pub f32);
