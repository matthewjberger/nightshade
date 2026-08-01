use super::slots::{LabeledSlot, RowSlot};
use nightshade::prelude::Entity;
use nightshade::prelude::{Vec2, vec2};

#[derive(Default, Clone, Copy)]
pub struct AccountRow {
    pub definition: usize,
    pub slot: RowSlot,
}

#[derive(Default, Clone, Copy)]
pub struct AccountOpened(pub bool);

pub struct SignedIn;

pub struct SelectedAccount;

#[derive(Default, Clone, Copy)]
pub struct ProgramIcon(pub usize);

#[derive(Default, Clone, Copy)]
pub struct ToastRow {
    pub slot: usize,
    pub root: Entity,
    pub who: Entity,
    pub body: Entity,
}

#[derive(Default, Clone, Copy)]
pub struct NoticeRow {
    pub slot: usize,
    pub root: Entity,
    pub stamp: Entity,
    pub who: Entity,
    pub body: Entity,
}

#[derive(Default, Clone, Copy)]
pub struct ChecklistRow {
    pub row: usize,
    pub root: Entity,
    pub tick: Entity,
    pub label: Entity,
}

#[derive(Default, Clone, Copy)]
pub struct StartRow {
    pub program: usize,
    pub slot: LabeledSlot,
}

#[derive(Default, Clone, Copy)]
pub struct TaskbarButton {
    pub program: usize,
    pub lamp: Entity,
    pub mark: Entity,
    pub label: Entity,
    pub badge: LabeledSlot,
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum FrameButtonKind {
    #[default]
    Minimize,
    Size,
    Close,
}

#[derive(Default, Clone, Copy)]
pub struct FrameButton {
    pub program: usize,
    pub kind: FrameButtonKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum WindowMode {
    #[default]
    Floating,
    Maximized,
}

#[derive(Clone, Copy)]
pub struct ProgramWindow {
    pub program: usize,
    pub position: Vec2,
    pub size: Vec2,
    pub mode: WindowMode,
    pub open: bool,
    pub minimized: bool,
    pub order: u32,
    pub installed: bool,
    pub announced: bool,
}

pub struct Dragging;

pub struct Resizing;

#[derive(Default, Clone, Copy)]
pub struct BarPressed(pub Option<f64>);

impl Default for ProgramWindow {
    fn default() -> Self {
        Self {
            program: 0,
            position: vec2(96.0, 64.0),
            size: vec2(840.0, 540.0),
            mode: WindowMode::Floating,
            open: false,
            minimized: false,
            order: 0,
            installed: false,
            announced: false,
        }
    }
}
