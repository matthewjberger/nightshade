use crate::data::Kind;
use nightshade::prelude::Entity;

#[derive(Default)]
pub struct Chosen(pub Option<Entity>);

#[derive(Default)]
pub struct Compiled {
    pub source: String,
    pub revision: u64,
    pub nodes: usize,
    pub wires: usize,
}

#[derive(Default)]
pub struct PaletteRows {
    pub buttons: Vec<(Entity, Kind)>,
    pub presets: Option<Entity>,
    pub pluck: Option<Entity>,
    pub write: Option<Entity>,
    pub shown: usize,
    pub placed: u32,
}

#[derive(Default)]
pub struct Tapping {
    pub node: Option<Entity>,
    pub travel: f32,
    pub tapped: Option<Entity>,
}

pub const SLOTS: usize = 8;
pub const KEPT: usize = 2;

#[derive(Default, Clone)]
pub struct Plucked {
    pub name: String,
    pub rgba: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

pub struct Pictures {
    pub held: Vec<Option<Plucked>>,
    pub revision: u64,
    pub next: usize,
    pub nudge: f32,
    pub trouble: Option<String>,
}

impl Default for Pictures {
    fn default() -> Self {
        Self {
            held: vec![None; SLOTS],
            revision: 0,
            next: 0,
            nudge: 0.0,
            trouble: None,
        }
    }
}

#[derive(Default)]
pub struct Preview {
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
}
