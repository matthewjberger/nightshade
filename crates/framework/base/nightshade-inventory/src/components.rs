use crate::data::Slot;

#[derive(Default, Clone)]
pub struct Slots(pub Vec<Slot>);

#[derive(Default, Clone, Copy)]
pub struct Selected(pub usize);
