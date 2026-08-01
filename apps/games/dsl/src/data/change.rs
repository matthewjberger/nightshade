use crate::data::cell::Cell;
use crate::resources::Marks;

#[derive(Clone, Default)]
pub struct Change {
    pub columns: Vec<(u16, u32, Cell)>,
    pub cleared: Vec<(u16, u32)>,
    pub fields: Vec<(u16, Cell)>,
    pub texts: Vec<(u32, String)>,
    pub lists: Vec<(u32, Vec<Cell>)>,
    pub lives: Vec<(u32, u32)>,
    pub width: usize,
    pub alive: Marks,
    pub free: Vec<u32>,
    pub spare_texts: Vec<u32>,
    pub spare_lists: Vec<u32>,
    pub seed: u32,
    pub frame: u32,
    pub started: bool,
}

pub fn alike(left: &Cell, right: &Cell) -> bool {
    left.tag == right.tag
        && left.width == right.width
        && left.slot == right.slot
        && left.parts == right.parts
}
