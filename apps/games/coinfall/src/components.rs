use nightshade::prelude::{Vec2, Vec3};

#[derive(Default, Clone, Copy, Debug)]
pub struct Home(pub Vec3);

#[derive(Default, Clone, Copy, Debug)]
pub struct Velocity(pub Vec2);

#[derive(Default, Clone, Copy, Debug)]
pub struct FacingLeft(pub bool);

#[derive(Default, Clone, Copy, Debug)]
pub struct Speed(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Direction(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Alive(pub bool);

#[derive(Default, Clone, Copy, Debug)]
pub struct Phase(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Parallax(pub f32);
