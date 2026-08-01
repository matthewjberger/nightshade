use nightshade::prelude::Entity;

#[derive(Default, Clone, Copy)]
pub struct RowSlot {
    pub root: Entity,
    pub avatar: Entity,
    pub letter: Entity,
    pub silhouette_head: Entity,
    pub silhouette_body: Entity,
    pub dot: Entity,
    pub name: Entity,
    pub status: Entity,
    pub badge: Entity,
    pub badge_text: Entity,
}

#[derive(Clone, Copy, Default)]
pub struct LabeledSlot {
    pub root: Entity,
    pub label: Entity,
}

#[derive(Clone, Copy, Default)]
pub struct FileSlot {
    pub root: Entity,
    pub name: Entity,
    pub owner: Entity,
}

#[derive(Clone, Copy, Default)]
pub struct PairingSlot {
    pub root: Entity,
    pub source: Entity,
    pub claim: Entity,
    pub record: Entity,
    pub reading: Entity,
    pub verdict: Entity,
}

#[derive(Clone, Copy, Default)]
pub struct FrameSlot {
    pub root: Entity,
    pub bar: Entity,
    pub title: Entity,
    pub content: Entity,
    pub minimize: Entity,
    pub maximize: Entity,
    pub close: Entity,
    pub grip: Entity,
}
