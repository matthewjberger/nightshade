use nightshade::prelude::Vec3;

#[derive(Default, Clone, Copy, Debug)]
pub struct Aim(pub Vec3);

#[derive(Default, Clone, Copy, Debug)]
pub struct Hook(pub Vec3);

#[derive(Default, Clone, Copy, Debug)]
pub struct Grounded(pub bool);

#[derive(Default, Clone, Copy, Debug)]
pub struct Home(pub Vec3);

#[derive(Default, Clone, Copy, Debug)]
pub struct LineLength(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Phase(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Speed(pub f32);
