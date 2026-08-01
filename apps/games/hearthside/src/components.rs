use crate::data::Need;
use nightshade::prelude::{Entity, Vec3};

#[derive(Default, Clone, Copy, Debug)]
pub struct Feeds(pub Need);

#[derive(Default, Clone, Copy, Debug)]
pub struct FillRate(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Cell(pub usize);

#[derive(Default, Clone, Copy, Debug)]
pub struct Preview(pub usize);

#[derive(Default, Clone, Copy, Debug)]
pub struct Seat(pub bool);

#[derive(Default, Clone, Copy, Debug)]
pub struct Needs(pub [f32; 4]);

#[derive(Default, Clone, Debug)]
pub struct Route(pub Vec<Vec3>);

#[derive(Default, Clone, Copy, Debug)]
pub struct Facing(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Using(pub Option<Entity>);
