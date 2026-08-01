use nightshade::prelude::*;

pub use crate::data::autopilot::State as Autopilot;
pub use crate::data::footing::State as Footing;

#[derive(Default, Clone, Debug)]
pub struct Around(pub Option<Entity>);

#[derive(Default, Clone, Debug)]
pub struct Offset(pub Vec3);

#[derive(Default, Clone, Debug)]
pub struct Pull(pub f32);

#[derive(Default, Clone, Debug)]
pub struct Reach(pub f32);

#[derive(Default, Clone, Debug)]
pub struct SpinRate(pub f32);

#[derive(Default, Clone, Debug)]
pub struct BodyRadius(pub f32);

#[derive(Default, Clone, Debug)]
pub struct Motion(pub Vec3);

#[derive(Default, Clone, Debug)]
pub struct Velocity(pub Vec3);

#[derive(Default, Clone, Debug)]
pub struct Ground(pub Option<Entity>);

#[derive(Default, Clone, Debug)]
pub struct Grounded(pub bool);

#[derive(Default, Clone, Debug)]
pub struct Bulk(pub f32);

#[derive(Default, Clone, Debug)]
pub struct Heading(pub Quat);

#[derive(Default, Clone, Debug)]
pub struct Pitch(pub f32);

#[derive(Default, Clone, Debug)]
pub struct Locked(pub Option<Entity>);

#[derive(Default, Clone, Debug)]
pub struct Push(pub Vec3);

#[derive(Default, Clone, Debug)]
pub struct Speaker(pub usize);

#[derive(Default, Clone, Debug)]
pub struct Relic(pub usize);

#[derive(Default, Clone, Debug)]
pub struct Grip(pub f32);

#[derive(Default, Clone, Debug)]
pub struct Draft(pub f32);

#[derive(Default, Clone, Debug)]
pub struct Gust(pub f32);

#[derive(Default, Clone, Debug)]
pub struct Fuel(pub f32);
