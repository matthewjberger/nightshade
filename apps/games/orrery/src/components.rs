use nightshade::prelude::Vec3;

#[derive(Default, Clone, Copy, Debug)]
pub struct Body(pub usize);

#[derive(Default, Clone, Copy, Debug)]
pub struct Velocity(pub Vec3);

#[derive(Default, Clone, Copy, Debug)]
pub struct Landed(pub Option<usize>);

#[derive(Default, Clone, Copy, Debug)]
pub struct Anchor(pub Vec3);

#[derive(Default, Clone, Copy, Debug)]
pub struct Target(pub usize);

#[derive(Default, Clone, Copy, Debug)]
pub struct Autopilot(pub bool);

#[derive(Default, Clone, Copy, Debug)]
pub struct StandingOn(pub usize);

#[derive(Default, Clone, Copy, Debug)]
pub struct Offset(pub Vec3);

#[derive(Default, Clone, Copy, Debug)]
pub struct Grounded(pub bool);

#[derive(Default, Clone, Copy, Debug)]
pub struct Inside(pub bool);

#[derive(Default, Clone, Copy, Debug)]
pub struct Found(pub bool);

#[derive(Default, Clone, Copy, Debug)]
pub struct Spin(pub f32);
