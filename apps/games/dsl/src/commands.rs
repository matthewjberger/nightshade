use crate::queries::show;
use crate::queries::{list_at, list_len};
pub mod edit;
pub mod name;
pub mod tape;
pub mod wide;

use crate::data::ask::Ask;
use crate::data::cell::{Cell, KEPT, Tag};
use crate::data::kept::{self, Kept};
use crate::data::plan::{Op, Pick, Plan, Routine, Unit};
use crate::data::{BinaryOperator, Cardinality, UnaryOperator};
use crate::resources::{Machine, Store};
use nightshade_save::commands::save_write;
use nightshade_save::queries::save_read;

pub fn seed(plan: &Plan, store: &mut Store, machine: &mut Machine) {
    store.ready(
        plan.columns.len(),
        plan.fields.len(),
        plan.events.len(),
        plan.rosters,
        &plan.holds,
    );
    for (slot, routine) in &plan.seeds {
        machine.clear();
        let value = execute(plan, store, machine, routine, 0);
        let held = settle(store, machine, value);
        store.set_field(*slot, held);
    }
    machine.clear();
    machine.shadow.ready(&plan.holds);
}

fn sum(base: f32, delta: f32) -> f32 {
    base + delta
}

fn peek(store: &Store, machine: &Machine, key: u16, row: u32) -> Cell {
    match machine.shadow.column(key, row) {
        Some((value, true)) => value,
        Some((value, false)) => store.read(key, row).spread(&value, sum),
        None => store.read(key, row),
    }
}

fn number(store: &Store, machine: &Machine, key: u16, row: u32) -> f32 {
    match machine.shadow.number(key, row) {
        Some((value, true)) => value,
        Some((value, false)) => store.read(key, row).value() + value,
        None => store.read(key, row).value(),
    }
}

fn peek_field(store: &Store, machine: &Machine, slot: u16) -> Cell {
    match machine.shadow.field(slot) {
        Some((value, true)) => value,
        Some((value, false)) => store.field(slot).spread(&value, sum),
        None => store.field(slot),
    }
}

fn mark(store: &mut Store, machine: &mut Machine, key: u16, row: u32, value: Cell, absolute: bool) {
    let (held, kind) = match machine.shadow.column(key, row) {
        Some((previous, kind)) => {
            store.release(previous);
            if absolute {
                (value, true)
            } else {
                (previous.spread(&value, sum), kind)
            }
        }
        None => (value, absolute),
    };
    machine.shadow.put_column(key, row, held, kind);
}

fn mark_field(store: &mut Store, machine: &mut Machine, slot: u16, value: Cell, absolute: bool) {
    let (held, kind) = match machine.shadow.field(slot) {
        Some((previous, kind)) => {
            store.release(previous);
            if absolute {
                (value, true)
            } else {
                (previous.spread(&value, sum), kind)
            }
        }
        None => (value, absolute),
    };
    machine.shadow.put_field(slot, held, kind);
}

fn own_field(store: &mut Store, machine: &mut Machine, slot: u16) -> Cell {
    if let Some((cell, true)) = machine.shadow.field(slot)
        && cell.tag == Tag::List
        && cell.slot & KEPT != 0
    {
        return cell;
    }
    let base = peek_field(store, machine, slot);
    let made = settle(store, machine, base);
    mark_field(store, machine, slot, made, true);
    made
}

fn own_column(store: &mut Store, machine: &mut Machine, key: u16, row: u32) -> Cell {
    if let Some((cell, true)) = machine.shadow.column(key, row)
        && cell.tag == Tag::List
        && cell.slot & KEPT != 0
    {
        return cell;
    }
    let base = peek(store, machine, key, row);
    let made = settle(store, machine, base);
    mark(store, machine, key, row, made, true);
    made
}

