use crate::data::wire::PeerId;
use nalgebra_glm::Vec3;
use nightshade::prelude::Entity;

#[derive(Default, Clone, Copy, Debug)]
pub struct Peer(pub PeerId);

#[derive(Default, Clone, Copy, Debug)]
pub struct Steering(pub Vec3);

#[derive(Default, Clone, Copy, Debug)]
pub struct Wants(pub u8);

#[derive(Default, Clone, Copy, Debug)]
pub struct At(pub Vec3);

#[derive(Default, Clone, Copy, Debug)]
pub struct WasAt(pub Vec3);

#[derive(Default, Clone, Copy, Debug)]
pub struct WasFacing(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Velocity(pub Vec3);

#[derive(Default, Clone, Copy, Debug)]
pub struct Facing(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Crouch(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Grounded(pub bool);

#[derive(Default, Clone, Copy, Debug)]
pub struct JumpHold(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Avatar(pub Option<Entity>);

#[derive(Default, Clone, Copy, Debug)]
pub struct Look(pub usize);

#[derive(Default, Clone, Copy, Debug)]
pub struct Fit(pub f32);

#[derive(Default, Clone, Debug)]
pub struct Track(pub std::collections::VecDeque<(f32, crate::data::wire::Body)>);

#[derive(Default, Clone, Copy, Debug)]
pub struct Replica(pub u32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Fade(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Ring(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Arc(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Pace(pub f32);
