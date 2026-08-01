use crate::resources::Hull;
use nightshade::prelude::{Entity, Vec3};

#[derive(Default, Clone, Copy, Debug)]
pub struct Look(pub usize);

#[derive(Default, Clone, Copy, Debug)]
pub struct Fit(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Rig(pub Option<Entity>);

#[derive(Default, Clone, Copy, Debug)]
pub struct Playing(pub usize);

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cell(pub [i32; 2]);

#[derive(Default, Clone, Copy, Debug)]
pub struct Velocity(pub Vec3);

#[derive(Default, Clone, Copy, Debug)]
pub struct Yaw(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Steer(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Heading(pub u8);

#[derive(Default, Clone, Copy, Debug)]
pub struct Turn(pub u8);

#[derive(Default, Clone, Copy, Debug)]
pub struct Next(pub [i32; 2]);

#[derive(Default, Clone, Copy, Debug)]
pub struct Bend(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Cruise(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Stride(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Phase(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Pace(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Corner(pub [i32; 3]);

#[derive(Default, Clone, Copy, Debug)]
pub struct Came(pub [i32; 3]);

#[derive(Default, Clone, Copy, Debug)]
pub struct Idle(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Dying(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Prop(pub usize);

#[derive(Default, Clone, Copy, Debug)]
pub struct Glow(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Body(pub Hull);

#[derive(Default, Clone, Copy, Debug)]
pub struct Toppling(pub Vec3);

#[derive(Default, Clone, Copy, Debug)]
pub struct Pivot(pub Vec3);

#[derive(Default, Clone, Copy, Debug)]
pub struct Fallen(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Indoor(pub u32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Fade(pub f32);
