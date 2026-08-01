use crate::contract::{Tier, Tower};
use nightshade::prelude::Entity;

#[derive(Default, Clone, Copy, Debug)]
pub struct Health(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct MaxHealth(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Leg(pub usize);

#[derive(Default, Clone, Copy, Debug)]
pub struct Progress(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Speed(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Pace(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Chill(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Ward(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Splits(pub u8);

#[derive(Default, Clone, Copy, Debug)]
pub struct Cut(pub Tier);

#[derive(Default, Clone, Copy, Debug)]
pub struct Tint(pub [f32; 4]);

#[derive(Default, Clone, Copy, Debug)]
pub struct Chills(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Splash(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Pierce(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Bounty(pub u32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Bar(pub Entity);

#[derive(Default, Clone, Copy, Debug)]
pub struct Backing(pub Entity);

#[derive(Default, Clone, Copy, Debug)]
pub struct Shade(pub Entity);

#[derive(Default, Clone, Copy, Debug)]
pub struct Rising(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Bob(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Flash(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Reach(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Damage(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Reload(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Cooldown(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Focus(pub u8);

#[derive(Default, Clone, Copy, Debug)]
pub struct Kind(pub Tower);

#[derive(Default, Clone, Copy, Debug)]
pub struct Level(pub u8);

#[derive(Default, Clone, Copy, Debug)]
pub struct Post(pub u32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Aim(pub Entity);

#[derive(Default, Clone, Copy, Debug)]
pub struct Owner(pub Entity);

#[derive(Default, Clone, Copy, Debug)]
pub struct Tally(pub u32);
