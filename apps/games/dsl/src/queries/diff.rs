use crate::commands::tape::advance;
use crate::data::cell::Cell;
use crate::data::change::{Change, alike};
use crate::resources::{Frame, Snapshot, Store, Tape};

pub fn measure(last: &Snapshot, store: &Store) -> Change {
    let mut change = Change {
        width: store.rows.len(),
        alive: store.alive.clone(),
        free: store.free.clone(),
        spare_texts: store.spare_texts.clone(),
        spare_lists: store.spare_lists.clone(),
        seed: store.seed,
        frame: store.frame,
        started: store.started,
        ..Default::default()
    };
    for key in 0..store.banks.len() {
        let bank = &store.banks[key];
        let held = last.banks.get(key);
        let words = bank
            .present
            .len()
            .max(held.map(|one| one.present.len()).unwrap_or_default());
        for word in 0..words {
            let now = bank.present.get(word).copied().unwrap_or_default();
            let was = held
                .and_then(|one| one.present.get(word))
                .copied()
                .unwrap_or_default();
            let mut bits = now | was;
            while bits != 0 {
                let at = bits.trailing_zeros();
                bits &= bits - 1;
                let row = ((word as u32) << 6) | at;
                let mine = now & (1 << at) != 0;
                let theirs = was & (1 << at) != 0;
                if !mine {
                    if theirs {
                        change.cleared.push((key as u16, row));
                    }
                    continue;
                }
                let value = bank.read(row);
                let before = match held {
                    Some(one) if theirs => one.read(row),
                    _ => Cell::nothing(),
                };
                if !theirs || !alike(&before, &value) {
                    change.columns.push((key as u16, row, value));
                }
            }
        }
    }
    for slot in 0..store.fields.len() {
        let value = store.fields[slot];
        let before = last.fields.get(slot).copied().unwrap_or_else(Cell::nothing);
        if !alike(&before, &value) {
            change.fields.push((slot as u16, value));
        }
    }
    for slot in 0..store.kept_texts.len() {
        match last.kept_texts.get(slot) {
            Some(before) if before == &store.kept_texts[slot] => {}
            _ => change
                .texts
                .push((slot as u32, store.kept_texts[slot].clone())),
        }
    }
    for slot in 0..store.kept_lists.len() {
        let items = &store.kept_lists[slot];
        let same = last.kept_lists.get(slot).is_some_and(|before| {
            before.len() == items.len()
                && before
                    .iter()
                    .zip(items.iter())
                    .all(|(one, other)| alike(one, other))
        });
        if !same {
            change.lists.push((slot as u32, items.clone()));
        }
    }
    for seat in 0..store.lives.len() {
        let life = store.lives[seat];
        if last.lives.get(seat).copied() != Some(life) {
            change.lives.push((seat as u32, life));
        }
    }
    change
}

pub fn read(tape: &Tape, at: usize) -> Option<Snapshot> {
    let mut start = at.min(tape.frames.len().checked_sub(1)?);
    while !matches!(tape.frames.get(start), Some(Frame::Base(_))) {
        start = start.checked_sub(1)?;
    }
    let Some(Frame::Base(base)) = tape.frames.get(start) else {
        return None;
    };
    let mut held = base.clone();
    for index in start + 1..=at.min(tape.frames.len() - 1) {
        if let Some(Frame::Step(change)) = tape.frames.get(index) {
            advance(&mut held, change);
        }
    }
    Some(held)
}
