use crate::data::{EffectKind, ExplosionType};
use nightshade::prelude::*;

#[derive(Default, Clone, Copy)]
pub struct Velocity(pub Vec3);

#[derive(Default, Clone, Copy)]
pub struct Fuse(pub f32);

#[derive(Default, Clone, Copy)]
pub struct Tint(pub Vec3);

#[derive(Default, Clone, Copy)]
pub struct SecondTint(pub Option<Vec3>);

#[derive(Default, Clone, Copy)]
pub struct Burst(pub ExplosionType);

#[derive(Default, Clone, Copy)]
pub struct Generation(pub u8);

#[derive(Default, Clone, Copy)]
pub struct Effect(pub EffectKind);
