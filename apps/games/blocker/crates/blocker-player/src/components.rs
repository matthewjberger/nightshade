pub use crate::data::motion::State as Motion;
use blocker_blocks::data::blocks::BlockKind;
use blocker_blocks::data::items::Item;
use nalgebra_glm::Vec3;
use nightshade::ecs::Entity;

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
pub struct Food(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Air(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Age(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Phase(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Boat(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Yaw(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Pitch(pub f32);

#[derive(Clone, Copy, Debug)]
pub struct Zoom(pub f32);

impl Default for Zoom {
    fn default() -> Self {
        Self(crate::theme::ZOOM_START)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ZoomWanted(pub f32);

impl Default for ZoomWanted {
    fn default() -> Self {
        Self(crate::theme::ZOOM_START)
    }
}

#[derive(Default, Clone, Copy, Debug)]
pub struct Grounded(pub bool);

#[derive(Default, Clone, Copy, Debug)]
pub struct Submerged(pub bool);

#[derive(Default, Clone, Copy, Debug)]
pub struct Fall(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Spawn(pub Vec3);

#[derive(Clone, Copy, Debug)]
pub struct WatchSpeed(pub f32);

impl Default for WatchSpeed {
    fn default() -> Self {
        Self(crate::theme::WATCH_SPEED)
    }
}

#[derive(Default, Clone, Copy, Debug)]
pub struct Riding(pub Entity);

#[derive(Default, Clone, Copy, Debug)]
pub struct Wing(pub Entity);
