use crate::data::ask::Ask;
use crate::data::cell::{Cell, KEPT, Tag};
use crate::data::plan::Plan;
use nightshade::ecs::Entity;
use nightshade::prelude::Vec3;
use nightshade_cli::clap;
use nightshade_random::resources::Random;
use std::collections::{HashMap, HashSet};

#[derive(clap::Args, Default)]
pub struct Arguments {
    #[arg(long, help = "The name of the game file to play")]
    pub game: Option<String>,
}

#[derive(Default)]
pub struct Script {
    pub source: crate::data::Program,
    pub plan: Plan,
    pub fault: Option<String>,
    pub dirty: bool,
    pub wanted: Option<String>,
    pub models: Vec<String>,
    pub sounds: Vec<String>,
}

#[derive(Default)]
pub struct Vault {
    pub asked: Vec<(String, nightshade_load::LoadHandle)>,
    pub prefabs: HashMap<String, nightshade_prefab::Prefab>,
    pub lost: HashSet<String>,
    pub heard: HashSet<String>,
}

impl Vault {
    pub fn ready(&self, models: &[String]) -> bool {
        models
            .iter()
            .all(|path| self.prefabs.contains_key(path) || self.lost.contains(path))
    }

    pub fn forget(&mut self) {
        self.asked.clear();
        self.lost.clear();
        self.heard.clear();
    }
}

#[derive(Clone, Copy, Default, PartialEq)]
pub struct Land {
    pub on: bool,
    pub seed: u32,
    pub hills: f32,
    pub shore: f32,
}

#[derive(Clone, Default, PartialEq)]
pub struct Sign {
    pub line: String,
    pub tone: [f32; 4],
    pub size: f32,
    pub justify: u8,
    pub anchor: [f32; 2],
}

#[derive(Clone, Default, PartialEq)]
pub struct Plate {
    pub size: [f32; 2],
    pub tone: [f32; 4],
    pub anchor: [f32; 2],
    pub line: String,
    pub ink: [f32; 4],
    pub letters: f32,
}

#[derive(Default)]
pub struct Screen {
    pub root: Option<Entity>,
    pub shown: Vec<Sign>,
    pub plates: Vec<Plate>,
}

#[derive(Default)]
pub struct Switcher {
    pub buttons: Vec<(String, Entity)>,
    pub playing: String,
}

#[derive(Clone, Default)]
pub struct Snapshot {
    pub banks: Vec<Bank>,
    pub fields: Vec<Cell>,
    pub width: usize,
    pub lives: Vec<u32>,
    pub alive: Marks,
    pub free: Vec<u32>,
    pub kept_texts: Vec<String>,
    pub kept_lists: Vec<Vec<Cell>>,
    pub spare_texts: Vec<u32>,
    pub spare_lists: Vec<u32>,
    pub seed: u32,
    pub frame: u32,
    pub started: bool,
}

pub enum Frame {
    Base(Snapshot),
    Step(crate::data::change::Change),
}

#[derive(Default)]
pub struct Tape {
    pub frames: Vec<Frame>,
    pub last: Snapshot,
    pub since: usize,
    pub cursor: Option<usize>,
    pub seek: Option<usize>,
}

impl Tape {
    pub fn forget(&mut self) {
        self.frames.clear();
        self.last = Snapshot::default();
        self.since = 0;
        self.cursor = None;
        self.seek = None;
    }

    pub fn at(&self) -> usize {
        self.cursor.unwrap_or(self.frames.len().saturating_sub(1))
    }
}

#[derive(Default)]
pub struct Transport {
    pub restart: Option<Entity>,
    pub back: Option<Entity>,
    pub play: Option<Entity>,
    pub pause: Option<Entity>,
    pub step: Option<Entity>,
    pub ahead: Option<Entity>,
    pub scrub: Option<Entity>,
}

#[derive(Default)]
pub struct Clock {
    pub paused: bool,
    pub step: bool,
}

#[derive(Default)]
pub struct Studio {
    pub root: Option<Entity>,
    pub graph: Option<Entity>,
    pub input: Option<Entity>,
    pub controls: Vec<(crate::data::edit::Touch, Entity)>,
    pub chosen: Option<usize>,
    pub shown: String,
    pub name: String,
    pub tab: usize,
    pub form: usize,
    pub port: usize,
    pub step: usize,
    pub stale: bool,
}

