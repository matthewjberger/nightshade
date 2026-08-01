use nightshade::prelude::Vec3;

#[derive(Default, Clone, Copy, Debug)]
pub struct Velocity(pub Vec3);

#[derive(Default, Clone, Copy, Debug)]
pub struct Yaw(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Steer(pub f32);
