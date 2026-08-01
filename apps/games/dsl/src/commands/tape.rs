use crate::data::cell::Cell;
use crate::data::change::Change;
use crate::queries::diff::{measure, read};
use crate::resources::{Bank, Frame, Snapshot, Store, Tape};

pub const BASE: usize = 60;

pub fn advance(held: &mut Snapshot, change: &Change) {
    held.width = change.width;
    held.alive = change.alive.clone();
    held.free = change.free.clone();
    held.spare_texts = change.spare_texts.clone();
    held.spare_lists = change.spare_lists.clone();
    held.seed = change.seed;
    held.frame = change.frame;
    held.started = change.started;
    for (seat, life) in &change.lives {
        if held.lives.len() <= *seat as usize {
            held.lives.resize(*seat as usize + 1, 0);
        }
        held.lives[*seat as usize] = *life;
    }
    for (key, row, value) in &change.columns {
        if held.banks.len() <= *key as usize {
            held.banks.resize_with(*key as usize + 1, Bank::default);
        }
        held.banks[*key as usize].put(*row, *value, change.width);
    }
    for (key, row) in &change.cleared {
        if let Some(bank) = held.banks.get_mut(*key as usize) {
            bank.unmark(*row);
        }
    }
    for (slot, value) in &change.fields {
        if held.fields.len() <= *slot as usize {
            held.fields.resize(*slot as usize + 1, Cell::nothing());
        }
        held.fields[*slot as usize] = *value;
    }
    for (slot, line) in &change.texts {
        if held.kept_texts.len() <= *slot as usize {
            held.kept_texts.resize(*slot as usize + 1, String::new());
        }
        held.kept_texts[*slot as usize].clear();
        held.kept_texts[*slot as usize].push_str(line);
    }
    for (slot, items) in &change.lists {
        if held.kept_lists.len() <= *slot as usize {
            held.kept_lists.resize(*slot as usize + 1, Vec::new());
        }
        held.kept_lists[*slot as usize].clear();
        held.kept_lists[*slot as usize].extend_from_slice(items);
    }
}

pub fn record(tape: &mut Tape, store: &Store, cap: usize) {
    if tape.frames.is_empty() || tape.since >= BASE {
        let held = store.capture();
        tape.last = held.clone();
        tape.frames.push(Frame::Base(held));
        tape.since = 0;
    } else {
        let change = measure(&tape.last, store);
        advance(&mut tape.last, &change);
        tape.frames.push(Frame::Step(change));
        tape.since += 1;
    }
    if tape.frames.len() > cap + BASE {
        let mut cut = tape.frames.len() - cap;
        while cut < tape.frames.len() && !matches!(tape.frames[cut], Frame::Base(_)) {
            cut += 1;
        }
        if cut < tape.frames.len() {
            tape.frames.drain(..cut);
        }
    }
}

pub fn rewind(tape: &mut Tape, at: usize) {
    tape.frames.truncate(at + 1);
    tape.since = 0;
    let mut start = at;
    while start > 0 && !matches!(tape.frames.get(start), Some(Frame::Base(_))) {
        start -= 1;
        tape.since += 1;
    }
    tape.last = read(tape, at).unwrap_or_default();
}