fn poke(store: &mut Store, held: Cell, at: usize, value: Cell) {
    if held.tag != Tag::List || held.slot & KEPT == 0 {
        return;
    }
    let Some(items) = store.kept_lists.get_mut((held.slot & !KEPT) as usize) else {
        return;
    };
    while items.len() <= at {
        items.push(Cell::nothing());
    }
    items[at] = value;
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

fn post(store: &mut Store, machine: &mut Machine, at: u32) {
    let mut touched = std::mem::take(&mut machine.shadow.touched);
    touched.sort_unstable();
    for key in &touched {
        let Some(column) = machine.shadow.columns.get(*key as usize) else {
            continue;
        };
        for index in 0..column.rows.len() {
            let row = column.rows[index];
            let value = column.held(index);
            store.batch.push((
                at,
                if column.absolute[index] {
                    Ask::SetColumn {
                        key: *key,
                        row,
                        value,
                    }
                } else {
                    Ask::AddColumn {
                        key: *key,
                        row,
                        value,
                    }
                },
            ));
        }
    }
    for key in &touched {
        if let Some(column) = machine.shadow.columns.get_mut(*key as usize) {
            column.clear();
        }
    }
    touched.clear();
    machine.shadow.touched = touched;
    let mut slots = std::mem::take(&mut machine.shadow.slots);
    slots.sort_unstable();
    for slot in &slots {
        let value = machine.shadow.values[*slot as usize];
        let absolute = machine.shadow.absolute[*slot as usize];
        machine.shadow.present.unmark(*slot as u32);
        store.batch.push((
            at,
            if absolute {
                Ask::SetField { slot: *slot, value }
            } else {
                Ask::AddField { slot: *slot, value }
            },
        ));
    }
    slots.clear();
    machine.shadow.slots = slots;
    let emits = std::mem::take(&mut machine.shadow.emits);
    for (key, value) in emits {
        store.batch.push((at, Ask::Emit { key, value }));
    }
    let keeps = std::mem::take(&mut machine.shadow.keeps);
    for (site, back) in keeps {
        store.batch.push((
            at,
            if back {
                Ask::Recall { site }
            } else {
                Ask::Keep { site }
            },
        ));
    }
}

fn parts(ask: &Ask) -> (u16, u8, u32) {
    match ask {
        Ask::SetColumn { key, row, .. } => (*key, 0, *row),
        Ask::AddColumn { key, row, .. } => (*key, 1, *row),
        _ => (u16::MAX, 2, 0),
    }
}

fn borne(ask: &Ask) -> Cell {
    match ask {
        Ask::SetColumn { value, .. } | Ask::AddColumn { value, .. } => *value,
        _ => Cell::nothing(),
    }
}

fn fold(store: &mut Store, segment: &mut Vec<Ask>) {
    if segment.is_empty() {
        return;
    }
    segment.sort_by_key(|ask| {
        let (key, kind, _) = parts(ask);
        (key, kind)
    });
    let mut index = 0usize;
    while index < segment.len() {
        let (key, kind, row) = parts(&segment[index]);
        let mut end = index + 1;
        while end < segment.len() {
            let (other, sort, at) = parts(&segment[end]);
            if other != key || sort != kind || at != row + (end - index) as u32 {
                break;
            }
            end += 1;
        }
        if end - index >= 2 {
            let at = store.spans.len() as u32;
            for one in &segment[index..end] {
                store.spans.push(borne(one));
            }
            let count = (end - index) as u32;
            store.asks.push(if kind == 0 {
                Ask::SetSpan {
                    key,
                    row,
                    at,
                    count,
                }
            } else {
                Ask::AddSpan {
                    key,
                    row,
                    at,
                    count,
                }
            });
        } else {
            for one in &segment[index..end] {
                store.asks.push(*one);
            }
        }
        index = end;
    }
    segment.clear();
}

fn seal(store: &mut Store) {
    let mut batch = std::mem::take(&mut store.batch);
    batch.sort_by_key(|(at, _)| *at);
    let mut segment = std::mem::take(&mut store.segment);
    for (_, ask) in batch.drain(..) {
        match ask {
            Ask::SetColumn { .. } | Ask::AddColumn { .. } => segment.push(ask),
            _ => {
                fold(store, &mut segment);
                store.asks.push(ask);
            }
        }
    }
    fold(store, &mut segment);
    store.batch = batch;
    store.segment = segment;
}

fn keep(plan: &Plan, store: &mut Store, site: u16) {
    let Some(held) = plan.kept.get(site as usize) else {
        return;
    };
    let mut kept = save_read::<Kept>(0).unwrap_or_default();
    for (slot, name) in held {
        let value = store.field(*slot);
        let line = match value.tag {
            Tag::Text => show(store, &Machine::default(), value),
            _ => String::new(),
        };
        let tag = value.tag as u8;
        match kept::place(&kept, name) {
            Some(at) => {
                kept.tags[at] = tag;
                kept.widths[at] = value.width;
                kept.parts[at] = value.parts;
                kept.lines[at] = line;
            }
            None => {
                kept.names.push(name.clone());
                kept.tags.push(tag);
                kept.widths.push(value.width);
                kept.parts.push(value.parts);
                kept.lines.push(line);
            }
        }
    }
    save_write::<Kept>(0, "description", &kept);
}

fn recall(plan: &Plan, store: &mut Store, site: u16) {
    let Some(held) = plan.kept.get(site as usize) else {
        return;
    };
    let Some(kept) = save_read::<Kept>(0) else {
        return;
    };
    for (slot, name) in held {
        let Some(at) = kept::place(&kept, name) else {
            continue;
        };
        let tag = match kept.tags[at] {
            1 => Tag::Truth,
            2 => Tag::Number,
            3 => Tag::Vector,
            5 => Tag::Text,
            _ => Tag::Nothing,
        };
        let value = match tag {
            Tag::Text => {
                let line = kept.lines[at].clone();
                Cell::text(store.keep_text(&line))
            }
            Tag::Nothing => Cell::nothing(),
            _ => Cell {
                tag,
                width: kept.widths[at],
                slot: 0,
                parts: kept.parts[at],
            },
        };
        store.set_field(*slot, value);
    }
}

pub fn apply(plan: &Plan, store: &mut Store) {
    let held = std::mem::take(&mut store.asks);
    store.commands += held.len() as u64;
    for one in held {
        match one {
            Ask::SetColumn { key, row, value } => store.write(key, row, value),
            Ask::AddColumn { key, row, value } => {
                let base = store.read(key, row);
                store.write(key, row, base.spread(&value, sum));
            }
            Ask::SetField { slot, value } => store.set_field(slot, value),
            Ask::AddField { slot, value } => {
                let base = store.field(slot);
                store.set_field(slot, base.spread(&value, sum));
            }
            Ask::Emit { key, value } => {
                if let Some(events) = store.events.get_mut(key as usize) {
                    events.push(value);
                }
            }
            Ask::SetSpan {
                key,
                row,
                at,
                count,
            } => {
                for step in 0..count {
                    let value = store.spans[(at + step) as usize];
                    store.write(key, row + step, value);
                }
            }
            Ask::AddSpan {
                key,
                row,
                at,
                count,
            } => {
                for step in 0..count {
                    let value = store.spans[(at + step) as usize];
                    let seat = row + step;
                    let base = store.read(key, seat);
                    store.write(key, seat, base.spread(&value, sum));
                }
            }
            Ask::ClearBank { key } => store.drop_bank(key),
            Ask::Keep { site } => keep(plan, store, site),
            Ask::Recall { site } => recall(plan, store, site),
        }
    }
    store.spans.clear();
}

pub fn enabled(plan: &Plan, store: &mut Store, machine: &mut Machine, unit: &Unit) -> bool {
    for routine in &unit.enable {
        machine.clear();
        if !execute(plan, store, machine, routine, 0).truthy() {
            return false;
        }
    }
    true
}

pub fn run(plan: &Plan, store: &mut Store, machine: &mut Machine, unit: &Unit) {
    machine.clear();
    machine.locals.resize(unit.locals as usize, Cell::nothing());

    let mut over: Option<(u16, Vec<u32>)> = None;
    for pick in &unit.picks {
        let members = choose(plan, store, machine, pick);
        match pick.cardinality {
            Cardinality::One => {
                if let Some(first) = members.first() {
                    machine.locals[pick.label as usize] = store.held(*first);
                }
            }
            Cardinality::Set => {
                let held: Vec<Cell> = members.iter().map(|seat| store.held(*seat)).collect();
                let slot = machine.hold_list(held);
                machine.shared.mark(slot);
                machine.locals[pick.label as usize] = Cell::list(slot);
            }
            Cardinality::Every => over = Some((pick.label, members)),
        }
    }

    match over {
        Some((_, members)) if unit.wide => wide::spread(store, machine, unit, &members),
        Some((label, members)) => {
            for member in members {
                machine.stack.clear();
                machine.halted = false;
                machine.draws = 0;
                machine.row = Some(member);
                machine.locals[label as usize] = store.held(member);
                execute(plan, store, machine, &unit.body, 0);
                post(store, machine, member);
                if machine.halted {
                    break;
                }
            }
        }
        None => {
            machine.row = None;
            execute(plan, store, machine, &unit.body, 0);
            post(store, machine, u32::MAX);
        }
    }
    seal(store);
}

fn choose(plan: &Plan, store: &mut Store, machine: &mut Machine, pick: &Pick) -> Vec<u32> {
    let mut found = Vec::new();
    if pick.need.is_empty() {
        return found;
    }
    if let Some((stamp, held)) = store.rosters.get(pick.roster as usize)
        && *stamp == store.shape
    {
        return held.clone();
    }
    store.select(&pick.need, &pick.ban, &mut found);

    for (key, routine) in &pick.starts {
        for row in &found {
            if store.holds(*key, *row) {
                continue;
            }
            machine.stack.clear();
            let value = execute(plan, store, machine, routine, 0);
            let held = settle(store, machine, value);
            store.write(*key, *row, held);
        }
    }
    let shape = store.shape;
    if let Some(roster) = store.rosters.get_mut(pick.roster as usize) {
        roster.0 = shape;
        roster.1.clear();
        roster.1.extend_from_slice(&found);
    }
    found
}

fn settle(store: &mut Store, machine: &Machine, value: Cell) -> Cell {
    match value.tag {
        Tag::Text => {
            let held = if value.slot & KEPT != 0 {
                store.kept_text(value.slot).to_string()
            } else {
                machine
                    .texts
                    .get(value.slot as usize)
                    .cloned()
                    .unwrap_or_default()
            };
            Cell::text(store.keep_text(&held))
        }
        Tag::List => {
            let held: Vec<Cell> = if value.slot & KEPT != 0 {
                store.kept_list(value.slot).to_vec()
            } else {
                machine
                    .lists
                    .get(value.slot as usize)
                    .cloned()
                    .unwrap_or_default()
            };
            Cell::list(store.keep_list(&held))
        }
        _ => value,
    }
}

fn words<'a>(store: &'a Store, machine: &'a Machine, value: Cell) -> &'a str {
    match value.tag {
        Tag::Text if value.slot & KEPT != 0 => store.kept_text(value.slot),
        Tag::Text => machine
            .texts
            .get(value.slot as usize)
            .map(String::as_str)
            .unwrap_or_default(),
        _ => "",
    }
}

fn alike(store: &Store, machine: &Machine, left: Cell, right: Cell) -> bool {
    if left.tag == Tag::Text || right.tag == Tag::Text {
        return left.tag == right.tag
            && words(store, machine, left) == words(store, machine, right);
    }
    left.same(&right)
}

fn gather(store: &Store, machine: &Machine, value: Cell) -> Vec<Cell> {
    let width = list_len(store, machine, value);
    (0..width)
        .map(|index| list_at(store, machine, value, index))
        .collect()
}

fn execute(
    plan: &Plan,
    store: &mut Store,
    machine: &mut Machine,
    routine: &Routine,
    base: usize,
) -> Cell {
    let outer = machine.base;
    machine.base = base;
    let mut at = 0usize;
    while at < routine.code.len() {
        let op = routine.code[at];
        at += 1;
        machine.steps += 1;
        match op {
            Op::PushNumber(value) => machine.stack.push(Cell::number(value)),
            Op::PushTruth(value) => machine.stack.push(Cell::truth(value)),
            Op::PushText(slot) => {
                let text = plan.texts.get(slot as usize).cloned().unwrap_or_default();
                let held = machine.hold_text(text);
                machine.stack.push(Cell::text(held));
            }
            Op::PushNothing => machine.stack.push(Cell::nothing()),
            Op::PushLocal(slot) | Op::PushSet(slot) => {
                let value = machine
                    .locals
                    .get(machine.base + slot as usize)
                    .copied()
                    .unwrap_or_else(Cell::nothing);
                machine.stack.push(value);
            }
            Op::PopLocal(slot) => {
                let value = machine.stack.pop().unwrap_or_else(Cell::nothing);
                machine.share(value);
                let target = machine.base + slot as usize;
                if target < machine.locals.len() {
                    machine.locals[target] = value;
                }
            }
            Op::PushColumn(key) => {
                let value = match machine.row {
                    Some(row) => peek(store, machine, key, row),
                    None => Cell::nothing(),
                };
                machine.stack.push(value);
            }
            Op::PushNumberColumn(key) => {
                let value = match machine.row {
                    Some(row) => number(store, machine, key, row),
                    None => 0.0,
                };
                machine.stack.push(Cell::number(value));
            }
            Op::PushColumnOf(key) => {
                let subject = machine.stack.pop().unwrap_or_else(Cell::nothing);
                let value = match store.seat(&subject) {
                    Some(row) => peek(store, machine, key, row),
                    None => Cell::nothing(),
                };
                machine.stack.push(value);
            }
            Op::PushField(slot) => {
                let value = peek_field(store, machine, slot);
                machine.stack.push(value);
            }
            Op::PushEvents(key) => {
                let held = store.events.get(key as usize).cloned().unwrap_or_default();
                let slot = machine.hold_list(held);
                machine.stack.push(Cell::list(slot));
            }
            Op::Axis(index) => {
                let value = machine.stack.pop().unwrap_or_else(Cell::nothing);
                machine.stack.push(Cell::number(value.axis(index as usize)));
            }
            Op::Index => {
                let index = machine.stack.pop().unwrap_or_else(Cell::nothing);
                let value = machine.stack.pop().unwrap_or_else(Cell::nothing);
                let target = index.value().max(0.0) as usize;
                let found = list_at(store, machine, value, target);
                machine.stack.push(found);
            }
            Op::Unary(operator) => {
                let value = machine.stack.pop().unwrap_or_else(Cell::nothing);
                let made = match operator {
                    UnaryOperator::Negate => Cell::number(0.0).spread(&value, |_, right| -right),
                    UnaryOperator::Not => Cell::truth(!value.truthy()),
                };
                machine.stack.push(made);
            }
            Op::Binary(operator) => {
                let right = machine.stack.pop().unwrap_or_else(Cell::nothing);
                let left = machine.stack.pop().unwrap_or_else(Cell::nothing);
                let made = combine(store, machine, operator, left, right);
                machine.stack.push(made);
            }
            Op::MixNumber(operator, value) => {
                let left = machine.stack.pop().unwrap_or_else(Cell::nothing);
                let made = combine(store, machine, operator, left, Cell::number(value));
                machine.stack.push(made);
            }
            Op::MixField(operator, slot) => {
                let right = peek_field(store, machine, slot);
                let left = machine.stack.pop().unwrap_or_else(Cell::nothing);
                let made = combine(store, machine, operator, left, right);
                machine.stack.push(made);
            }
            Op::MixColumn(operator, key) => {
                let right = match machine.row {
                    Some(row) => peek(store, machine, key, row),
                    None => Cell::nothing(),
                };
                let left = machine.stack.pop().unwrap_or_else(Cell::nothing);
                let made = combine(store, machine, operator, left, right);
                machine.stack.push(made);
            }
            Op::MakeVector(width) => {
                let mut parts = [0.0; 4];
                for index in (0..width as usize).rev() {
                    let value = machine.stack.pop().unwrap_or_else(Cell::nothing);
                    if index < 4 {
                        parts[index] = value.value();
                    }
                }
                machine.stack.push(Cell {
                    tag: Tag::Vector,
                    width: width.min(4),
                    slot: 0,
                    parts,
                });
            }
            Op::MakeList(width) => {
                let mut items = vec![Cell::nothing(); width as usize];
                for index in (0..width as usize).rev() {
                    items[index] = machine.stack.pop().unwrap_or_else(Cell::nothing);
                }
                let slot = machine.hold_list(items);
                machine.stack.push(Cell::list(slot));
            }
            Op::IsSome => {
                let value = machine.stack.pop().unwrap_or_else(Cell::nothing);
                machine.stack.push(Cell::truth(value.tag != Tag::Nothing));
            }
            Op::Builtin(which, count) => {
                let mut found = [Cell::nothing(); 4];
                for index in (0..count as usize).rev() {
                    let value = machine.stack.pop().unwrap_or_else(Cell::nothing);
                    if index < 4 {
                        found[index] = value;
                    }
                }
                let made = if which == 20 {
                    let width = found[1].value().max(0.0) as usize;
                    let slot = machine.hold_list(vec![found[0]; width]);
                    Cell::list(slot)
                } else if which == 12 {
                    let step = machine.draws;
                    machine.draws += 1;
                    Cell::number(store.draw(machine.row.unwrap_or(u32::MAX), step))
                } else {
                    builtin(store, machine, which, &found)
                };
                machine.stack.push(made);
            }
            Op::CallQuery(index, count) => {
                let Some((inner, locals)) = plan.queries.get(index as usize) else {
                    machine.stack.push(Cell::nothing());
                    continue;
                };
                let mark = machine.locals.len();
                let width = (*locals as usize).max(count as usize);
                machine.locals.resize(mark + width, Cell::nothing());
                for slot in (0..count as usize).rev() {
                    let value = machine.stack.pop().unwrap_or_else(Cell::nothing);
                    machine.share(value);
                    machine.locals[mark + slot] = value;
                }
                let value = execute(plan, store, machine, inner, mark);
                machine.locals.truncate(mark);
                machine.stack.push(value);
            }
            Op::StoreColumn(key) => {
                let value = machine.stack.pop().unwrap_or_else(Cell::nothing);
                if let Some(row) = machine.row {
                    let held = settle(store, machine, value);
                    mark(store, machine, key, row, held, true);
                }
            }
            Op::AddColumn(key) => {
                let value = machine.stack.pop().unwrap_or_else(Cell::nothing);
                if let Some(row) = machine.row {
                    mark(store, machine, key, row, value, false);
                }
            }
            Op::StoreNumberColumn(key) => {
                let value = machine.stack.pop().unwrap_or_else(Cell::nothing);
                if let Some(row) = machine.row {
                    machine.shadow.put_number(key, row, value.value(), true);
                }
            }
            Op::AddNumberColumn(key) => {
                let value = machine.stack.pop().unwrap_or_else(Cell::nothing);
                if let Some(row) = machine.row {
                    let (held, kind) = match machine.shadow.number(key, row) {
                        Some((previous, kind)) => (previous + value.value(), kind),
                        None => (value.value(), false),
                    };
                    machine.shadow.put_number(key, row, held, kind);
                }
            }
            Op::StoreColumnOf(key) => {
                let subject = machine.stack.pop().unwrap_or_else(Cell::nothing);
                let value = machine.stack.pop().unwrap_or_else(Cell::nothing);
                if let Some(row) = store.seat(&subject) {
                    let held = settle(store, machine, value);
                    mark(store, machine, key, row, held, true);
                }
            }
            Op::AddColumnOf(key) => {
                let subject = machine.stack.pop().unwrap_or_else(Cell::nothing);
                let value = machine.stack.pop().unwrap_or_else(Cell::nothing);
                if let Some(row) = store.seat(&subject) {
                    mark(store, machine, key, row, value, false);
                }
            }
            Op::StoreAxis(key, index) => {
                let value = machine.stack.pop().unwrap_or_else(Cell::nothing);
                if let Some(row) = machine.row {
                    let held = peek(store, machine, key, row);
                    let made = blend(held, index as usize, value.value());
                    mark(store, machine, key, row, made, true);
                }
            }
            Op::AddAxis(key, index) => {
                let value = machine.stack.pop().unwrap_or_else(Cell::nothing);
                if let Some(row) = machine.row {
                    let made = only(index, value.value());
                    mark(store, machine, key, row, made, false);
                }
            }
            Op::StoreAxisOf(key, index) => {
                let subject = machine.stack.pop().unwrap_or_else(Cell::nothing);
                let value = machine.stack.pop().unwrap_or_else(Cell::nothing);
                if let Some(row) = store.seat(&subject) {
                    let held = peek(store, machine, key, row);
                    let made = blend(held, index as usize, value.value());
                    mark(store, machine, key, row, made, true);
                }
            }
            Op::AddAxisOf(key, index) => {
                let subject = machine.stack.pop().unwrap_or_else(Cell::nothing);
                let value = machine.stack.pop().unwrap_or_else(Cell::nothing);
                if let Some(row) = store.seat(&subject) {
                    let made = only(index, value.value());
                    mark(store, machine, key, row, made, false);
                }
            }
            Op::PokeField(slot) => {
                let value = machine.stack.pop().unwrap_or_else(Cell::nothing);
                let index = machine.stack.pop().unwrap_or_else(Cell::nothing);
                let held = own_field(store, machine, slot);
                let made = settle(store, machine, value);
                poke(store, held, index.value().max(0.0) as usize, made);
            }
            Op::PokeColumn(key) => {
                let value = machine.stack.pop().unwrap_or_else(Cell::nothing);
                let index = machine.stack.pop().unwrap_or_else(Cell::nothing);
                if let Some(row) = machine.row {
                    let held = own_column(store, machine, key, row);
                    let made = settle(store, machine, value);
                    poke(store, held, index.value().max(0.0) as usize, made);
                }
            }
            Op::PokeColumnOf(key) => {
                let subject = machine.stack.pop().unwrap_or_else(Cell::nothing);
                let value = machine.stack.pop().unwrap_or_else(Cell::nothing);
                let index = machine.stack.pop().unwrap_or_else(Cell::nothing);
                if let Some(row) = store.seat(&subject) {
                    let held = own_column(store, machine, key, row);
                    let made = settle(store, machine, value);
                    poke(store, held, index.value().max(0.0) as usize, made);
                }
            }
            Op::StoreField(slot) => {
                let value = machine.stack.pop().unwrap_or_else(Cell::nothing);
                let held = settle(store, machine, value);
                mark_field(store, machine, slot, held, true);
            }
            Op::AddField(slot) => {
                let value = machine.stack.pop().unwrap_or_else(Cell::nothing);
                mark_field(store, machine, slot, value, false);
            }
            Op::StoreAxisField(slot, index) => {
                let value = machine.stack.pop().unwrap_or_else(Cell::nothing);
                let held = peek_field(store, machine, slot);
                let made = blend(held, index as usize, value.value());
                mark_field(store, machine, slot, made, true);
            }
            Op::Spawn(site) => {
                let Some(keys) = plan.sites.get(site as usize) else {
                    continue;
                };
                let start = store.spawn_values.len() as u32;
                let mark = machine.stack.len().saturating_sub(keys.len());
                let values: Vec<Cell> = machine.stack.drain(mark..).collect();
                for value in values {
                    let held = settle(store, machine, value);
                    store.spawn_values.push(held);
                }
                store.spawn_sites.push((site, start, keys.len() as u32));
            }
            Op::Remove => {
                let subject = machine.stack.pop().unwrap_or_else(Cell::nothing);
                if let Some(row) = store.seat(&subject) {
                    store.doom(row);
                }
            }
            Op::Emit(key) => {
                let value = machine.stack.pop().unwrap_or_else(Cell::nothing);
                let held = settle(store, machine, value);
                machine.shadow.emits.push((key, held));
            }
            Op::Jump(target) => at = target as usize,
            Op::JumpIfFalse(target) => {
                let value = machine.stack.pop().unwrap_or_else(Cell::nothing);
                if !value.truthy() {
                    at = target as usize;
                }
            }
            Op::ForNext(site) => {
                let Some(site) = plan.loops.get(site as usize) else {
                    continue;
                };
                let list = machine
                    .locals
                    .get(machine.base + site.list as usize)
                    .copied()
                    .unwrap_or_else(Cell::nothing);
                let mut cursor = machine
                    .locals
                    .get(machine.base + site.cursor as usize)
                    .map(|value| value.value() as usize)
                    .unwrap_or_default();
                let width = list_len(store, machine, list);
                let mut item = Cell::nothing();
                let mut found = false;
                while cursor < width {
                    let held = list_at(store, machine, list, cursor);
                    cursor += 1;
                    if let Some(row) = held.row()
                        && store.doomed.holds(row)
                    {
                        continue;
                    }
                    item = held;
                    found = true;
                    break;
                }
                let cursor_at = machine.base + site.cursor as usize;
                if cursor_at < machine.locals.len() {
                    machine.locals[cursor_at] = Cell::number(cursor as f32);
                }
                if !found {
                    at = site.done as usize;
                    continue;
                }
                let name_at = machine.base + site.name as usize;
                machine.share(item);
                if name_at < machine.locals.len() {
                    machine.locals[name_at] = item;
                }
            }
            Op::SetIndex => {
                let value = machine.stack.pop().unwrap_or_else(Cell::nothing);
                let index = machine.stack.pop().unwrap_or_else(Cell::nothing);
                let held = machine.stack.pop().unwrap_or_else(Cell::nothing);
                let target = index.value().max(0.0) as usize;
                if machine.alone(held)
                    && let Some(items) = machine.lists.get_mut(held.slot as usize)
                {
                    while items.len() <= target {
                        items.push(Cell::nothing());
                    }
                    items[target] = value;
                    machine.stack.push(held);
                    continue;
                }
                let mut items = gather(store, machine, held);
                while items.len() <= target {
                    items.push(Cell::nothing());
                }
                items[target] = value;
                let slot = machine.hold_list(items);
                machine.stack.push(Cell::list(slot));
            }
            Op::Append => {
                let value = machine.stack.pop().unwrap_or_else(Cell::nothing);
                let held = machine.stack.pop().unwrap_or_else(Cell::nothing);
                if machine.alone(held)
                    && let Some(items) = machine.lists.get_mut(held.slot as usize)
                {
                    items.push(value);
                    machine.stack.push(held);
                    continue;
                }
                let mut items = gather(store, machine, held);
                items.push(value);
                let slot = machine.hold_list(items);
                machine.stack.push(Cell::list(slot));
            }
            Op::Pop => {
                machine.stack.pop();
            }
            Op::Keep(site) => machine.shadow.keeps.push((site, false)),
            Op::Recall(site) => machine.shadow.keeps.push((site, true)),
            Op::Halt => {
                machine.halted = true;
                break;
            }
        }
    }
    machine.base = outer;
    machine.stack.pop().unwrap_or_else(Cell::nothing)
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

fn combine(
    store: &mut Store,
    machine: &mut Machine,
    operator: BinaryOperator,
    left: Cell,
    right: Cell,
) -> Cell {
    match operator {
        BinaryOperator::Range => {
            let from = left.value().round() as i64;
            let to = right.value().round() as i64;
            let items: Vec<Cell> = (from..to).map(|step| Cell::number(step as f32)).collect();
            let slot = machine.hold_list(items);
            Cell::list(slot)
        }
        BinaryOperator::And => Cell::truth(left.truthy() && right.truthy()),
        BinaryOperator::Or => Cell::truth(left.truthy() || right.truthy()),
        BinaryOperator::Equal => Cell::truth(alike(store, machine, left, right)),
        BinaryOperator::NotEqual => Cell::truth(!alike(store, machine, left, right)),
        BinaryOperator::Less => Cell::truth(left.value() < right.value()),
        BinaryOperator::LessOrEqual => Cell::truth(left.value() <= right.value()),
        BinaryOperator::Greater => Cell::truth(left.value() > right.value()),
        BinaryOperator::GreaterOrEqual => Cell::truth(left.value() >= right.value()),
        BinaryOperator::Add => {
            if left.tag == Tag::Text || right.tag == Tag::Text {
                let joined = format!(
                    "{}{}",
                    show(store, machine, left),
                    show(store, machine, right)
                );
                let slot = machine.hold_text(joined);
                return Cell::text(slot);
            }
            left.spread(&right, |first, second| first + second)
        }
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
    }
}

fn builtin(store: &Store, machine: &Machine, which: u8, found: &[Cell; 4]) -> Cell {
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
            let width = if value.tag == Tag::Vector {
                value.width as usize
            } else {
                1
            };
            let total: f32 = (0..width).map(|index| value.axis(index).powi(2)).sum();
            Cell::number(total.sqrt())
        }
        11 => separation(found),
        13 => Cell::number(list_len(store, machine, found[0]) as f32),
        14 => whole(first, second, |left, right| left.rem_euclid(right)),
        15 => whole(first, second, |left, right| left.div_euclid(right)),
        16 => bits(first, second, |left, right| left & right),
        17 => bits(first, second, |left, right| left | right),
        18 => bits(first, second, |left, right| left ^ right),
        19 => {
            let steps = second as i64;
            let base = first as i64;
            let moved = if steps < 0 {
                base >> (-steps).min(63)
            } else {
                base << steps.min(63)
            };
            Cell::number(moved as f32)
        }
        _ => Cell::nothing(),
    }
}

