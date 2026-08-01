use crate::data::cell::{Cell, KEPT, Tag};
use crate::resources::{Machine, Store};
pub mod compile;
pub mod describe;
pub mod diff;
pub mod edit;
pub mod lesson;
pub mod lex;
pub mod parse;
pub mod plan;
pub mod shape;
pub mod write;

pub fn show(store: &Store, machine: &Machine, value: Cell) -> String {
    match value.tag {
        Tag::Nothing => String::new(),
        Tag::Truth => (value.parts[0] != 0.0).to_string(),
        Tag::Number => {
            let held = value.parts[0];
            if (held - held.round()).abs() < 1.0e-4 {
                format!("{}", held.round() as i64)
            } else {
                format!("{held:.2}")
            }
        }
        Tag::Text if value.slot & KEPT != 0 => store.kept_text(value.slot).to_string(),
        Tag::Text => machine
            .texts
            .get(value.slot as usize)
            .cloned()
            .unwrap_or_default(),
        Tag::Vector => (0..value.width as usize)
            .map(|index| format!("{:.2}", value.parts[index]))
            .collect::<Vec<_>>()
            .join(", "),
        Tag::List => (0..list_len(store, machine, value))
            .map(|index| show(store, machine, list_at(store, machine, value, index)))
            .collect::<Vec<_>>()
            .join(", "),
        Tag::Handle => value.slot.to_string(),
    }
}

pub fn list_len(store: &Store, machine: &Machine, value: Cell) -> usize {
    match value.tag {
        Tag::List if value.slot & KEPT != 0 => store.kept_list(value.slot).len(),
        Tag::List => machine
            .lists
            .get(value.slot as usize)
            .map(Vec::len)
            .unwrap_or_default(),
        Tag::Vector => value.width as usize,
        Tag::Nothing => 0,
        _ => 1,
    }
}

pub fn list_at(store: &Store, machine: &Machine, value: Cell, index: usize) -> Cell {
    match value.tag {
        Tag::List if value.slot & KEPT != 0 => store
            .kept_list(value.slot)
            .get(index)
            .copied()
            .unwrap_or_else(Cell::nothing),
        Tag::List => machine
            .lists
            .get(value.slot as usize)
            .and_then(|held| held.get(index))
            .copied()
            .unwrap_or_else(Cell::nothing),
        Tag::Vector => Cell::number(value.axis(index)),
        Tag::Nothing => Cell::nothing(),
        _ if index == 0 => value,
        _ => Cell::nothing(),
    }
}
