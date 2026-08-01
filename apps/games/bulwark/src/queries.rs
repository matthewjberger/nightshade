use crate::contract::{Cell, Scenery, Tower};
use crate::data::{self, Grade, Wave};
use crate::resources::Bulwark;
use crate::theme;
use nalgebra_glm::Vec2;
use nightshade::prelude::Entity;
use nightshade_waves::WaveProgress;

pub const NORTHEAST: u8 = 1;
pub const SOUTHEAST: u8 = 2;
pub const SOUTHWEST: u8 = 4;
pub const NORTHWEST: u8 = 8;

pub fn cell_center(cell: Cell) -> Vec2 {
    let column = cell.column as f32;
    let row = cell.row as f32;
    Vec2::new(
        (column - row) * theme::HALF_STEP_ACROSS,
        -(column + row) * theme::HALF_STEP_INTO,
    )
}

pub fn cell_under(point: Vec2) -> Option<Cell> {
    let across = point.x / theme::HALF_STEP_ACROSS;
    let into = point.y / theme::HALF_STEP_INTO;
    let column = ((across - into) * 0.5).round();
    let row = ((-across - into) * 0.5).round();
    if column < 0.0 || row < 0.0 || column >= data::COLUMNS as f32 || row >= data::ROWS as f32 {
        return None;
    }
    Some(Cell::new(column as u32, row as u32))
}

pub fn cells_beside(cell: Cell) -> Vec<Cell> {
    let mut beside = Vec::new();
    if cell.column > 0 {
        beside.push(Cell::new(cell.column - 1, cell.row));
    }
    if cell.row > 0 {
        beside.push(Cell::new(cell.column, cell.row - 1));
    }
    if cell.column + 1 < data::COLUMNS {
        beside.push(Cell::new(cell.column + 1, cell.row));
    }
    if cell.row + 1 < data::ROWS {
        beside.push(Cell::new(cell.column, cell.row + 1));
    }
    beside
}

pub fn cell_index(cell: Cell) -> usize {
    (cell.row * data::COLUMNS + cell.column) as usize
}

pub fn board_of(board: usize) -> &'static data::Board {
    &data::BOARDS[board.min(data::BOARDS.len() - 1)]
}

pub fn boards_total() -> usize {
    data::BOARDS.len()
}

pub fn board_name(board: usize) -> &'static str {
    board_of(board).name
}

pub fn road_step(board: usize, cell: Cell) -> Option<usize> {
    board_of(board).road.iter().position(|road| *road == cell)
}

pub fn is_road(board: usize, cell: Cell) -> bool {
    road_step(board, cell).is_some()
}

pub fn road_mask(board: usize, cell: Cell) -> u8 {
    let mut mask = 0;
    for neighbour in board_of(board).road.iter().copied() {
        if neighbour.row == cell.row && neighbour.column + 1 == cell.column {
            mask |= NORTHWEST;
        }
        if neighbour.row == cell.row && cell.column + 1 == neighbour.column {
            mask |= SOUTHEAST;
        }
        if neighbour.column == cell.column && neighbour.row + 1 == cell.row {
            mask |= NORTHEAST;
        }
        if neighbour.column == cell.column && cell.row + 1 == neighbour.row {
            mask |= SOUTHWEST;
        }
    }
    if mask == 0 || mask == NORTHEAST || mask == SOUTHWEST {
        return NORTHEAST | SOUTHWEST;
    }
    if mask == SOUTHEAST || mask == NORTHWEST {
        return SOUTHEAST | NORTHWEST;
    }
    mask
}

pub fn cell_noise(cell: Cell) -> f32 {
    let mixed = cell
        .column
        .wrapping_mul(73_856_093)
        .wrapping_add(cell.row.wrapping_mul(19_349_663))
        .wrapping_mul(2_654_435_761)
        >> 11;
    (mixed % 1000) as f32 / 1000.0
}

pub fn grass_tint(cell: Cell) -> [f32; 4] {
    let draw = cell_noise(cell);
    let shade = 0.94 + draw * 0.12;
    [shade, 0.97 + draw * 0.06, shade * 0.98, 1.0]
}

pub fn scenery_on(board: usize, cell: Cell) -> Option<Scenery> {
    board_of(board)
        .scatter
        .iter()
        .find(|(scattered, _)| *scattered == cell)
        .map(|(_, scenery)| *scenery)
}

