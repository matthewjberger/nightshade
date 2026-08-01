use crate::data::ask::Ask;
use crate::data::cell::{Cell, Tag};
use crate::data::plan::{Op, Unit};
use crate::data::{BinaryOperator, UnaryOperator};
use crate::resources::{Bench, Machine, Sheet, Store};

pub const CHUNK: usize = 1024;
pub const CREW_FLOOR: usize = 4;
pub const CREW_LIMIT: usize = 8;

fn sum(base: f32, delta: f32) -> f32 {
    base + delta
}

fn blend(held: Cell, index: usize, value: f32) -> Cell {
    let mut parts = [0.0; 4];
    let mut width = 0usize;
    if held.tag == Tag::Vector {
        parts = held.parts;
        width = held.width as usize;
    }
    let target = index.min(3);
    parts[target] = value;
    Cell {
        tag: Tag::Vector,
        width: width.max(target + 1).min(4) as u8,
        slot: 0,
        parts,
    }
}

fn only(index: u8, value: f32) -> Cell {
    let mut parts = [0.0; 4];
    let at = (index as usize).min(3);
    parts[at] = value;
    Cell {
        tag: Tag::Vector,
        width: (at + 1) as u8,
        slot: 0,
        parts,
    }
}

fn join(operator: BinaryOperator, left: Cell, right: Cell) -> Cell {
    match operator {
        BinaryOperator::And => Cell::truth(left.truthy() && right.truthy()),
        BinaryOperator::Or => Cell::truth(left.truthy() || right.truthy()),
        BinaryOperator::Equal => Cell::truth(left.same(&right)),
        BinaryOperator::NotEqual => Cell::truth(!left.same(&right)),
        BinaryOperator::Less => Cell::truth(left.value() < right.value()),
        BinaryOperator::LessOrEqual => Cell::truth(left.value() <= right.value()),
        BinaryOperator::Greater => Cell::truth(left.value() > right.value()),
        BinaryOperator::GreaterOrEqual => Cell::truth(left.value() >= right.value()),
        BinaryOperator::Add => left.spread(&right, |first, second| first + second),
        BinaryOperator::Subtract => left.spread(&right, |first, second| first - second),
        BinaryOperator::Multiply => left.spread(&right, |first, second| first * second),
        BinaryOperator::Divide => {
            left.spread(
                &right,
                |first, second| {
                    if second == 0.0 { 0.0 } else { first / second }
                },
            )
        }
        BinaryOperator::Range => Cell::nothing(),
    }
}

fn alone(operator: UnaryOperator, value: Cell) -> Cell {
    match operator {
        UnaryOperator::Negate => Cell::number(0.0 - value.value()),
        UnaryOperator::Not => Cell::truth(!value.truthy()),
    }
}

fn reach(which: u8, found: &[Cell; 4]) -> Cell {
    let first = found[0].value();
    let second = found[1].value();
    let third = found[2].value();
    match which {
        0 => Cell::number(first.abs()),
        1 => Cell::number(if first < 0.0 { -1.0 } else { 1.0 }),
        2 => Cell::number(first.min(second)),
        3 => Cell::number(first.max(second)),
        4 => Cell::number(first.clamp(second.min(third), third.max(second))),
        5 => Cell::number(first.floor()),
        6 => Cell::number(first.ceil()),
        7 => Cell::number(first.max(0.0).sqrt()),
        8 => Cell::number(first.sin()),
        9 => Cell::number(first.cos()),
        10 => {
            let value = found[0];
            let width = match value.tag {
                Tag::Vector => value.width as usize,
                _ => 1,
            };
            let total: f32 = (0..width).map(|index| value.axis(index).powi(2)).sum();
            Cell::number(total.sqrt())
        }
        14 => whole(first, second, |left, right| left.rem_euclid(right)),
        15 => whole(first, second, |left, right| left.div_euclid(right)),
        16 => bits(first, second, |left, right| left & right),
        17 => bits(first, second, |left, right| left | right),
        18 => bits(first, second, |left, right| left ^ right),
        19 => {
            let steps = second as i64;
            let base = first as i64;
            let moved = match steps < 0 {
                true => base >> (-steps).min(63),
                false => base << steps.min(63),
            };
            Cell::number(moved as f32)
        }
        _ => Cell::nothing(),
    }
}