#[derive(Clone, Default)]
pub struct Bank {
    pub present: Vec<u64>,
    pub data: Vec<Cell>,
    pub numbers: Vec<f32>,
    pub plain: bool,
}

impl Bank {
    pub fn read(&self, row: u32) -> Cell {
        if !self.holds(row) {
            return Cell::nothing();
        }
        if self.plain {
            return Cell::number(self.numbers.get(row as usize).copied().unwrap_or_default());
        }
        self.data
            .get(row as usize)
            .copied()
            .unwrap_or_else(Cell::nothing)
    }

    pub fn put(&mut self, row: u32, value: Cell, width: usize) {
        let wanted = width.max(row as usize + 1);
        if self.plain {
            if self.numbers.len() < wanted {
                self.numbers.resize(wanted, 0.0);
            }
            self.numbers[row as usize] = value.value();
        } else {
            if self.data.len() < wanted {
                self.data.resize(wanted, Cell::nothing());
            }
            self.data[row as usize] = value;
        }
        self.mark(row);
    }

    pub fn weight(&self) -> u64 {
        if self.plain {
            std::mem::size_of::<f32>() as u64
        } else {
            std::mem::size_of::<Cell>() as u64
        }
    }

    pub fn holds(&self, row: u32) -> bool {
        let word = (row >> 6) as usize;
        self.present
            .get(word)
            .is_some_and(|bits| bits & (1 << (row & 63)) != 0)
    }

    pub fn mark(&mut self, row: u32) -> bool {
        let word = (row >> 6) as usize;
        if word >= self.present.len() {
            self.present.resize(word + 1, 0);
        }
        let held = self.present[word];
        self.present[word] = held | (1 << (row & 63));
        self.present[word] != held
    }

    pub fn unmark(&mut self, row: u32) -> bool {
        let word = (row >> 6) as usize;
        match self.present.get_mut(word) {
            Some(bits) => {
                let held = *bits;
                *bits &= !(1 << (row & 63));
                *bits != held
            }
            None => false,
        }
    }
}

#[derive(Default)]
pub struct Writes {
    pub rows: Vec<u32>,
    pub values: Vec<Cell>,
    pub numbers: Vec<f32>,
    pub absolute: Vec<bool>,
    pub present: Marks,
    pub at: Vec<u32>,
    pub plain: bool,
}

impl Writes {
    fn find(&self, row: u32) -> Option<usize> {
        if !self.present.holds(row) {
            return None;
        }
        self.at.get(row as usize).map(|slot| *slot as usize)
    }

    fn get(&self, row: u32) -> Option<(Cell, bool)> {
        let at = self.find(row)?;
        if self.plain {
            return Some((Cell::number(self.numbers[at]), self.absolute[at]));
        }
        Some((self.values[at], self.absolute[at]))
    }

    pub fn held(&self, at: usize) -> Cell {
        if self.plain {
            Cell::number(self.numbers[at])
        } else {
            self.values[at]
        }
    }

    fn open(&mut self, row: u32) -> usize {
        match self.find(row) {
            Some(at) => at,
            None => {
                if self.at.len() <= row as usize {
                    self.at.resize(row as usize + 1, 0);
                }
                self.at[row as usize] = self.rows.len() as u32;
                self.present.mark(row);
                self.rows.push(row);
                self.values.push(Cell::nothing());
                self.numbers.push(0.0);
                self.absolute.push(false);
                self.rows.len() - 1
            }
        }
    }

    fn put(&mut self, row: u32, value: Cell, absolute: bool) {
        let at = self.open(row);
        if self.plain {
            self.numbers[at] = value.value();
        } else {
            self.values[at] = value;
        }
        self.absolute[at] = absolute;
    }

    pub fn clear(&mut self) {
        for row in &self.rows {
            self.present.unmark(*row);
        }
        self.rows.clear();
        self.values.clear();
        self.numbers.clear();
        self.absolute.clear();
    }
}

#[derive(Default)]
pub struct Shadow {
    pub columns: Vec<Writes>,
    pub touched: Vec<u16>,
    pub values: Vec<Cell>,
    pub absolute: Vec<bool>,
    pub present: Marks,
    pub slots: Vec<u16>,
    pub emits: Vec<(u16, Cell)>,
    pub keeps: Vec<(u16, bool)>,
}

