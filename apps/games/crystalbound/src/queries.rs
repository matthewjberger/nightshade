use crate::contract::Place;
use crate::data;
use crate::resources::{Battle, Field, Party};
use crate::theme;
use nalgebra_glm::Vec2;

pub fn tile_of(field: &Field, position: Vec2) -> (i32, i32) {
    let origin = field.origin();
    (
        (position.x - origin.x).floor() as i32,
        (position.y - origin.y).floor() as i32,
    )
}

pub fn world_of(field: &Field, column: u32, row: u32) -> Vec2 {
    let origin = field.origin();
    Vec2::new(origin.x + column as f32 + 0.5, origin.y + row as f32 + 0.5)
}

pub fn walkable(field: &Field, position: Vec2) -> bool {
    let radius = theme::BODY_RADIUS;
    let corners = [
        Vec2::new(position.x - radius, position.y - radius),
        Vec2::new(position.x + radius, position.y - radius),
        Vec2::new(position.x - radius, position.y + radius),
        Vec2::new(position.x + radius, position.y + radius),
    ];
    corners.iter().all(|corner| {
        let (column, row) = tile_of(field, *corner);
        !field.is_blocked(column, row)
    })
}

pub fn portal_at(field: &Field, position: Vec2) -> Option<Place> {
    let (column, row) = tile_of(field, position);
    field
        .portals
        .iter()
        .find(|portal| portal.tile.x as i32 == column && portal.tile.y as i32 == row)
        .map(|portal| portal.to)
}

pub fn spot_in(place: Place, mark: char, drop: i32) -> Option<Vec2> {
    let map = data::map_of(place);
    let rows = map.len() as u32;
    let columns = map[0].len() as u32;
    let origin = Vec2::new(-(columns as f32) / 2.0, -(rows as f32) / 2.0);
    for (line, text) in map.iter().enumerate() {
        let Some(column) = text.chars().position(|found| found == mark) else {
            continue;
        };
        let row = rows as i32 - 1 - line as i32 + drop;
        return Some(Vec2::new(
            origin.x + column as f32 + 0.5,
            origin.y + row as f32 + 0.5,
        ));
    }
    None
}

pub fn arrival(from: Place, to: Place) -> Option<Vec2> {
    match (from, to) {
        (Place::Town, Place::Overworld) => spot_in(Place::Overworld, 't', -1),
        (Place::Dungeon, Place::Overworld) => spot_in(Place::Overworld, 'd', -1),
        _ => None,
    }
}

pub fn stage_center() -> Vec2 {
    theme::STAGE_ORIGIN
}

pub fn foe_spot(index: usize, count: usize) -> Vec2 {
    let middle = (count.max(1) as f32 - 1.0) / 2.0;
    Vec2::new(
        theme::STAGE_ORIGIN.x + theme::STAGE_FOE_X,
        theme::STAGE_ORIGIN.y + (middle - index as f32) * theme::STAGE_ROW,
    )
}

pub fn hero_spot(index: usize, count: usize) -> Vec2 {
    let middle = (count.max(1) as f32 - 1.0) / 2.0;
    Vec2::new(
        theme::STAGE_ORIGIN.x + theme::STAGE_HERO_X,
        theme::STAGE_ORIGIN.y + (middle - index as f32) * theme::STAGE_ROW,
    )
}

pub fn actor_spot(battle: &Battle, party: &Party, foe: bool, slot: usize) -> Vec2 {
    if foe {
        foe_spot(slot, battle.foes.len())
    } else {
        hero_spot(slot, party.heroes.len())
    }
}

pub fn living_foe(battle: &Battle) -> Option<usize> {
    battle.foes.iter().position(|foe| foe.alive())
}

pub fn foes_standing(battle: &Battle) -> usize {
    battle.foes.iter().filter(|foe| foe.alive()).count()
}

pub fn heroes_standing(party: &Party) -> usize {
    party.heroes.iter().filter(|hero| hero.alive()).count()
}

pub fn weakest_hero(party: &Party) -> Option<usize> {
    party
        .heroes
        .iter()
        .enumerate()
        .filter(|(_, hero)| hero.alive())
        .min_by_key(|(_, hero)| hero.max_health - hero.health)
        .map(|(index, _)| index)
}

pub fn neediest_hero(party: &Party) -> Option<usize> {
    party
        .heroes
        .iter()
        .enumerate()
        .filter(|(_, hero)| hero.alive())
        .max_by_key(|(_, hero)| hero.max_health - hero.health)
        .map(|(index, _)| index)
}

pub fn hero_strike(party: &Party, index: usize, foe_guard: i32) -> i32 {
    let Some(hero) = party.heroes.get(index) else {
        return 0;
    };
    let power = hero.power() + party.blade_level as i32 * 3;
    (power - foe_guard).max(1)
}

pub fn foe_strike(battle: &Battle, foe: usize, party: &Party, hero: usize) -> i32 {
    let Some(foe) = battle.foes.get(foe) else {
        return 0;
    };
    let Some(hero) = party.heroes.get(hero) else {
        return 0;
    };
    let power = data::monster_stats(foe.kind).power;
    let guard = hero.guard() + party.guard_level as i32 * 2;
    (power - guard).max(1)
}

pub fn hero_card(party: &Party, index: usize) -> String {
    let Some(hero) = party.heroes.get(index) else {
        return String::new();
    };
    if !hero.alive() {
        return format!("{}  down", hero.name);
    }
    format!(
        "{}  {}\nL{}  HP {}/{}  MP {}/{}",
        hero.name,
        hero.job.name(),
        hero.level,
        hero.health,
        hero.max_health,
        hero.magic,
        hero.max_magic,
    )
}

pub fn purse_line(party: &Party) -> String {
    format!(
        "Gold {}   Shards {}   Potions {}   Blade {}   Guard {}",
        party.gold, party.shards, party.potions, party.blade_level, party.guard_level
    )
}

pub fn foe_row(battle: &Battle, index: usize) -> String {
    let Some(foe) = battle.foes.get(index) else {
        return String::new();
    };
    if !foe.alive() {
        return format!("{}  felled", foe.kind.name());
    }
    format!("{}  {}/{}", foe.kind.name(), foe.health, foe.max_health)
}
