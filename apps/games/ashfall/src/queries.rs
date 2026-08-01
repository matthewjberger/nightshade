use crate::components::ItemKind;
use crate::data::ask::Ask;
use crate::data::level::*;
use crate::data::notes::{NOTE_SITES, NOTES};
use crate::data::props::EXAMINES;
use crate::data::rooms::ROOMS;
use crate::data::theme::{self, CONDITION_CAUTION, CONDITION_DANGER, CONDITION_FINE};
use crate::resources::{Satchel, SlotAction};
use nightshade::prelude::*;
use nightshade_input::Actions;
use nightshade_platform::prelude::{Input, MouseState};
use std::collections::VecDeque;

pub fn tile(col: i32, row: i32) -> u8 {
    if col < 0 || row < 0 || col >= WIDTH as i32 || row >= HEIGHT as i32 {
        return FACADE;
    }
    MAP[row as usize].as_bytes()[col as usize]
}

pub fn room_at(col: i32, row: i32, upper: bool) -> Option<usize> {
    ROOMS.iter().position(|room| {
        room.upper == upper
            && col >= room.col0
            && col <= room.col1
            && row >= room.row0
            && row <= room.row1
    })
}

pub fn upper_tile(col: i32, row: i32) -> u8 {
    if col < 0 || row < 0 || col >= WIDTH as i32 || row >= HEIGHT as i32 {
        return NOTHING;
    }
    UPPER[row as usize].as_bytes()[col as usize]
}

pub fn tile_on(col: i32, row: i32, upper: bool) -> u8 {
    if upper {
        upper_tile(col, row)
    } else {
        tile(col, row)
    }
}

pub fn tile_index(col: i32, row: i32) -> Option<usize> {
    if col < 0 || row < 0 || col >= WIDTH as i32 || row >= HEIGHT as i32 {
        return None;
    }
    Some(row as usize * WIDTH + col as usize)
}

pub fn is_wall(tile: u8) -> bool {
    tile == FACADE || tile == PARTITION
}

pub fn is_floor(tile: u8) -> bool {
    !is_wall(tile) && tile != NOTHING
}

pub fn is_room(col: i32, row: i32) -> bool {
    let here = tile(col, row);
    if here == ROOM {
        return true;
    }
    if is_wall(here) || here == STREET {
        return false;
    }
    [(1, 0), (-1, 0), (0, 1), (0, -1)]
        .iter()
        .any(|(step_col, step_row)| tile(col + step_col, row + step_row) == ROOM)
}

pub fn door_runs_along_x(col: i32, row: i32, upper: bool) -> bool {
    is_wall(tile_on(col - 1, row, upper)) && is_wall(tile_on(col + 1, row, upper))
}

pub fn tile_center(col: i32, row: i32) -> Vec3 {
    Vec3::new(col as f32 * TILE, 0.0, row as f32 * TILE)
}

pub fn tile_of(position: &Vec3) -> (i32, i32) {
    (
        (position.x / TILE).round() as i32,
        (position.z / TILE).round() as i32,
    )
}

pub fn stair_run(col: i32, row: i32) -> Option<(i32, i32)> {
    if tile(col, row) != STAIR {
        return None;
    }
    for (step_col, step_row) in [(1, 0), (0, 1)] {
        if tile(col + step_col, row + step_row) == STAIR {
            return Some((step_col, step_row));
        }
    }
    None
}

pub fn note_at(col: i32, row: i32, upper: bool) -> Option<usize> {
    NOTE_SITES
        .iter()
        .find(|site| site.col == col && site.row == row && site.upper == upper)
        .map(|site| site.index)
}

pub fn examine_text(index: usize) -> &'static str {
    EXAMINES[index % EXAMINES.len()]
}