fn whole(first: f32, second: f32, meet: fn(f32, f32) -> f32) -> Cell {
    if second == 0.0 {
        return Cell::number(0.0);
    }
    Cell::number(meet(first.trunc(), second.trunc()))
}

fn bits(first: f32, second: f32, meet: fn(i64, i64) -> i64) -> Cell {
    Cell::number(meet(first as i64, second as i64) as f32)
}

struct Gate {
    at: u32,
    held: Vec<bool>,
    other: Vec<bool>,
    waiting: bool,
}

pub fn spread(store: &mut Store, machine: &mut Machine, unit: &Unit, members: &[u32]) {
    let chunks: Vec<&[u32]> = members.chunks(CHUNK).collect();
    let hands = match chunks.len() >= CREW_FLOOR {
        true => std::thread::available_parallelism()
            .map(|held| held.get())
            .unwrap_or(1)
            .min(chunks.len())
            .min(CREW_LIMIT),
        false => 1,
    };
    machine.benches.resize_with(hands.max(1), Bench::default);
    let share = chunks.len().div_ceil(hands.max(1));
    if hands <= 1 {
        let bench = &mut machine.benches[0];
        bench.batch.clear();
        for chunk in &chunks {
            work(store, bench, unit, chunk);
        }
    } else {
        std::thread::scope(|crew| {
            for (index, bench) in machine.benches.iter_mut().enumerate() {
                let mine = &chunks
                    [(index * share).min(chunks.len())..((index + 1) * share).min(chunks.len())];
                let held: &Store = store;
                crew.spawn(move || {
                    bench.batch.clear();
                    for chunk in mine {
                        work(held, bench, unit, chunk);
                    }
                });
            }
        });
    }
    for bench in machine.benches.iter_mut() {
        machine.steps += std::mem::take(&mut bench.steps);
        store.batch.append(&mut bench.batch);
    }
}

