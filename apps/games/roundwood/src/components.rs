use crate::data::Item;
use fastrand::Rng;
use nalgebra_glm::Vec2;

#[derive(Default, Clone, Copy, Debug)]
pub struct Spot(pub Vec2);

#[derive(Default, Clone, Copy, Debug)]
pub struct Base(pub Vec2);

#[derive(Default, Clone, Copy, Debug)]
pub struct Lift(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Speed(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Stride(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Home(pub Vec2);

#[derive(Default, Clone, Copy, Debug)]
pub struct Target(pub Vec2);

#[derive(Default, Clone, Copy, Debug)]
pub struct Reach(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Rest(pub f32);

#[derive(Default, Clone, Debug)]
pub struct Roll(pub Rng);

#[derive(Default, Clone, Copy, Debug)]
pub struct Villager(pub usize);

#[derive(Default, Clone, Copy, Debug)]
pub struct Phase(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Strength(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Lit(pub [f32; 4]);

#[derive(Default, Clone, Copy, Debug)]
pub struct Ripe(pub bool);

#[derive(Default, Clone, Copy, Debug)]
pub struct Regrow(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Tree(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Rocks(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Shaking(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Lean(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Held(pub Item);

#[derive(Default, Clone, Copy, Debug)]
pub struct Count(pub u32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Hop(pub f32);