pub fn next_step(blocked: &[bool], from: (i32, i32), to: (i32, i32)) -> Option<(i32, i32)> {
    if from == to {
        return None;
    }
    let mut parents = vec![None; WIDTH * HEIGHT];
    let mut seen = vec![false; WIDTH * HEIGHT];
    let start = tile_index(from.0, from.1)?;
    let goal = tile_index(to.0, to.1)?;
    seen[start] = true;
    let mut queue = VecDeque::from([from]);
    while let Some(current) = queue.pop_front() {
        let current_index = tile_index(current.0, current.1)?;
        if current_index == goal {
            let mut step = current;
            loop {
                let index = tile_index(step.0, step.1)?;
                match parents[index] {
                    Some(parent) if parent == from => return Some(step),
                    Some(parent) => step = parent,
                    None => return None,
                }
            }
        }
        for (step_col, step_row) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let next = (current.0 + step_col, current.1 + step_row);
            let Some(index) = tile_index(next.0, next.1) else {
                continue;
            };
            if seen[index] || blocked[index] {
                continue;
            }
            seen[index] = true;
            parents[index] = Some(current);
            queue.push_back(next);
        }
    }
    None
}

pub fn hold_push(input: &Input, actions: &Actions<Ask>) -> f32 {
    input.mouse.wheel_delta.y + actions.axis(Ask::PushHold) * 0.35
}

pub fn carrying(satchel: &Satchel, kind: ItemKind) -> bool {
    satchel.slots.iter().any(|slot| slot.kind == Some(kind))
}

pub fn pairs_into(first: ItemKind, second: ItemKind) -> Option<ItemKind> {
    match (first, second) {
        (ItemKind::BoneHaft, ItemKind::BoneWard) | (ItemKind::BoneWard, ItemKind::BoneHaft) => {
            Some(ItemKind::BoneKey)
        }
        _ => None,
    }
}

pub fn is_a_key(kind: ItemKind) -> bool {
    matches!(
        kind,
        ItemKind::BrassKey | ItemKind::WardKey | ItemKind::BoneKey
    )
}

pub fn chosen_action(index: usize) -> SlotAction {
    SlotAction::ALL[index.min(SlotAction::ALL.len() - 1)]
}

pub fn moved_down(actions: &Actions<Ask>) -> i32 {
    i32::from(actions.just_pressed(Ask::NavigateDown))
        - i32::from(actions.just_pressed(Ask::NavigateUp))
}

pub fn moved_across(actions: &Actions<Ask>) -> i32 {
    i32::from(actions.just_pressed(Ask::NavigateRight))
        - i32::from(actions.just_pressed(Ask::NavigateLeft))
}

pub fn spin_across(input: &Input, actions: &Actions<Ask>) -> f32 {
    let dragged = if input.mouse.state.contains(MouseState::LEFT_CLICKED) {
        input.mouse.position_delta.x * 0.05
    } else {
        0.0
    };
    dragged + actions.axis2(Ask::Spin).x
}

pub fn spin_down(input: &Input, actions: &Actions<Ask>) -> f32 {
    let dragged = if input.mouse.state.contains(MouseState::LEFT_CLICKED) {
        input.mouse.position_delta.y * 0.05
    } else {
        0.0
    };
    dragged - actions.axis2(Ask::Spin).y
}

pub fn item_look(kind: ItemKind) -> (&'static str, Vec3, [f32; 4], [f32; 3]) {
    match kind {
        ItemKind::BrassKey => (
            "Cube",
            Vec3::new(0.5, 0.08, 0.22),
            theme::KEY_COLOR,
            theme::KEY_GLOW,
        ),
        ItemKind::WardKey => (
            "Cube",
            Vec3::new(0.48, 0.08, 0.2),
            theme::WARD_COLOR,
            theme::WARD_GLOW,
        ),
        ItemKind::BoneKey => (
            "Cube",
            Vec3::new(0.55, 0.1, 0.2),
            theme::BONE_COLOR,
            theme::BONE_GLOW,
        ),
        ItemKind::HealthDrink => (
            "Cylinder",
            Vec3::new(0.26, 0.5, 0.26),
            theme::HEALTH_COLOR,
            theme::HEALTH_GLOW,
        ),
        ItemKind::Ammo => (
            "Cube",
            Vec3::new(0.35, 0.22, 0.28),
            theme::AMMO_COLOR,
            theme::AMMO_GLOW,
        ),
        ItemKind::Battery => (
            "Cylinder",
            Vec3::new(0.2, 0.34, 0.2),
            theme::BATTERY_COLOR,
            theme::BATTERY_GLOW,
        ),
        ItemKind::TownMap => (
            "Cube",
            Vec3::new(0.5, 0.03, 0.4),
            theme::MAP_ITEM_COLOR,
            theme::MAP_ITEM_GLOW,
        ),
        ItemKind::Lockbox => (
            "Cube",
            Vec3::new(0.42, 0.3, 0.3),
            theme::LOCKBOX_COLOR,
            theme::LOCKBOX_GLOW,
        ),
        ItemKind::Case => (
            "Cube",
            Vec3::new(0.46, 0.2, 0.34),
            theme::CASE_COLOR,
            theme::CASE_GLOW,
        ),
        ItemKind::BoneHaft => (
            "Cube",
            Vec3::new(0.34, 0.1, 0.16),
            theme::BONE_COLOR,
            theme::BONE_GLOW,
        ),
        ItemKind::BoneWard => (
            "Cube",
            Vec3::new(0.22, 0.12, 0.2),
            theme::BONE_COLOR,
            theme::BONE_GLOW,
        ),
        ItemKind::Note(_) => (
            "Cube",
            Vec3::new(0.45, 0.02, 0.6),
            theme::NOTE_COLOR,
            theme::NOTE_GLOW,
        ),
    }
}