fn work(store: &Store, bench: &mut Bench, unit: &Unit, rows: &[u32]) {
    let width = rows.len();
    let mut sheet = std::mem::take(&mut bench.sheet);
    sheet.ready(width);
    let mut stack: Vec<Vec<Cell>> = std::mem::take(&mut bench.lanes);
    let mut spare: Vec<Vec<Cell>> = std::mem::take(&mut bench.spare);
    let mut locals: Vec<Vec<Cell>> = std::mem::take(&mut bench.locals);
    locals.resize_with(unit.locals as usize, Vec::new);
    for held in locals.iter_mut() {
        held.clear();
        held.resize(width, Cell::nothing());
    }
    let mut mask = vec![true; width];
    let mut gates: Vec<Gate> = Vec::new();

    let take = |spare: &mut Vec<Vec<Cell>>| -> Vec<Cell> {
        let mut held = spare.pop().unwrap_or_default();
        held.clear();
        held.resize(width, Cell::nothing());
        held
    };

    let code = &unit.body.code;
    let mut at = 0usize;
    while at < code.len() {
        while let Some(gate) = gates.last()
            && gate.at as usize == at
        {
            mask.copy_from_slice(&gate.held);
            gates.pop();
        }
        let op = code[at];
        at += 1;
        bench.steps += 1;
        match op {
            Op::PushNumber(value) => {
                let mut held = take(&mut spare);
                held.fill(Cell::number(value));
                stack.push(held);
            }
            Op::PushTruth(value) => {
                let mut held = take(&mut spare);
                held.fill(Cell::truth(value));
                stack.push(held);
            }
            Op::PushNothing => {
                let mut held = take(&mut spare);
                held.fill(Cell::nothing());
                stack.push(held);
            }
            Op::PushField(slot) => {
                let mut held = take(&mut spare);
                let base = store.field(slot);
                for (index, value) in held.iter_mut().enumerate() {
                    *value = match sheet.field(slot, index) {
                        Some((held, true)) => held,
                        Some((held, false)) => base.spread(&held, sum),
                        None => base,
                    };
                }
                stack.push(held);
            }
            Op::PushLocal(slot) => {
                let mut held = take(&mut spare);
                match locals.get(slot as usize) {
                    Some(source) => held.copy_from_slice(source),
                    None => held.fill(Cell::nothing()),
                }
                stack.push(held);
            }
            Op::PopLocal(slot) => {
                let held = stack.pop().unwrap_or_default();
                if let Some(target) = locals.get_mut(slot as usize) {
                    for index in 0..width {
                        if mask[index] {
                            target[index] = held[index];
                        }
                    }
                }
                spare.push(held);
            }
            Op::PushColumn(key) | Op::PushNumberColumn(key) => {
                let mut held = take(&mut spare);
                for index in 0..width {
                    held[index] = read(store, &sheet, key, rows[index], index);
                }
                stack.push(held);
            }
            Op::Axis(index) => {
                let Some(held) = stack.last_mut() else {
                    continue;
                };
                for value in held.iter_mut() {
                    *value = Cell::number(value.axis(index as usize));
                }
            }
            Op::Unary(operator) => {
                let Some(held) = stack.last_mut() else {
                    continue;
                };
                for value in held.iter_mut() {
                    *value = alone(operator, *value);
                }
            }
            Op::Binary(operator) => {
                let right = stack.pop().unwrap_or_default();
                let Some(left) = stack.last_mut() else {
                    spare.push(right);
                    continue;
                };
                for index in 0..width {
                    left[index] = join(operator, left[index], right[index]);
                }
                spare.push(right);
            }
            Op::MixNumber(operator, value) => {
                let right = Cell::number(value);
                let Some(left) = stack.last_mut() else {
                    continue;
                };
                for value in left.iter_mut() {
                    *value = join(operator, *value, right);
                }
            }
            Op::MixField(operator, slot) => {
                let base = store.field(slot);
                let Some(left) = stack.last_mut() else {
                    continue;
                };
                for (index, value) in left.iter_mut().enumerate() {
                    let right = match sheet.field(slot, index) {
                        Some((held, true)) => held,
                        Some((held, false)) => base.spread(&held, sum),
                        None => base,
                    };
                    *value = join(operator, *value, right);
                }
            }
            Op::MixColumn(operator, key) => {
                let Some(left) = stack.last_mut() else {
                    continue;
                };
                for index in 0..width {
                    let right = read(store, &sheet, key, rows[index], index);
                    left[index] = join(operator, left[index], right);
                }
            }
            Op::IsSome => {
                let Some(held) = stack.last_mut() else {
                    continue;
                };
                for value in held.iter_mut() {
                    *value = Cell::truth(value.tag != Tag::Nothing);
                }
            }
            Op::MakeVector(parts) => {
                let mut found: Vec<Vec<Cell>> = Vec::new();
                for _ in 0..parts {
                    found.push(stack.pop().unwrap_or_default());
                }
                found.reverse();
                let mut held = take(&mut spare);
                for index in 0..width {
                    let mut made = [0.0; 4];
                    for (slot, lane) in found.iter().enumerate().take(4) {
                        made[slot] = lane[index].value();
                    }
                    held[index] = Cell {
                        tag: Tag::Vector,
                        width: parts.min(4),
                        slot: 0,
                        parts: made,
                    };
                }
                stack.push(held);
                spare.extend(found);
            }
            Op::Builtin(which, count) => {
                let mut found: Vec<Vec<Cell>> = Vec::new();
                for _ in 0..count {
                    found.push(stack.pop().unwrap_or_default());
                }
                found.reverse();
                let mut held = take(&mut spare);
                for index in 0..width {
                    let mut given = [Cell::nothing(); 4];
                    for (slot, lane) in found.iter().enumerate().take(4) {
                        given[slot] = lane[index];
                    }
                    held[index] = reach(which, &given);
                }
                stack.push(held);
                spare.extend(found);
            }
            Op::StoreColumn(key) | Op::StoreNumberColumn(key) => {
                let held = stack.pop().unwrap_or_default();
                for index in 0..width {
                    if mask[index] {
                        sheet.put(key, index, held[index], true);
                    }
                }
                spare.push(held);
            }
            Op::AddColumn(key) | Op::AddNumberColumn(key) => {
                let held = stack.pop().unwrap_or_default();
                for index in 0..width {
                    if mask[index] {
                        sheet.put(key, index, held[index], false);
                    }
                }
                spare.push(held);
            }
            Op::StoreAxis(key, axis) => {
                let held = stack.pop().unwrap_or_default();
                for index in 0..width {
                    if mask[index] {
                        let base = read(store, &sheet, key, rows[index], index);
                        let made = blend(base, axis as usize, held[index].value());
                        sheet.put(key, index, made, true);
                    }
                }
                spare.push(held);
            }
            Op::AddAxis(key, axis) => {
                let held = stack.pop().unwrap_or_default();
                for index in 0..width {
                    if mask[index] {
                        sheet.put(key, index, only(axis, held[index].value()), false);
                    }
                }
                spare.push(held);
            }
            Op::StoreField(slot) => {
                let held = stack.pop().unwrap_or_default();
                for index in 0..width {
                    if mask[index] {
                        sheet.put_field(slot, index, held[index], true);
                    }
                }
                spare.push(held);
            }
            Op::AddField(slot) => {
                let held = stack.pop().unwrap_or_default();
                for index in 0..width {
                    if mask[index] {
                        sheet.put_field(slot, index, held[index], false);
                    }
                }
                spare.push(held);
            }
            Op::StoreAxisField(slot, axis) => {
                let held = stack.pop().unwrap_or_default();
                for index in 0..width {
                    if mask[index] {
                        let base = store.field(slot);
                        let base = match sheet.field(slot, index) {
                            Some((value, true)) => value,
                            Some((value, false)) => base.spread(&value, sum),
                            None => base,
                        };
                        let made = blend(base, axis as usize, held[index].value());
                        sheet.put_field(slot, index, made, true);
                    }
                }
                spare.push(held);
            }
            Op::JumpIfFalse(target) => {
                let held = stack.pop().unwrap_or_default();
                let mut other = spare.pop().unwrap_or_default();
                other.clear();
                other.resize(width, Cell::nothing());
                let keep = mask.clone();
                let mut turned = vec![false; width];
                for index in 0..width {
                    turned[index] = mask[index] && !held[index].truthy();
                    mask[index] = mask[index] && held[index].truthy();
                }
                gates.push(Gate {
                    at: target,
                    held: keep,
                    other: turned,
                    waiting: true,
                });
                spare.push(held);
                spare.push(other);
            }
            Op::Jump(target) => {
                let Some(gate) = gates.last_mut() else {
                    continue;
                };
                if !gate.waiting || gate.at as usize != at {
                    continue;
                }
                gate.waiting = false;
                gate.at = target;
                mask.copy_from_slice(&gate.other);
            }
            Op::Pop => {
                if let Some(held) = stack.pop() {
                    spare.push(held);
                }
            }
            _ => {}
        }
    }

    post(&mut bench.batch, &sheet, rows);

    spare.append(&mut stack);
    bench.lanes = stack;
    bench.spare = spare;
    bench.locals = locals;
    bench.sheet = sheet;
}