impl Shadow {
    pub fn column(&self, key: u16, row: u32) -> Option<(Cell, bool)> {
        self.columns.get(key as usize)?.get(row)
    }

    pub fn ready(&mut self, plain: &[bool]) {
        self.clear();
        self.columns.clear();
        self.columns.resize_with(plain.len(), Writes::default);
        for (column, held) in self.columns.iter_mut().zip(plain.iter()) {
            column.plain = *held;
        }
    }

    pub fn put_column(&mut self, key: u16, row: u32, value: Cell, absolute: bool) {
        if self.columns.len() <= key as usize {
            self.columns.resize_with(key as usize + 1, Writes::default);
        }
        let column = &mut self.columns[key as usize];
        let fresh = column.rows.is_empty();
        column.put(row, value, absolute);
        if fresh {
            self.touched.push(key);
        }
    }

    pub fn number(&self, key: u16, row: u32) -> Option<(f32, bool)> {
        let column = self.columns.get(key as usize)?;
        let at = column.find(row)?;
        Some((column.held(at).value(), column.absolute[at]))
    }

    pub fn put_number(&mut self, key: u16, row: u32, value: f32, absolute: bool) {
        if self.columns.len() <= key as usize {
            self.columns.resize_with(key as usize + 1, Writes::default);
        }
        let column = &mut self.columns[key as usize];
        let fresh = column.rows.is_empty();
        let at = column.open(row);
        if column.plain {
            column.numbers[at] = value;
        } else {
            column.values[at] = Cell::number(value);
        }
        column.absolute[at] = absolute;
        if fresh {
            self.touched.push(key);
        }
    }

    pub fn field(&self, slot: u16) -> Option<(Cell, bool)> {
        if !self.present.holds(slot as u32) {
            return None;
        }
        Some((self.values[slot as usize], self.absolute[slot as usize]))
    }

    pub fn put_field(&mut self, slot: u16, value: Cell, absolute: bool) {
        if self.values.len() <= slot as usize {
            self.values.resize(slot as usize + 1, Cell::nothing());
            self.absolute.resize(slot as usize + 1, false);
        }
        if !self.present.holds(slot as u32) {
            self.present.mark(slot as u32);
            self.slots.push(slot);
        }
        self.values[slot as usize] = value;
        self.absolute[slot as usize] = absolute;
    }

    pub fn clear(&mut self) {
        for key in self.touched.drain(..) {
            if let Some(column) = self.columns.get_mut(key as usize) {
                column.clear();
            }
        }
        for slot in self.slots.drain(..) {
            self.present.unmark(slot as u32);
        }
        self.emits.clear();
        self.keeps.clear();
    }

    pub fn empty(&self) -> bool {
        self.touched.is_empty()
            && self.slots.is_empty()
            && self.emits.is_empty()
            && self.keeps.is_empty()
    }
}

#[derive(Default)]
pub struct Spread {
    pub values: Vec<Cell>,
    pub kind: Vec<u8>,
}

#[derive(Default)]
pub struct Sheet {
    pub width: usize,
    pub at: Vec<u32>,
    pub holds: Vec<Spread>,
    pub touched: Vec<u16>,
    pub slot_at: Vec<u32>,
    pub fields: Vec<Spread>,
    pub marked: Vec<u16>,
}

impl Sheet {
    pub fn ready(&mut self, width: usize) {
        self.width = width;
        for key in self.touched.drain(..) {
            self.at[key as usize] = u32::MAX;
        }
        for slot in self.marked.drain(..) {
            self.slot_at[slot as usize] = u32::MAX;
        }
        self.holds.clear();
        self.fields.clear();
    }

    pub fn column(&self, key: u16, index: usize) -> Option<(Cell, bool)> {
        let at = *self.at.get(key as usize)?;
        let held = self.holds.get(at as usize)?;
        match held.kind[index] {
            0 => None,
            1 => Some((held.values[index], true)),
            _ => Some((held.values[index], false)),
        }
    }

