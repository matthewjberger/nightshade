use crate::data::{Person, Potion};

use fastrand::Rng;
use nalgebra_glm::Vec2;
use nightshade::prelude::Entity;

#[derive(Default, Clone, Copy, Debug)]
pub struct Speed(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Stride(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Rest(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Target(pub Vec2);

#[derive(Default, Clone, Copy, Debug)]
pub struct Home(pub Vec2);

#[derive(Default, Clone, Copy, Debug)]
pub struct Reach(pub f32);

#[derive(Default, Clone, Debug)]
pub struct Roll(pub Rng);

#[derive(Default, Clone, Copy, Debug)]
pub struct Phase(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Strength(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct NightOnly(pub bool);

#[derive(Default, Clone, Debug)]
pub struct Npc {
    pub person: Person,
}

#[derive(Default, Clone, Copy, Debug)]
pub struct House(pub Option<Vec2>);

#[derive(Default, Clone, Copy, Debug)]
pub struct Post(pub Vec2);

#[derive(Default, Clone, Copy, Debug)]
pub struct Arrive(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Inside(pub bool);

#[derive(Default, Clone, Copy, Debug)]
pub struct Stuck(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Shadow(pub Option<Entity>);

#[derive(Default, Clone, Copy, Debug)]
pub struct Progress(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct PlotIndex(pub usize);

#[derive(Default, Clone, Copy, Debug)]
pub struct Hits(pub u32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Wood(pub u32);

#[derive(Default, Clone, Debug)]
pub struct Lit {
    pub base: [f32; 4],
}

#[derive(Default, Clone, Copy, Debug)]
pub struct Bend(pub f32);

#[derive(Default, Clone, Debug)]
pub struct Stump {
    pub since_day: u32,
}

#[derive(Default, Clone, Debug)]
pub struct Bush {
    pub picked_day: Option<u32>,
}

#[derive(Default, Clone, Debug)]
pub struct Milkable {
    pub milked_day: Option<u32>,
}

#[derive(Default, Clone, Debug)]
pub struct Shake {
    pub remaining: f32,
}

#[derive(Default, Clone, Copy, Debug)]
pub struct Count(pub u32);

#[derive(Default, Clone, Debug)]
pub struct Upright {
    pub yaw: f32,
}

#[derive(Default, Clone, Copy, Debug)]
pub struct Remaining(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Flip(pub bool);

#[derive(Default, Clone, Debug)]
pub struct Layer {
    pub until_next: f32,
}

#[derive(Default, Clone, Copy, Debug)]
pub struct Hp(pub u32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Cooldown(pub f32);

#[derive(Default, Clone, Debug)]
pub struct Hurt {
    pub remaining: f32,
}

#[derive(Default, Clone, Copy, Debug)]
pub struct Opened(pub bool);

#[derive(Default, Clone, Copy, Debug)]
pub struct Mimic(pub bool);

#[derive(Default, Clone, Debug)]
pub struct Ladder {
    pub down: bool,
}

#[derive(Default, Clone, Copy, Debug)]
pub struct Vial(pub Option<Potion>);

#[derive(Default, Clone, Copy, Debug)]
pub struct Coins(pub u32);
