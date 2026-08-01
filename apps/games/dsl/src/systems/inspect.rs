use crate::data;
use crate::data::edit::Touch;
use crate::data::plan::{Plan, Unit};
use crate::queries::shape::{Card, Kind};
use nightshade_ui::components::CanvasCommand;
use nightshade_ui::prelude::*;

const DIM: [f32; 4] = [0.42, 0.47, 0.58, 1.0];
const READ: [f32; 4] = [0.40, 0.74, 1.0, 1.0];
const WRITE: [f32; 4] = [1.0, 0.62, 0.20, 1.0];
const SHARED: [f32; 4] = [0.32, 0.86, 0.56, 1.0];
const ASKED: [f32; 4] = [1.0, 0.49, 0.78, 1.0];
const CODE: [f32; 4] = [0.80, 0.84, 0.94, 1.0];
const LOOP: [f32; 4] = [0.55, 0.80, 0.45, 1.0];
const GATE: [f32; 4] = [0.55, 0.62, 1.0, 1.0];
const MAKE: [f32; 4] = [0.30, 0.80, 0.78, 1.0];
const HALT: [f32; 4] = [0.95, 0.40, 0.42, 1.0];

const CARD_WIDTH: f32 = 196.0;
const TITLE_HEIGHT: f32 = 21.0;
const ROW_HEIGHT: f32 = 15.0;
const CARD_GAP: f32 = 9.0;
const COLUMN_GAP: f32 = 78.0;
const EDGE_ROOM: f32 = 6.0;
const BLOCK_HEIGHT: f32 = 20.0;
const BLOCK_STEP: f32 = 14.0;
pub const MARK: usize = 10000;

pub fn draw(
    tree: &mut UiTreeBuilder,
    plan: &Plan,
    cards: &[Card],
    chosen: Option<usize>,
    controls: &mut Vec<(Touch, Entity)>,
) -> Entity {
    let (size, paint) = plot(plan, cards, chosen);
    let canvas = tree.add_canvas_interactive(size);
    tree.edit::<UiCanvasData>(canvas, move |data| {
        data.commands = paint;
        data.hit_test_enabled = true;
    });
    controls.push((Touch::Graph, canvas));

    match chosen.and_then(|at| plan.units.get(at)) {
        Some(unit) => detail(tree, unit),
        None => chip(tree, "pick a system to follow its lines", DIM),
    }
    canvas
}

