use nightshade::prelude::*;

#[derive(Default, Clone, Copy)]
pub struct Label(pub usize);

#[derive(Default, Clone, Copy)]
pub struct OrbitRadius(pub f32);

#[derive(Default, Clone, Copy)]
pub struct Height(pub f32);

#[derive(Default, Clone, Copy)]
pub struct Angle(pub f32);

#[derive(Default, Clone, Copy)]
pub struct AngularSpeed(pub f32);

#[derive(Default, Clone, Copy)]
pub struct BobPhase(pub f32);

#[derive(Default, Clone, Copy)]
pub struct Health(pub i32);

#[derive(Default, Clone, Copy)]
pub struct Hit(pub f32);

#[derive(Default, Clone, Copy)]
pub struct Respawn(pub f32);

#[derive(Default, Clone, Copy)]
pub struct Velocity(pub Vec3);

#[derive(Default, Clone, Copy)]
pub struct Life(pub f32);

#[derive(Default, Clone, Copy)]
pub struct Target(pub Entity);
