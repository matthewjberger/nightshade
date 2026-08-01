use crate::contract::{Fixture, HuntState, ItemKind};
use nightshade::prelude::Vec3;

#[derive(Default, Clone, Copy, Debug)]
pub struct Handle(pub Fixture);

#[derive(Clone, Copy, Debug)]
pub struct Heft(pub f32);

impl Default for Heft {
    fn default() -> Self {
        Self(1.0)
    }
}

#[derive(Default, Clone, Copy, Debug)]
pub struct Item(pub ItemKind);

#[derive(Default, Clone, Copy, Debug)]
pub struct Locked(pub Option<ItemKind>);

#[derive(Default, Clone, Copy, Debug)]
pub struct Readable(pub usize);

#[derive(Default, Clone, Copy, Debug)]
pub struct Hunt(pub HuntState);

#[derive(Default, Clone, Copy, Debug)]
pub struct Interest(pub Vec3);

#[derive(Default, Clone, Copy, Debug)]
pub struct Dwell(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Stride(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Waypoint(pub usize);

#[derive(Default, Clone, Copy, Debug)]
pub struct Facing(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Puppet(pub Option<nightshade::prelude::Entity>);