fn read(store: &Store, sheet: &Sheet, key: u16, row: u32, index: usize) -> Cell {
    match sheet.column(key, index) {
        Some((value, true)) => value,
        Some((value, false)) => store.read(key, row).spread(&value, sum),
        None => store.read(key, row),
    }
}

fn post(batch: &mut Vec<(u32, Ask)>, sheet: &Sheet, rows: &[u32]) {
    let mut keys: Vec<u16> = sheet.touched.clone();
    keys.sort_unstable();
    let mut slots: Vec<u16> = sheet.marked.clone();
    slots.sort_unstable();
    for (index, row) in rows.iter().enumerate() {
        for key in &keys {
            let Some((value, absolute)) = sheet.column(*key, index) else {
                continue;
            };
            batch.push((
                *row,
                match absolute {
                    true => Ask::SetColumn {
                        key: *key,
                        row: *row,
                        value,
                    },
                    false => Ask::AddColumn {
                        key: *key,
                        row: *row,
                        value,
                    },
                },
            ));
        }
        for slot in &slots {
            let Some((value, absolute)) = sheet.field(*slot, index) else {
                continue;
            };
            batch.push((
                *row,
                match absolute {
                    true => Ask::SetField { slot: *slot, value },
                    false => Ask::AddField { slot: *slot, value },
                },
            ));
        }
    }
}
