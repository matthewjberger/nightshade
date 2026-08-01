use crate::data::cell::{Cell, Tag};
use crate::data::plan::Plan;
use crate::resources::Store;

pub const MOST_ROWS: usize = 6;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Holder,
    Shared,
    Signal,
}

#[derive(Clone)]
pub struct Card {
    pub name: String,
    pub kind: Kind,
    pub rows: Vec<(String, String)>,
}

pub fn cards(plan: &Plan, store: &Store) -> Vec<Card> {
    let mut held: Vec<Card> = Vec::new();

    for (key, name) in plan.columns.iter().enumerate() {
        let mut count = 0;
        let mut first = Cell::nothing();
        for row in 0..store.rows.len() as u32 {
            if !store.living(row) || !store.holds(key as u16, row) {
                continue;
            }
            if count == 0 {
                first = store.read(key as u16, row);
            }
            count += 1;
        }
        let mut rows = vec![("on".to_string(), format!("{count} things"))];
        if count > 0 {
            rows.push(("holds".to_string(), shape_of(&first)));
            rows.push(("one".to_string(), show(store, &first)));
        }
        held.push(Card {
            name: name.clone(),
            kind: Kind::Holder,
            rows,
        });
    }

    let mut blocks: Vec<String> = Vec::new();
    for (block, _) in &plan.fields {
        if !blocks.contains(block) {
            blocks.push(block.clone());
        }
    }
    for block in blocks {
        let mut rows = Vec::new();
        let mut more = 0;
        for (slot, (held_block, field)) in plan.fields.iter().enumerate() {
            if held_block != &block {
                continue;
            }
            if rows.len() == MOST_ROWS {
                more += 1;
                continue;
            }
            rows.push((field.clone(), show(store, &store.field(slot as u16))));
        }
        if more > 0 {
            rows.push(("and".to_string(), format!("{more} more")));
        }
        held.push(Card {
            name: block,
            kind: Kind::Shared,
            rows,
        });
    }

    for (at, name) in plan.events.iter().enumerate() {
        let count = store.events.get(at).map(|held| held.len()).unwrap_or(0);
        held.push(Card {
            name: name.clone(),
            kind: Kind::Signal,
            rows: vec![("this turn".to_string(), format!("{count}"))],
        });
    }

    held
}

pub fn shape_of(cell: &Cell) -> String {
    match cell.tag {
        Tag::Nothing => "nothing".to_string(),
        Tag::Truth => "yes or no".to_string(),
        Tag::Number => "a number".to_string(),
        Tag::Vector => format!("{} numbers", cell.width.max(1)),
        Tag::Handle => "a thing".to_string(),
        Tag::Text => "words".to_string(),
        Tag::List => "a list".to_string(),
    }
}

pub fn show(store: &Store, cell: &Cell) -> String {
    match cell.tag {
        Tag::Nothing => "nothing".to_string(),
        Tag::Truth => {
            if cell.truthy() {
                "yes".to_string()
            } else {
                "no".to_string()
            }
        }
        Tag::Number => number(cell.value()),
        Tag::Vector => {
            let parts: Vec<String> = (0..cell.width.max(1))
                .map(|at| number(cell.axis(at as usize)))
                .collect();
            format!("({})", parts.join(", "))
        }
        Tag::Handle => match cell.row() {
            Some(row) => format!("thing {row}"),
            None => "no thing".to_string(),
        },
        Tag::Text => store.kept_text(cell.slot).to_string(),
        Tag::List => format!("{} items", store.kept_list(cell.slot).len()),
    }
}

fn number(value: f32) -> String {
    if value == value.trunc() && value.abs() < 1.0e9 {
        return format!("{}", value as i64);
    }
    let held = format!("{value:.2}");
    held.trim_end_matches('0').trim_end_matches('.').to_string()
}
