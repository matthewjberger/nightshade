use crate::data::Kind;

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub struct NodeKind(pub Kind);

#[derive(Default, Clone, Copy, Debug)]
pub struct NodeValue(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct NodeSlot(pub u32);