pub fn item_story(kind: ItemKind) -> &'static str {
    match kind {
        ItemKind::BrassKey => {
            "Brass, worn smooth. The bow is stamped with a number that has rubbed away."
        }
        ItemKind::WardKey => "Steel, and a paper tag on a loop of string. The tag reads eleven.",
        ItemKind::BoneKey => {
            "It is not metal. It is warm, and it is the right shape for a hand that is not yours."
        }
        ItemKind::HealthDrink => {
            "A dispensary bottle with the label soaked off. Whatever is in it is dark."
        }
        ItemKind::Ammo => "A card box of rounds. Somebody counted them in pencil on the lid.",
        ItemKind::Battery => "A cell with some charge left. Cold to hold.",
        ItemKind::TownMap => "The streets, drawn by somebody who kept correcting them.",
        ItemKind::Lockbox => "A dispensary lockbox. No keyhole anywhere on the lid or the sides.",
        ItemKind::Case => {
            "A police evidence case. Three brass wheels on the front, and a tag loop with nothing on it."
        }
        ItemKind::BoneHaft => {
            "The handle of something, cut from bone and worn smooth by a hand. It ends in a clean break."
        }
        ItemKind::BoneWard => {
            "The business end of a key, cut from bone. The teeth are cut for one lock and the break is old."
        }
        ItemKind::Note(_) => "Somebody's handwriting.",
    }
}

pub fn condition(health: f32) -> (&'static str, Vec4) {
    if health > 60.0 {
        ("FINE", CONDITION_FINE)
    } else if health > 30.0 {
        ("CAUTION", CONDITION_CAUTION)
    } else {
        ("DANGER", CONDITION_DANGER)
    }
}

pub fn item_name(kind: ItemKind) -> String {
    match kind {
        ItemKind::BrassKey => "Brass Key".to_string(),
        ItemKind::WardKey => "Ward Key".to_string(),
        ItemKind::BoneKey => "Bone Key".to_string(),
        ItemKind::HealthDrink => "Health Drink".to_string(),
        ItemKind::Ammo => "Handgun Bullets".to_string(),
        ItemKind::Battery => "Cell Battery".to_string(),
        ItemKind::TownMap => "Town Map".to_string(),
        ItemKind::Lockbox => "Dispensary Lockbox".to_string(),
        ItemKind::Case => "Evidence Case".to_string(),
        ItemKind::BoneHaft => "Bone Haft".to_string(),
        ItemKind::BoneWard => "Bone Ward".to_string(),
        ItemKind::Note(index) => NOTES[index % NOTES.len()].0.to_string(),
    }
}

pub fn person(index: usize) -> &'static crate::data::people::Person {
    &crate::data::people::PEOPLE[index % crate::data::people::PEOPLE.len()]
}

pub fn note_body(index: usize) -> &'static str {
    NOTES[index % NOTES.len()].1
}

pub fn flat_distance(a: &Vec3, b: &Vec3) -> f32 {
    (a.x - b.x).hypot(a.z - b.z)
}