    pub fn put(&mut self, key: u16, index: usize, value: Cell, absolute: bool) {
        if self.at.len() <= key as usize {
            self.at.resize(key as usize + 1, u32::MAX);
        }
        if self.at[key as usize] == u32::MAX {
            self.at[key as usize] = self.holds.len() as u32;
            self.holds.push(Spread {
                values: vec![Cell::nothing(); self.width],
                kind: vec![0; self.width],
            });
            self.touched.push(key);
        }
        let held = &mut self.holds[self.at[key as usize] as usize];
        match (held.kind[index], absolute) {
            (0, true) => {
                held.values[index] = value;
                held.kind[index] = 1;
            }
            (0, false) => {
                held.values[index] = value;
                held.kind[index] = 2;
            }
            (_, true) => {
                held.values[index] = value;
                held.kind[index] = 1;
            }
            (_, false) => {
                held.values[index] = held.values[index].spread(&value, |base, more| base + more);
            }
        }
    }

    pub fn field(&self, slot: u16, index: usize) -> Option<(Cell, bool)> {
        let at = *self.slot_at.get(slot as usize)?;
        let held = self.fields.get(at as usize)?;
        match held.kind[index] {
            0 => None,
            1 => Some((held.values[index], true)),
            _ => Some((held.values[index], false)),
        }
    }

    pub fn put_field(&mut self, slot: u16, index: usize, value: Cell, absolute: bool) {
        if self.slot_at.len() <= slot as usize {
            self.slot_at.resize(slot as usize + 1, u32::MAX);
        }
        if self.slot_at[slot as usize] == u32::MAX {
            self.slot_at[slot as usize] = self.fields.len() as u32;
            self.fields.push(Spread {
                values: vec![Cell::nothing(); self.width],
                kind: vec![0; self.width],
            });
            self.marked.push(slot);
        }
        let held = &mut self.fields[self.slot_at[slot as usize] as usize];
        match (held.kind[index], absolute) {
            (0, false) => {
                held.values[index] = value;
                held.kind[index] = 2;
            }
            (_, true) => {
                held.values[index] = value;
                held.kind[index] = 1;
            }
            (_, false) => {
                held.values[index] = held.values[index].spread(&value, |base, more| base + more);
            }
        }
    }
}

#[derive(Default)]
pub struct Machine {
    pub stack: Vec<Cell>,
    pub locals: Vec<Cell>,
    pub base: usize,
    pub texts: Vec<String>,
    pub lists: Vec<Vec<Cell>>,
    pub members: Vec<u32>,
    pub row: Option<u32>,
    pub halted: bool,
    pub draws: u32,
    pub steps: u64,
    pub shadow: Shadow,
    pub shared: Marks,
    pub benches: Vec<Bench>,
}

#[derive(Default)]
pub struct Bench {
    pub sheet: Sheet,
    pub lanes: Vec<Vec<Cell>>,
    pub spare: Vec<Vec<Cell>>,
    pub locals: Vec<Vec<Cell>>,
    pub batch: Vec<(u32, Ask)>,
    pub steps: u64,
}

impl Machine {
    pub fn clear(&mut self) {
        self.stack.clear();
        self.locals.clear();
        self.texts.clear();
        self.lists.clear();
        self.base = 0;
        self.row = None;
        self.halted = false;
        self.draws = 0;
        self.shadow.clear();
        self.shared.clear();
    }

    pub fn share(&mut self, value: Cell) {
        if value.tag == Tag::List && value.slot & KEPT == 0 {
            self.shared.mark(value.slot);
        }
    }

    pub fn alone(&self, value: Cell) -> bool {
        value.tag == Tag::List && value.slot & KEPT == 0 && !self.shared.holds(value.slot)
    }

    pub fn hold_text(&mut self, value: String) -> u32 {
        self.texts.push(value);
        (self.texts.len() - 1) as u32
    }

    pub fn hold_list(&mut self, items: Vec<Cell>) -> u32 {
        self.lists.push(items);
        (self.lists.len() - 1) as u32
    }
}

#[derive(Clone, Default)]
pub struct Marks {
    pub bits: Vec<u64>,
}

impl Marks {
    pub fn holds(&self, seat: u32) -> bool {
        self.bits
            .get((seat >> 6) as usize)
            .is_some_and(|held| held & (1 << (seat & 63)) != 0)
    }

    pub fn mark(&mut self, seat: u32) {
        let word = (seat >> 6) as usize;
        if word >= self.bits.len() {
            self.bits.resize(word + 1, 0);
        }
        self.bits[word] |= 1 << (seat & 63);
    }

    pub fn unmark(&mut self, seat: u32) {
        if let Some(held) = self.bits.get_mut((seat >> 6) as usize) {
            *held &= !(1 << (seat & 63));
        }
    }