pub fn keep_cell(board: usize) -> Cell {
    let road = board_of(board).road;
    road[road.len() - 1]
}

pub fn spawn_center(board: usize) -> Vec2 {
    cell_center(board_of(board).road[0])
}

pub fn leg_center(board: usize, leg: usize, progress: f32) -> Vec2 {
    let road = board_of(board).road;
    let last = road.len() - 1;
    let from = cell_center(road[leg.min(last)]);
    let to = cell_center(road[(leg + 1).min(last)]);
    from + (to - from) * progress.clamp(0.0, 1.0)
}

pub fn leg_length(board: usize, leg: usize) -> f32 {
    let road = board_of(board).road;
    let last = road.len() - 1;
    let from = cell_center(road[leg.min(last)]);
    let to = cell_center(road[(leg + 1).min(last)]);
    (to - from).magnitude().max(f32::EPSILON)
}

pub fn last_leg(board: usize) -> usize {
    board_of(board).road.len() - 1
}

pub fn grade_of(tower: Tower, level: u8) -> Grade {
    let step = (level.max(1) as usize - 1).min(2);
    data::GRADES[tower.slot()][step]
}

pub fn wave_of(index: usize) -> Wave {
    data::WAVES[index.min(data::WAVES.len() - 1)]
}

pub fn planned_wave(index: usize) -> nightshade_waves::Wave {
    let wave = wave_of(index);
    nightshade_waves::Wave {
        count: wave.count + u32::from(wave.boss),
        spacing: wave.spacing,
        rest: 0.0,
    }
}

pub fn wave_plan(rest: f32) -> Vec<nightshade_waves::Wave> {
    (0..waves_total())
        .map(|index| nightshade_waves::Wave {
            rest,
            ..planned_wave(index)
        })
        .collect()
}

pub fn waves_total() -> usize {
    data::WAVES.len()
}

pub fn build_cost(tower: Tower) -> u32 {
    grade_of(tower, 1).cost
}

pub fn upgrade_cost(tower: Tower, level: u8) -> Option<u32> {
    if level >= 3 {
        return None;
    }
    Some(grade_of(tower, level + 1).cost)
}

pub fn sell_value(tower: Tower, level: u8) -> u32 {
    let spent: u32 = (1..=level).map(|step| grade_of(tower, step).cost).sum();
    (spent as f32 * theme::TOWER_SELL_SHARE) as u32
}

pub fn buildable(bulwark: &Bulwark, cell: Cell) -> bool {
    !is_road(bulwark.board, cell)
        && scenery_on(bulwark.board, cell).is_none()
        && bulwark
            .taken
            .get(cell_index(cell))
            .is_some_and(|slot| slot.is_none())
}

pub fn tower_on(bulwark: &Bulwark, cell: Cell) -> Option<Entity> {
    bulwark.taken.get(cell_index(cell)).copied().flatten()
}

pub fn affordable(bulwark: &Bulwark, cost: u32) -> bool {
    bulwark.coin >= cost
}

pub fn pace(bulwark: &Bulwark) -> f32 {
    theme::PACE_STEPS[bulwark.pace_step.min(theme::PACE_STEPS.len() - 1)]
}

pub fn pace_label(bulwark: &Bulwark) -> String {
    format!("x{:.0}", pace(bulwark))
}

pub fn bar_color(share: f32) -> [f32; 4] {
    if share > 0.6 {
        theme::BAR_COLOR
    } else if share > 0.28 {
        theme::BAR_HURT
    } else {
        theme::BAR_LOW
    }
}

pub fn wave_label(progress: &WaveProgress) -> String {
    if progress.wave + 1 > waves_total() {
        return format!("{}", progress.wave + 1);
    }
    format!("{} / {}", progress.wave + 1, waves_total())
}

pub fn banner_note(bulwark: &Bulwark, progress: &WaveProgress) -> String {
    let wave = wave_of(progress.wave);
    let mut said = Vec::new();
    if progress.resting {
        if wave.ward > 0.0 {
            said.push("Warded");
        }
        if wave.splits > 0 {
            said.push("they break apart");
        }
        if wave.boss {
            said.push("one walks in heavy");
        }
    }
    if said.is_empty() {
        return board_name(bulwark.board).to_string();
    }
    said.join("   ")
}

