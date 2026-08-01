use crate::{Entity, Mask};

#[inline]
pub const fn tick_is_newer(tick: u32, since_tick: u32) -> bool {
    tick.wrapping_sub(since_tick) as i32 > 0
}

pub const STRUCTURAL_LOG_CAPACITY: usize = 262_144;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum StructuralChangeKind {
    Spawned,
    Despawned,
    ComponentsAdded,
    ComponentsRemoved,
    TagsAdded,
    TagsRemoved,
}

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StructuralChange {
    pub sequence: u64,
    pub entity: Entity,
    pub kind: StructuralChangeKind,
    pub(crate) mask: Mask,
}