    pub fn clear(&mut self) {
        self.bits.clear();
    }
}

#[derive(Default)]
pub struct Store {
    pub rows: Vec<Entity>,
    pub lives: Vec<u32>,
    pub alive: Marks,
    pub doomed: Marks,
    pub free: Vec<u32>,
    pub banks: Vec<Bank>,
    pub fields: Vec<Cell>,
    pub events: Vec<Vec<Cell>>,
    pub kept_texts: Vec<String>,
    pub kept_lists: Vec<Vec<Cell>>,
    pub spare_texts: Vec<u32>,
    pub spare_lists: Vec<u32>,
    pub asks: Vec<Ask>,
    pub batch: Vec<(u32, Ask)>,
    pub spans: Vec<Cell>,
    pub segment: Vec<Ask>,
    pub frame: u32,
    pub marks: Vec<u32>,
    pub field_marks: Vec<u32>,
    pub ran: Vec<u32>,
    pub spawn_values: Vec<Cell>,
    pub spawn_sites: Vec<(u16, u32, u32)>,
    pub removals: Vec<u32>,
    pub materials: HashSet<String>,
    pub painted: Vec<u32>,
    pub camera: Option<Entity>,
    pub camera_home: Vec3,
    pub sun: Option<Entity>,
    pub sight: Option<(f32, f32, f32, f32)>,
    pub rate: f32,
    pub gpu: f32,
    pub signs: Vec<Sign>,
    pub panels: Vec<Plate>,
    pub canvas: Option<(f32, f32)>,
    pub pending: Vec<(u32, String)>,
    pub started: bool,
    pub fresh: bool,
    pub aimed: Option<(f32, f32, f32)>,
    pub walked: u32,
    pub carved: u32,
    pub dug: bool,
    pub steered: bool,
    pub seed: u32,
    pub shape: u32,
    pub rosters: Vec<(u32, Vec<u32>)>,
    pub bytes: u64,
    pub commands: u64,
    pub cost: (u64, u64, u64),
}

impl Store {
    pub fn forget(&mut self) {
        self.rows.clear();
        self.lives.clear();
        self.alive.clear();
        self.doomed.clear();
        self.free.clear();
        self.banks.clear();
        self.fields.clear();
        self.events.clear();
        self.kept_texts.clear();
        self.kept_lists.clear();
        self.spare_texts.clear();
        self.spare_lists.clear();
        self.asks.clear();
        self.batch.clear();
        self.spans.clear();
        self.segment.clear();
        self.frame = 0;
        self.marks.clear();
        self.field_marks.clear();
        self.ran.clear();
        self.spawn_values.clear();
        self.spawn_sites.clear();
        self.removals.clear();
        self.pending.clear();
        self.signs.clear();
        self.panels.clear();
        self.painted.clear();
        self.started = false;
        self.aimed = None;
        self.walked = 0;
        self.carved = 0;
        self.steered = false;
        self.seed = 1;
        self.shape = 1;
        self.rosters.clear();
        self.bytes = 0;
        self.commands = 0;
        self.cost = (0, 0, 0);
    }

    pub fn ready(
        &mut self,
        columns: usize,
        fields: usize,
        events: usize,
        rosters: usize,
        plain: &[bool],
    ) {
        self.rosters.clear();
        self.rosters.resize_with(rosters, || (0, Vec::new()));
        self.shape = self.shape.wrapping_add(1);
        self.banks.clear();
        self.banks.resize_with(columns, Bank::default);
        for key in 0..columns {
            self.banks[key].plain = plain.get(key).copied().unwrap_or_default();
        }
        self.marks.clear();
        self.marks.resize(columns, 0);
        self.fields.clear();
        self.fields.resize(fields, Cell::nothing());
        self.field_marks.clear();
        self.field_marks.resize(fields, 0);
        self.ran.clear();
        self.events.clear();
        self.events.resize_with(events, Vec::new);
    }

    pub fn capture(&self) -> Snapshot {
        Snapshot {
            banks: self.banks.clone(),
            fields: self.fields.clone(),
            width: self.rows.len(),
            lives: self.lives.clone(),
            alive: self.alive.clone(),
            free: self.free.clone(),
            kept_texts: self.kept_texts.clone(),
            kept_lists: self.kept_lists.clone(),
            spare_texts: self.spare_texts.clone(),
            spare_lists: self.spare_lists.clone(),
            seed: self.seed,
            frame: self.frame,
            started: self.started,
        }
    }

