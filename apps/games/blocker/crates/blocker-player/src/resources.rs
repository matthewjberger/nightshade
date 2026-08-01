use blocker_blocks::data::items::Item;
use blocker_blocks::resources::Hit;
use nalgebra_glm::{IVec3, Vec3};
use nightshade::ecs::Entity;

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Aim(pub Option<Hit>);

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Mining {
    pub block: Option<IVec3>,
    pub progress: f32,
    pub stage: Option<usize>,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Markers {
    pub outline: Option<Entity>,
    pub outlined: Option<IVec3>,
    pub crack: Option<Entity>,
    pub crack_shown: bool,
}

#[derive(Default)]
pub struct Hand {
    pub entity: Option<Entity>,
    pub shown: Option<Item>,
    pub swing: f32,
}

#[derive(Clone)]
pub struct Profile {
    pub name: String,
}

impl Default for Profile {
    fn default() -> Self {
        Self {
            name: crate::data::body::PLAYER_NAME.to_string(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Default)]
pub enum Prompt {
    #[default]
    None,
    Chest(IVec3),
    Board(Entity),
    Leave,
    Door(IVec3, bool),
    Harvest(Entity),
    Craft,
    Sign(String),
    Talk(String),
}

#[derive(Clone, Copy, Debug)]
pub struct Shell {
    pub entity: Entity,
    pub position: Vec3,
    pub velocity: Vec3,
    pub fuse: f32,
    pub color: Vec3,
}

#[derive(Default)]
pub struct Fireworks {
    pub shells: Vec<Shell>,
}

pub struct Cursor {
    pub shown: bool,
    pub playing: bool,
}

impl Default for Cursor {
    fn default() -> Self {
        Self {
            shown: true,
            playing: false,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Resumed {
    pub position: Vec3,
    pub home: Vec3,
    pub yaw: f32,
    pub pitch: f32,
}

#[derive(Default, Clone, Copy, Debug)]
pub struct Resume(pub Option<Resumed>);
