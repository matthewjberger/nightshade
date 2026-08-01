use super::story::Mood;
use super::{Class, Facing, Growth, Mobility, StructureKind, Taste, Team, Upgrade};
use nightshade::prelude::Entity;

#[derive(Default, Clone, Debug)]
pub struct Fighter {
    pub name: &'static str,
    pub title: &'static str,
    pub class: Class,
    pub taste: Taste,
    pub mood: Mood,
    pub slot: usize,
    pub column: u32,
    pub row: u32,
    pub facing: Facing,
    pub experience: u32,
    pub rapport: u32,
    pub bonds: Vec<(Entity, u32)>,
    pub morale: i32,
    pub wounded: u32,
    pub deployed: bool,
    pub busy: bool,
    pub kills: u32,
    pub battles: u32,
    pub joined: bool,
    pub arc_stage: u32,
    pub boon: Growth,
    pub fallen: bool,
    pub marks: u32,
    pub taken: Vec<Upgrade>,
}

#[derive(Default, Clone, Debug)]
pub struct Unit {
    pub fighter: Option<Entity>,
    pub bonds: Vec<(Entity, u32)>,
    pub merit: u32,
    pub kills: u32,
    pub name: &'static str,
    pub class: Class,
    pub team: Team,
    pub mobility: Mobility,
    pub hp: i32,
    pub max_hp: i32,
    pub mp: i32,
    pub max_mp: i32,
    pub attack: i32,
    pub defense: i32,
    pub magic: i32,
    pub resist: i32,
    pub speed: i32,
    pub move_range: u32,
    pub jump: u32,
    pub evasion: i32,
    pub ct: i32,
    pub facing: Facing,
    pub column: u32,
    pub row: u32,
    pub flipped: bool,
    pub flash: f32,
    pub flash_tint: [f32; 4],
    pub stunned: u32,
    pub suppressed: u32,
}

#[derive(Default, Clone, Debug)]
pub struct Structure {
    pub kind: StructureKind,
    pub owner: Option<Team>,
    pub column: u32,
    pub row: u32,
    pub flag: Option<Entity>,
}