pub fn plot(plan: &Plan, cards: &[Card], chosen: Option<usize>) -> (Vec2, Vec<CanvasCommand>) {
    let unit = chosen.and_then(|at| plan.units.get(at));
    let touched: Vec<String> = unit.map(names).unwrap_or_default();

    let mut places: Vec<(f32, f32)> = Vec::new();
    let mut cursor = CARD_GAP;
    for card in cards {
        let height = TITLE_HEIGHT + card.rows.len() as f32 * ROW_HEIGHT + 6.0;
        places.push((cursor, height));
        cursor += height + CARD_GAP;
    }
    let left_end = cursor;

    let mut seats: Vec<(f32, f32)> = Vec::new();
    let mut cursor = CARD_GAP;
    for _ in &plan.units {
        let height = TITLE_HEIGHT + 4.0;
        seats.push((cursor, height));
        cursor += height + CARD_GAP;
    }
    let right_end = cursor;

    let width = CARD_WIDTH * 2.0 + COLUMN_GAP + EDGE_ROOM * 2.0;
    let height = left_end.max(right_end) + CARD_GAP;
    let mut paint: Vec<CanvasCommand> = Vec::new();

    let left = EDGE_ROOM;
    let right = EDGE_ROOM + CARD_WIDTH + COLUMN_GAP;

    for (at, unit) in plan.units.iter().enumerate() {
        let Some((top, _)) = seats.get(at) else {
            continue;
        };
        let bright = chosen == Some(at);
        let alpha = match chosen {
            Some(_) if bright => 1.0,
            Some(_) => 0.04,
            None => 0.2,
        };
        let reads = unit
            .reads
            .iter()
            .chain(unit.takes.iter())
            .map(|name| (name, true));
        let writes = unit
            .writes
            .iter()
            .chain(unit.gives.iter())
            .map(|name| (name, false));
        for (name, reading) in reads.chain(writes) {
            let Some(which) = cards.iter().position(|card| &card.name == name) else {
                continue;
            };
            let (card_top, _) = places[which];
            let tone = if reading { READ } else { WRITE };
            let from = Vec2::new(left + CARD_WIDTH, card_top + TITLE_HEIGHT * 0.5);
            let to = Vec2::new(right, top + TITLE_HEIGHT * 0.5);
            let (start, end) = if reading { (from, to) } else { (to, from) };
            let lean = if reading {
                COLUMN_GAP * 0.55
            } else {
                0.0 - COLUMN_GAP * 0.55
            };
            paint.push(CanvasCommand::CubicBezier {
                start,
                control1: start + Vec2::new(lean, 0.0),
                control2: end - Vec2::new(lean, 0.0),
                end,
                thickness: if bright { 2.0 } else { 1.0 },
                color: Vec4::new(tone[0], tone[1], tone[2], alpha),
                id: None,
            });
        }
    }

    for (at, card) in cards.iter().enumerate() {
        let (top, height) = places[at];
        let here = touched.iter().any(|one| one == &card.name);
        let alpha = if chosen.is_none() || here { 1.0 } else { 0.3 };
        let tone = match card.kind {
            Kind::Holder => READ,
            Kind::Shared => SHARED,
            Kind::Signal => ASKED,
        };
        let place = Vec2::new(left, top);
        paint.push(CanvasCommand::Rect {
            position: place,
            size: Vec2::new(CARD_WIDTH, height),
            color: Vec4::new(0.10, 0.12, 0.17, alpha),
            corner_radius: 6.0,
            id: None,
        });
        paint.push(CanvasCommand::Rect {
            position: place,
            size: Vec2::new(CARD_WIDTH, TITLE_HEIGHT),
            color: Vec4::new(tone[0] * 0.30, tone[1] * 0.30, tone[2] * 0.30, alpha),
            corner_radius: 6.0,
            id: None,
        });
        paint.push(CanvasCommand::RectStroke {
            position: place,
            size: Vec2::new(CARD_WIDTH, height),
            color: Vec4::new(
                tone[0],
                tone[1],
                tone[2],
                alpha * if here { 1.0 } else { 0.5 },
            ),
            thickness: if here { 2.0 } else { 1.0 },
            corner_radius: 6.0,
            id: None,
        });
        paint.push(CanvasCommand::Text {
            text: clip(&card.name, 26),
            position: place + Vec2::new(8.0, 5.0),
            font_size: data::CHIP_SIZE,
            color: Vec4::new(tone[0], tone[1], tone[2], alpha),
            id: None,
        });
        for (row, (name, value)) in card.rows.iter().enumerate() {
            let line = place + Vec2::new(8.0, TITLE_HEIGHT + 3.0 + row as f32 * ROW_HEIGHT);
            paint.push(CanvasCommand::Text {
                text: clip(name, 14),
                position: line,
                font_size: data::SMALL_SIZE,
                color: Vec4::new(DIM[0], DIM[1], DIM[2], alpha),
                id: None,
            });
            paint.push(CanvasCommand::Text {
                text: clip(value, 16),
                position: line + Vec2::new(92.0, 0.0),
                font_size: data::SMALL_SIZE,
                color: Vec4::new(CODE[0], CODE[1], CODE[2], alpha),
                id: None,
            });
        }
    }

    for (at, unit) in plan.units.iter().enumerate() {
        let (top, height) = seats[at];
        let here = chosen == Some(at);
        let alpha = if chosen.is_none() || here { 1.0 } else { 0.35 };
        let tone = if here { WRITE } else { CODE };
        let place = Vec2::new(right, top);
        paint.push(CanvasCommand::Rect {
            position: place,
            size: Vec2::new(CARD_WIDTH, height),
            color: Vec4::new(0.16, 0.18, 0.26, alpha),
            corner_radius: 6.0,
            id: Some(MARK as u32 + at as u32),
        });
        paint.push(CanvasCommand::RectStroke {
            position: place,
            size: Vec2::new(CARD_WIDTH, height),
            color: Vec4::new(tone[0], tone[1], tone[2], alpha),
            thickness: if here { 2.0 } else { 1.0 },
            corner_radius: 6.0,
            id: None,
        });
        paint.push(CanvasCommand::Text {
            text: clip(&unit.name, 26),
            position: place + Vec2::new(8.0, 5.0),
            font_size: data::CHIP_SIZE,
            color: Vec4::new(tone[0], tone[1], tone[2], alpha),
            id: None,
        });
    }

    (Vec2::new(width, height), paint)
}

