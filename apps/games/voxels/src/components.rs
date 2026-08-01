use crate::data::{BlockKind, Item};
use fastrand::Rng;
use nalgebra_glm::{IVec3, Vec2, Vec3};

#[derive(Clone, Copy, Debug)]
pub struct Held(pub Item);

impl Default for Held {
    fn default() -> Self {
        Self(Item::Block(BlockKind::Dirt))
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Count(pub u32);

impl Default for Count {
    fn default() -> Self {
        Self(1)
    }
}

#[derive(Default, Clone, Copy, Debug)]
pub struct Velocity(pub Vec3);

#[derive(Default, Clone, Copy, Debug)]
pub struct Age(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Phase(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct NpcName(pub &'static str);

#[derive(Default, Clone, Copy, Debug)]
pub struct Home(pub Vec2);

#[derive(Default, Clone, Copy, Debug)]
pub struct Target(pub Vec2);

#[derive(Clone, Copy, Debug)]
pub struct Reach(pub f32);

impl Default for Reach {
    fn default() -> Self {
        Self(3.0)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Speed(pub f32);

impl Default for Speed {
    fn default() -> Self {
        Self(1.0)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Rest(pub f32);

impl Default for Rest {
    fn default() -> Self {
        Self(1.0)
    }
}

#[derive(Clone, Debug)]
pub struct Roll(pub Rng);

impl Default for Roll {
    fn default() -> Self {
        Self(Rng::with_seed(1))
    }
}

#[derive(Default, Clone, Copy, Debug)]
pub struct Block(pub IVec3);

#[derive(Default, Clone, Copy, Debug)]
pub struct Progress(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Stage(pub usize);

#[derive(Default, Clone, Copy, Debug)]
pub struct Watered(pub bool);

#[derive(Default, Clone, Copy, Debug)]
pub struct GlowBlock(pub [i32; 3]);

#[derive(Default, Clone, Copy, Debug)]
pub struct SignLabel {
    pub block: [i32; 3],
}

#[derive(Default, Clone, Copy, Debug)]
pub struct Boat {
    pub yaw: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct Health(pub f32);

impl Default for Health {
    fn default() -> Self {
        Self(1.0)
    }
}

#[derive(Default, Clone, Copy, Debug)]
pub struct Hurt(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Wait(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Bob(pub f32);