    pub fn restore(&mut self, held: &Snapshot) {
        self.banks = held.banks.clone();
        self.fields = held.fields.clone();
        self.free = held.free.clone();
        self.lives = held.lives.clone();
        self.alive = held.alive.clone();
        self.doomed.clear();
        self.kept_texts = held.kept_texts.clone();
        self.kept_lists = held.kept_lists.clone();
        self.spare_texts = held.spare_texts.clone();
        self.spare_lists = held.spare_lists.clone();
        self.seed = held.seed;
        self.started = held.started;
        self.rows.resize(held.width, Entity::default());
        self.painted.clear();
        self.painted.resize(held.width, u32::MAX);
        self.asks.clear();
        self.batch.clear();
        self.spans.clear();
        self.segment.clear();
        self.frame = held.frame;
        self.shape = self.shape.wrapping_add(1);
        for roster in &mut self.rosters {
            roster.0 = 0;
        }
        for mark in self.marks.iter_mut().chain(self.field_marks.iter_mut()) {
            *mark = held.frame;
        }
        self.ran.clear();
        self.spawn_values.clear();
        self.spawn_sites.clear();
        self.removals.clear();
        self.pending.clear();
        self.signs.clear();
        self.panels.clear();
        for events in &mut self.events {
            events.clear();
        }
    }

    pub fn living(&self, seat: u32) -> bool {
        self.alive.holds(seat)
    }

    pub fn seat(&self, value: &Cell) -> Option<u32> {
        let seat = value.row()?;
        let life = self.lives.get(seat as usize).copied().unwrap_or_default();
        (life == value.life() && self.alive.holds(seat)).then_some(seat)
    }

    pub fn held(&self, seat: u32) -> Cell {
        let life = self.lives.get(seat as usize).copied().unwrap_or_default();
        Cell::handle(seat, life)
    }

    pub fn draw(&self, at: u32, step: u32) -> f32 {
        let key = (self.seed as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)
            ^ (self.frame as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F)
            ^ (at as u64).wrapping_mul(0xBF58_476D_1CE4_E5B9)
            ^ (step as u64).wrapping_mul(0x94D0_49BB_1331_11EB);
        let mut stream = Random::seeded(key);
        nightshade_random::commands::between(&mut stream, 0.0, 1.0)
    }

    pub fn make_row(&mut self, entity: Entity) -> u32 {
        let seat = match self.free.pop() {
            Some(seat) => {
                self.rows[seat as usize] = entity;
                if let Some(life) = self.lives.get_mut(seat as usize) {
                    *life = life.wrapping_add(1);
                }
                seat
            }
            None => {
                self.rows.push(entity);
                self.lives.push(1);
                self.painted.push(u32::MAX);
                (self.rows.len() - 1) as u32
            }
        };
        self.alive.mark(seat);
        self.doomed.unmark(seat);
        self.shape = self.shape.wrapping_add(1);
        seat
    }

    pub fn entity(&self, row: u32) -> Option<Entity> {
        self.rows.get(row as usize).copied()
    }

    pub fn drop_row(&mut self, seat: u32) {
        for key in 0..self.banks.len() {
            self.clear(key as u16, seat);
        }
        if let Some(slot) = self.painted.get_mut(seat as usize) {
            *slot = u32::MAX;
        }
        self.alive.unmark(seat);
        self.doomed.unmark(seat);
        self.free.push(seat);
        self.shape = self.shape.wrapping_add(1);
    }

    pub fn doom(&mut self, seat: u32) {
        if self.doomed.holds(seat) {
            return;
        }
        self.doomed.mark(seat);
        self.removals.push(seat);
    }

    pub fn read(&self, key: u16, row: u32) -> Cell {
        match self.banks.get(key as usize) {
            Some(bank) => bank.read(row),
            None => Cell::nothing(),
        }
    }