pub fn banner_line(progress: &WaveProgress) -> String {
    let wave = wave_of(progress.wave);
    if progress.resting {
        return format!(
            "Wave {} rolls in {:.0}   {} {}",
            progress.wave + 1,
            progress.rest.max(0.0).ceil(),
            wave.count,
            wave.tier.name()
        );
    }
    format!(
        "Wave {}   {} still walking",
        progress.wave + 1,
        progress.alive
    )
}

pub fn waves_held(bulwark: &Bulwark, progress: &WaveProgress) -> u32 {
    if bulwark.lives == 0 {
        return progress.wave as u32;
    }
    progress.wave as u32 + 1
}

pub fn best_run() -> u32 {
    nightshade_save::prelude::save_read::<crate::data::Record>(theme::RECORD_SLOT)
        .map(|record| record.best)
        .unwrap_or(0)
}

pub fn best_line(best: u32) -> String {
    match best {
        0 => "No wave held yet.".to_string(),
        1 => "Best run so far: one wave held.".to_string(),
        many => format!("Best run so far: {many} waves held."),
    }
}

pub fn opening_coin(difficulty: usize) -> u32 {
    let share = data::DIFFICULTY_PURSE[difficulty.min(data::DIFFICULTY_PURSE.len() - 1)];
    (theme::STARTING_COIN as f32 * share) as u32
}

pub fn opening_wave(asked: usize) -> usize {
    asked.max(1).min(waves_total()) - 1
}

pub fn wall_level(wave: usize) -> u8 {
    match wave {
        0..=3 => 1,
        4..=7 => 2,
        _ => 3,
    }
}

pub fn skipped_coin(wave: usize) -> u32 {
    theme::SKIPPED_PURSE * wave as u32
}

pub fn mend_cost(mends: u32) -> u32 {
    theme::MEND_COST + theme::MEND_STEP * mends
}

pub fn mend_label(bulwark: &Bulwark) -> String {
    if bulwark.lives >= theme::STARTING_LIVES {
        return "Keep is whole".to_string();
    }
    format!("Mend {}", mend_cost(bulwark.mends))
}

pub fn mendable(bulwark: &Bulwark) -> bool {
    bulwark.lives < theme::STARTING_LIVES && bulwark.coin >= mend_cost(bulwark.mends)
}

pub fn wave_scale(wave: usize) -> f32 {
    let over = wave.saturating_sub(waves_total() - 1);
    1.0 + 0.3 * over as f32
}

pub fn wave_bounty(wave: usize) -> u32 {
    theme::WAVE_BOUNTY + theme::WAVE_BOUNTY_STEP * wave as u32
}

pub fn focus_label(focus: u8) -> &'static str {
    match focus {
        1 => "Aim at the leader",
        2 => "Aim at the strongest",
        _ => "Aim at the nearest",
    }
}

pub fn inspect_name(tower: Tower, tally: u32) -> String {
    match tally {
        0 => tower.name().to_string(),
        1 => format!("{}   broke one", tower.name()),
        many => format!("{}   broke {many}", tower.name()),
    }
}

pub fn inspect_stats(tower: Tower, level: u8) -> String {
    let grade = grade_of(tower, level);
    format!(
        "level {}   hit {:.0}   every {:.2}s",
        level, grade.damage, grade.reload
    )
}

pub fn inspect_reach(tower: Tower, level: u8) -> String {
    let grade = grade_of(tower, level);
    match upgrade_cost(tower, level) {
        Some(cost) => format!("reach {:.1}   raise for {}", grade.reach, cost),
        None => format!("reach {:.1}   fully raised", grade.reach),
    }
}

fn best_mark(beaten: bool) -> &'static str {
    if beaten { "   a new best" } else { "" }
}

pub fn defeat_line(bulwark: &Bulwark, progress: &WaveProgress, beaten: bool) -> String {
    format!(
        "{} crystals broken, {} slipped past, on wave {}{}",
        bulwark.kills,
        bulwark.leaked,
        progress.wave + 1,
        best_mark(beaten)
    )
}

pub fn victory_line(bulwark: &Bulwark, beaten: bool) -> String {
    format!(
        "{} crystals broken with {} lives standing{}",
        bulwark.kills,
        bulwark.lives,
        best_mark(beaten)
    )
}