fn detail(tree: &mut UiTreeBuilder, unit: &Unit) {
    tree.add_separator();
    tree.add_heading(&unit.name);

    band(tree, "writes", &unit.writes, WRITE);
    band(tree, "reads", &unit.reads, READ);
    band(tree, "writes shared", &unit.gives, WRITE);
    band(tree, "reads shared", &unit.takes, SHARED);
    band(tree, "asks", &unit.asks, ASKED);

    let width = CARD_WIDTH * 2.0 + COLUMN_GAP + EDGE_ROOM * 2.0;
    let height = unit.lines.len() as f32 * (BLOCK_HEIGHT + 3.0) + 12.0;
    let canvas = tree.add_canvas(Vec2::new(width, height.max(BLOCK_HEIGHT)));
    let mut paint: Vec<CanvasCommand> = Vec::new();

    for (at, (depth, line)) in unit.lines.iter().enumerate() {
        let lead = EDGE_ROOM + *depth as f32 * BLOCK_STEP;
        let top = 6.0 + at as f32 * (BLOCK_HEIGHT + 3.0);
        let tone = tone_of(line);
        paint.push(CanvasCommand::Rect {
            position: Vec2::new(lead, top),
            size: Vec2::new(width - lead - EDGE_ROOM, BLOCK_HEIGHT),
            color: Vec4::new(tone[0] * 0.22, tone[1] * 0.22, tone[2] * 0.22, 1.0),
            corner_radius: 4.0,
            id: None,
        });
        paint.push(CanvasCommand::Rect {
            position: Vec2::new(lead, top),
            size: Vec2::new(3.0, BLOCK_HEIGHT),
            color: Vec4::new(tone[0], tone[1], tone[2], 1.0),
            corner_radius: 2.0,
            id: None,
        });
        paint.push(CanvasCommand::Text {
            text: clip(line, 62),
            position: Vec2::new(lead + 10.0, top + 5.0),
            font_size: data::CHIP_SIZE,
            color: Vec4::new(tone[0], tone[1], tone[2], 1.0),
            id: None,
        });
    }

    tree.edit::<UiCanvasData>(canvas, move |data| {
        data.commands = paint;
    });
}

fn tone_of(line: &str) -> [f32; 4] {
    let word = line.split_whitespace().next().unwrap_or_default();
    match word {
        "when" | "otherwise" => GATE,
        "for" => LOOP,
        "make" | "announce" => MAKE,
        "stop" | "leave" | "take" => HALT,
        _ => WRITE,
    }
}

fn names(unit: &Unit) -> Vec<String> {
    unit.reads
        .iter()
        .chain(unit.writes.iter())
        .chain(unit.takes.iter())
        .chain(unit.gives.iter())
        .cloned()
        .collect()
}

fn clip(name: &str, width: usize) -> String {
    if name.chars().count() <= width {
        return name.to_string();
    }
    let held: String = name.chars().take(width.saturating_sub(1)).collect();
    format!("{held}.")
}

fn band(tree: &mut UiTreeBuilder, title: &str, names: &[String], tone: [f32; 4]) {
    if names.is_empty() {
        return;
    }
    chip(tree, &format!("{title}  {}", names.join(", ")), tone);
}

fn chip(tree: &mut UiTreeBuilder, text: &str, tone: [f32; 4]) {
    tree.label_node(text, data::CHIP_SIZE)
        .color_raw::<UiBase>(Vec4::new(tone[0], tone[1], tone[2], tone[3]))
        .entity();
}