    pub fn select(&self, need: &[u16], ban: &[u16], found: &mut Vec<u32>) {
        found.clear();
        let Some(first) = need.first() else {
            return;
        };
        let Some(bank) = self.banks.get(*first as usize) else {
            return;
        };
        for word in 0..bank.present.len() {
            let mut bits = bank.present[word] & self.alive.bits.get(word).copied().unwrap_or(0);
            for key in &need[1..] {
                if bits == 0 {
                    break;
                }
                bits &= self.present_word(*key, word);
            }
            for key in ban {
                if bits == 0 {
                    break;
                }
                bits &= !self.present_word(*key, word);
            }
            while bits != 0 {
                let at = bits.trailing_zeros();
                bits &= bits - 1;
                found.push(((word as u32) << 6) | at);
            }
        }
    }

    fn present_word(&self, key: u16, word: usize) -> u64 {
        self.banks
            .get(key as usize)
            .and_then(|bank| bank.present.get(word))
            .copied()
            .unwrap_or(0)
    }

    pub fn holds(&self, key: u16, row: u32) -> bool {
        self.banks
            .get(key as usize)
            .is_some_and(|bank| bank.holds(row))
    }

    pub fn write(&mut self, key: u16, row: u32, value: Cell) {
        if let Some(mark) = self.marks.get_mut(key as usize) {
            *mark = self.frame;
        }
        let previous = self.read(key, row);
        self.release(previous);
        let width = self.rows.len();
        let Some(bank) = self.banks.get_mut(key as usize) else {
            return;
        };
        let fresh = !bank.holds(row);
        let weight = bank.weight();
        bank.put(row, value, width);
        self.bytes += weight;
        if fresh {
            self.shape = self.shape.wrapping_add(1);
        }
    }

    pub fn clear(&mut self, key: u16, row: u32) {
        if let Some(mark) = self.marks.get_mut(key as usize) {
            *mark = self.frame;
        }
        let previous = self.read(key, row);
        self.release(previous);
        if let Some(bank) = self.banks.get_mut(key as usize)
            && bank.unmark(row)
        {
            self.shape = self.shape.wrapping_add(1);
        }
    }

    pub fn field(&self, slot: u16) -> Cell {
        self.fields
            .get(slot as usize)
            .copied()
            .unwrap_or_else(Cell::nothing)
    }

    pub fn set_field(&mut self, slot: u16, value: Cell) {
        if let Some(mark) = self.field_marks.get_mut(slot as usize) {
            *mark = self.frame;
        }
        let previous = self.field(slot);
        self.release(previous);
        self.bytes += std::mem::size_of::<Cell>() as u64;
        if let Some(held) = self.fields.get_mut(slot as usize) {
            *held = value;
        }
    }

    pub fn keep_text(&mut self, value: &str) -> u32 {
        match self.spare_texts.pop() {
            Some(slot) => {
                self.kept_texts[slot as usize].clear();
                self.kept_texts[slot as usize].push_str(value);
                slot | KEPT
            }
            None => {
                self.kept_texts.push(value.to_string());
                ((self.kept_texts.len() - 1) as u32) | KEPT
            }
        }
    }

    pub fn keep_list(&mut self, items: &[Cell]) -> u32 {
        match self.spare_lists.pop() {
            Some(slot) => {
                let held = &mut self.kept_lists[slot as usize];
                held.clear();
                held.extend_from_slice(items);
                slot | KEPT
            }
            None => {
                self.kept_lists.push(items.to_vec());
                ((self.kept_lists.len() - 1) as u32) | KEPT
            }
        }
    }

    pub fn kept_text(&self, slot: u32) -> &str {
        self.kept_texts
            .get((slot & !KEPT) as usize)
            .map(String::as_str)
            .unwrap_or_default()
    }

    pub fn kept_list(&self, slot: u32) -> &[Cell] {
        self.kept_lists
            .get((slot & !KEPT) as usize)
            .map(Vec::as_slice)
            .unwrap_or_default()
    }

    pub fn drop_events(&mut self) {
        for key in 0..self.events.len() {
            let held = std::mem::take(&mut self.events[key]);
            for value in held {
                self.release(value);
            }
        }
    }

    pub fn drop_bank(&mut self, key: u16) {
        let width = self.rows.len() as u32;
        for row in 0..width {
            if self.holds(key, row) {
                self.clear(key, row);
            }
        }
    }

    pub fn release(&mut self, value: Cell) {
        if value.slot & KEPT == 0 {
            return;
        }
        match value.tag {
            Tag::Text => self.spare_texts.push(value.slot & !KEPT),
            Tag::List => self.spare_lists.push(value.slot & !KEPT),
            _ => {}
        }
    }
}
