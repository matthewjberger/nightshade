use super::slots::{FileSlot, LabeledSlot, PairingSlot};
use nightshade::prelude::Entity;

#[derive(Default, Clone, Copy)]
pub struct FileRow {
    pub file: usize,
    pub slot: FileSlot,
}

pub struct OpenFile;

#[derive(Default, Clone, Copy)]
pub struct LampRow {
    pub program: usize,
    pub place: usize,
    pub lit: bool,
    pub slot: LabeledSlot,
}

#[derive(Default, Clone, Copy)]
pub struct WayRow {
    pub way: usize,
    pub slot: LabeledSlot,
}

#[derive(Default, Clone, Copy)]
pub struct ThingRow {
    pub thing: usize,
    pub slot: LabeledSlot,
}

#[derive(Default, Clone, Copy)]
pub struct CaseRow {
    pub file: usize,
    pub slot: LabeledSlot,
}

#[derive(Default, Clone, Copy)]
pub struct GamePlace {
    pub place: usize,
    pub face: char,
    pub label: Entity,
}

#[derive(Default, Clone, Copy)]
pub struct BookmarkRow {
    pub bookmark: usize,
    pub filed: bool,
    pub slot: LabeledSlot,
}

#[derive(Default, Clone, Copy)]
pub struct LinkRow {
    pub link: usize,
    pub slot: LabeledSlot,
}

#[derive(Default, Clone, Copy)]
pub struct StoreButton(pub usize);

#[derive(Default, Clone, Copy)]
pub struct StoreLink(pub &'static str);

#[derive(Default, Clone, Copy)]
pub struct EndingRow {
    pub which: usize,
    pub root: Entity,
    pub dot: Entity,
    pub label: Entity,
    pub note: Entity,
}

#[derive(Default, Clone, Copy)]
pub struct StoreRow {
    pub slot: usize,
    pub listing: usize,
    pub root: Entity,
    pub title: Entity,
    pub by: Entity,
    pub blurb: Entity,
    pub note: Entity,
    pub button: Entity,
    pub button_label: Entity,
    pub mark: Entity,
    pub glyph: Entity,
}

#[derive(Default, Clone, Copy)]
pub struct PageBlock {
    pub block: usize,
    pub slot: LabeledSlot,
    pub mark: Entity,
    pub right: Entity,
    pub rule: Entity,
    pub stripe: Entity,
    pub above: Entity,
    pub below: Entity,
}

#[derive(Default, Clone, Copy)]
pub struct TallyRow {
    pub lead: usize,
    pub label: Entity,
}

#[derive(Default, Clone, Copy)]
pub struct SessionRow(pub usize);

#[derive(Default, Clone, Copy)]
pub struct PairingRow {
    pub pairing: usize,
    pub slot: PairingSlot,
}

#[derive(Default, Clone, Copy)]
pub struct NoteCard {
    pub note: usize,
    pub root: Entity,
    pub text: Entity,
    pub author: Entity,
}

#[derive(Default, Clone, Copy)]
pub struct BoardColumn {
    pub lead: usize,
    pub heading: Entity,
    pub footer: Entity,
}

#[derive(Default, Clone, Copy)]
pub struct BoardCard {
    pub lead: usize,
    pub note: usize,
    pub label: Entity,
}
