use crate::resources::Dungeon;
use crate::theme;
use nalgebra_glm::Vec2;

pub fn cell_of(dungeon: &Dungeon, point: Vec2) -> Option<(u32, u32)> {
    let column = (point.x - dungeon.origin.x).floor();
    let row = (point.y - dungeon.origin.y).floor();
    if column < 0.0 || row < 0.0 || column >= dungeon.columns as f32 || row >= dungeon.rows as f32 {
        return None;
    }
    Some((column as u32, row as u32))
}

pub fn index(dungeon: &Dungeon, column: u32, row: u32) -> usize {
    (row * dungeon.columns + column) as usize
}

pub fn is_floor(dungeon: &Dungeon, column: u32, row: u32) -> bool {
    dungeon.floor[index(dungeon, column, row)]
}

pub fn floor_at(dungeon: &Dungeon, point: Vec2) -> bool {
    cell_of(dungeon, point).is_some_and(|(column, row)| is_floor(dungeon, column, row))
}

pub fn center(dungeon: &Dungeon, column: u32, row: u32) -> Vec2 {
    dungeon.origin + Vec2::new(column as f32 + 0.5, row as f32 + 0.5)
}

pub fn can_step(dungeon: &Dungeon, to: Vec2) -> bool {
    let margin = 0.28;
    floor_at(dungeon, to)
        && floor_at(dungeon, to + Vec2::new(margin, 0.0))
        && floor_at(dungeon, to - Vec2::new(margin, 0.0))
        && floor_at(dungeon, to + Vec2::new(0.0, margin))
        && floor_at(dungeon, to - Vec2::new(0.0, margin))
}

pub fn neighbors(dungeon: &Dungeon, column: u32, row: u32) -> [Option<(u32, u32)>; 4] {
    let left = (column > 0).then(|| (column - 1, row));
    let right = (column + 1 < dungeon.columns).then(|| (column + 1, row));
    let back = (row > 0).then(|| (column, row - 1));
    let front = (row + 1 < dungeon.rows).then(|| (column, row + 1));
    [left, right, back, front]
}

pub fn beside_floor(dungeon: &Dungeon, column: u32, row: u32) -> Option<(u32, u32)> {
    neighbors(dungeon, column, row)
        .into_iter()
        .flatten()
        .find(|(c, r)| is_floor(dungeon, *c, *r))
}

pub fn level_for(xp: u32) -> u32 {
    1 + xp / crate::theme::XP_PER_LEVEL
}

pub fn damage_of(hero: &crate::resources::Hero) -> u32 {
    hero.weapon.damage()
        + hero.level / 2
        + if hero.strength > 0.0 {
            theme::STRENGTH_BONUS
        } else {
            0
        }
}