fn whole(first: f32, second: f32, join: fn(f32, f32) -> f32) -> Cell {
    if second == 0.0 {
        return Cell::number(0.0);
    }
    Cell::number(join(first.trunc(), second.trunc()))
}

fn bits(first: f32, second: f32, join: fn(i64, i64) -> i64) -> Cell {
    Cell::number(join(first as i64, second as i64) as f32)
}

fn separation(found: &[Cell; 4]) -> Cell {
    let center = found[0];
    let radius = found[1].value();
    let box_center = found[2];
    let half = found[3];

    let away_x = center.axis(0) - box_center.axis(0);
    let away_y = center.axis(1) - box_center.axis(1);
    let half_x = half.axis(0);
    let half_y = half.axis(1);

    let outside_x = away_x - away_x.clamp(-half_x, half_x);
    let outside_y = away_y - away_y.clamp(-half_y, half_y);
    let distance = (outside_x * outside_x + outside_y * outside_y).sqrt();

    if distance > 1.0e-6 {
        if distance >= radius {
            return Cell::nothing();
        }
        return Cell::vector(&[outside_x / distance, outside_y / distance]);
    }

    let reach_x = half_x + radius - away_x.abs();
    let reach_y = half_y + radius - away_y.abs();
    if reach_x < reach_y {
        Cell::vector(&[if away_x < 0.0 { -1.0 } else { 1.0 }, 0.0])
    } else {
        Cell::vector(&[0.0, if away_y < 0.0 { -1.0 } else { 1.0 }])
    }
}
