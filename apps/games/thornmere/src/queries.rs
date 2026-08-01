pub mod chat;
pub mod items;
pub mod models;
pub mod save;
use crate::data::*;
use crate::resources::Walls;
use nightshade::prelude::{Vec2, Vec3, nalgebra_glm};

pub fn segment_distance(point: Vec2, start: Vec2, end: Vec2) -> f32 {
    let span = end - start;
    let length_squared = span.dot(&span);
    if length_squared <= f32::EPSILON {
        return (point - start).magnitude();
    }
    let along = ((point - start).dot(&span) / length_squared).clamp(0.0, 1.0);
    (point - (start + span * along)).magnitude()
}

pub fn road_distance(spot: Vec2) -> f32 {
    let mut nearest = f32::MAX;
    for road in ROADS {
        for pair in road.windows(2) {
            nearest = nearest.min(segment_distance(spot, pair[0], pair[1]));
        }
    }
    nearest
}

fn rect_outside(spot: Vec2, low: Vec2, high: Vec2) -> f32 {
    let dx = (low.x - spot.x).max(spot.x - high.x).max(0.0);
    let dz = (low.y - spot.y).max(spot.y - high.y).max(0.0);
    (dx * dx + dz * dz).sqrt()
}

pub fn inside_rect(spot: Vec2, low: Vec2, high: Vec2) -> bool {
    spot.x >= low.x && spot.x <= high.x && spot.y >= low.y && spot.y <= high.y
}

fn city_relief(spot: Vec2) -> (f32, f32) {
    let outside = rect_outside(spot, CITY_MIN, CITY_MAX);
    let plateau = CITY_RISE * (1.0 - nalgebra_glm::smoothstep(0.0, CITY_SKIRT, outside));
    let calm = 1.0 - nalgebra_glm::smoothstep(CITY_SKIRT, CITY_SKIRT + MOAT_WIDTH + 4.0, outside);
    let mut relief = plateau;
    if spot.y > CITY_MIN.y + 2.0 {
        let trench = nalgebra_glm::smoothstep(MOAT_BANK, MOAT_BANK + 2.5, outside)
            * (1.0
                - nalgebra_glm::smoothstep(
                    MOAT_BANK + MOAT_WIDTH - 2.5,
                    MOAT_BANK + MOAT_WIDTH,
                    outside,
                ));
        relief -= MOAT_DEPTH * trench;
    }
    if outside <= 0.0 {
        let across = (spot.y - CANAL_Z).abs();
        let reach =
            1.0 - nalgebra_glm::smoothstep(CITY_MAX.x - 8.0, CITY_MAX.x - 4.0, spot.x.abs());
        let canal = (1.0 - nalgebra_glm::smoothstep(CANAL_HALF, CANAL_HALF + 1.3, across)) * reach;
        relief -= CANAL_DEPTH * canal;
        let keep_outside = rect_outside(spot, KEEP_MIN, KEEP_MAX);
        relief += KEEP_RISE * (1.0 - nalgebra_glm::smoothstep(0.0, 3.0, keep_outside));
    }
    (relief, calm)
}

fn pit_relief(spot: Vec2) -> f32 {
    let mut relief = 0.0;
    for pit in &PITS {
        let outside = rect_outside(spot, pit.min, pit.max);
        relief -= pit.depth * (1.0 - nalgebra_glm::smoothstep(0.0, 3.0, outside));
    }
    relief
}

pub fn terrain_height(x: f32, z: f32) -> f32 {
    let spot = Vec2::new(x, z);
    let mut relief = 0.0;
    let mut calm: f32 = 0.0;
    for place in &PLACES {
        let distance = (spot - place.center).magnitude();
        if place.height != 0.0 {
            let inner = place.radius * 0.3;
            relief +=
                place.height * (1.0 - nalgebra_glm::smoothstep(inner, place.radius, distance));
        }
        let settle =
            1.0 - nalgebra_glm::smoothstep(place.radius * 0.55, place.radius * 1.1, distance);
        calm = calm.max(settle);
    }
    let road = 1.0 - nalgebra_glm::smoothstep(ROAD_WIDTH, ROAD_WIDTH * 2.6, road_distance(spot));
    calm = calm.max(road);
    let (city, city_calm) = city_relief(spot);
    calm = calm.max(city_calm);
    relief += city + pit_relief(spot);

    let roll = (x * 0.09).sin() * (z * 0.07).cos() * 1.2
        + (x * 0.031 + 1.7).sin() * (z * 0.043).sin() * 0.9
        + (x * 0.17 + z * 0.11).sin() * 0.25;
    relief + roll * (1.0 - 0.88 * calm)
}

pub fn terrain_normal(x: f32, z: f32) -> Vec3 {
    let step = 0.5;
    let slope_x = (terrain_height(x + step, z) - terrain_height(x - step, z)) / (2.0 * step);
    let slope_z = (terrain_height(x, z + step) - terrain_height(x, z - step)) / (2.0 * step);
    Vec3::new(-slope_x, 1.0, -slope_z).normalize()
}

pub fn moat_bridge_span() -> (f32, f32) {
    (
        CITY_MAX.y + WALL_SCALE / 2.0 - 0.3,
        CITY_MAX.y + MOAT_BANK + MOAT_WIDTH + 0.8,
    )
}

pub fn bridge_deck(spot: Vec2) -> Option<f32> {
    let (start, end) = moat_bridge_span();
    if spot.x.abs() <= MOAT_BRIDGE_HALF_WIDTH && spot.y >= start && spot.y <= end {
        return Some(terrain_height(0.0, CITY_MAX.y - 2.0));
    }
    let across = (spot.y - CANAL_Z).abs();
    let on_span = CANAL_BRIDGES
        .iter()
        .any(|bridge| (spot.x - bridge).abs() <= CANAL_BRIDGE_HALF_WIDTH);
    if across <= CANAL_BRIDGE_HALF_SPAN && on_span {
        return Some(terrain_height(15.0, CANAL_Z - 8.0));
    }
    None
}

pub fn footing(x: f32, z: f32) -> f32 {
    let ground = terrain_height(x, z);
    match bridge_deck(Vec2::new(x, z)) {
        Some(deck) => deck.max(ground),
        None => ground.max(WATER_LEVEL - SWIM_DRAFT),
    }
}

pub fn swimming(x: f32, z: f32) -> bool {
    bridge_deck(Vec2::new(x, z)).is_none() && terrain_height(x, z) < WATER_LEVEL - SWIM_DRAFT
}

pub fn ground(spot: Vec2) -> Vec3 {
    Vec3::new(spot.x, terrain_height(spot.x, spot.y), spot.y)
}

pub fn flat(position: Vec3) -> Vec2 {
    Vec2::new(position.x, position.z)
}

pub fn flat_distance(first: Vec3, second: Vec3) -> f32 {
    (flat(first) - flat(second)).magnitude()
}

pub fn heading_toward(from: Vec3, to: Vec3) -> f32 {
    (from.x - to.x).atan2(from.z - to.z)
}

pub fn is_alive(health: f32) -> bool {
    health > 0.0
}

pub fn level_threshold(level: u32) -> u32 {
    80 + 45 * level
}

pub fn sim_max_health(level: u32) -> f32 {
    90.0 + 10.0 * level as f32
}

pub fn sim_strength(level: u32) -> f32 {
    9.0 + 1.4 * level as f32
}

pub fn hunt_for_level(level: u32) -> usize {
    HUNTS
        .iter()
        .rposition(|hunt| level >= hunt.min_level)
        .unwrap_or(0)
}

pub fn person_plate(name: &str, guild: &str, level: u32) -> String {
    format!("<{guild}> {level}\n{name}")
}

pub fn player_plate(name: &str, level: u32) -> String {
    format!("Level {level}\n{name}")
}

pub fn mob_plate(name: &str, level: u32, rank: u8) -> String {
    let tag = if rank >= ELITE { " Elite" } else { "" };
    format!("Level {level}{tag}\n{name}")
}

pub fn giver_plate(giver: usize, mark: u8) -> String {
    let data = &GIVERS[giver];
    let sign = match mark {
        1 => "!",
        2 => "?",
        _ => "",
    };
    format!("<{}>\n{}\n{sign}", data.title, data.name)
}

pub fn place_at(position: Vec3) -> Option<usize> {
    let spot = flat(position);
    let mut best = None;
    let mut best_fraction = 1.0;
    for (index, place) in PLACES.iter().enumerate() {
        let fraction = (spot - place.center).magnitude() / place.radius;
        if fraction < best_fraction {
            best_fraction = fraction;
            best = Some(index);
        }
    }
    best
}

pub fn zone_clamp(position: Vec3) -> Vec3 {
    let limit = ZONE_SIZE / 2.0 - ZONE_MARGIN;
    Vec3::new(
        position.x.clamp(-limit, limit),
        position.y,
        position.z.clamp(-limit, limit),
    )
}

pub fn near_hamlet(spot: Vec2) -> bool {
    (spot - HAMLET).magnitude() < HAMLET_CLEARING
}

pub fn far_enough(spot: Vec2, taken: &[Vec2], gap: f32) -> bool {
    taken.iter().all(|other| (spot - other).magnitude() >= gap)
}

pub fn in_place(spot: Vec2, place: usize, fraction: f32) -> bool {
    let data = &PLACES[place];
    (spot - data.center).magnitude() < data.radius * fraction
}

pub fn near_anything_built(spot: Vec2) -> bool {
    road_distance(spot) < ROAD_WIDTH * 2.2
        || in_city(spot, MOAT_WIDTH + 6.0)
        || in_place(spot, FIELD, 1.0)
        || in_place(spot, STARTER, 1.0)
        || in_place(spot, FORT, 1.1)
        || in_place(spot, BARROW, 1.1)
        || in_place(spot, LAKE, 1.15)
        || in_place(spot, FARM, 1.1)
        || in_place(spot, MINE, 1.05)
        || in_place(spot, HARBOR, 1.2)
}

pub fn wall_blocks(walls: &Walls, spot: Vec2) -> bool {
    walls
        .segments
        .iter()
        .any(|(from, to)| segment_distance(spot, *from, *to) < BODY_RADIUS)
}

pub fn in_city(spot: Vec2, margin: f32) -> bool {
    rect_outside(spot, CITY_MIN, CITY_MAX) < margin
}

pub fn under_water(spot: Vec2) -> bool {
    terrain_height(spot.x, spot.y) < WATER_LEVEL + 0.3
}

pub fn hour_text(hour: f32) -> String {
    let total_minutes = ((hour.rem_euclid(24.0)) * 60.0) as u32;
    format!("{:02}:{:02}", total_minutes / 60, total_minutes % 60)
}

pub fn coin_text(copper: u32) -> String {
    let gold = copper / 10_000;
    let silver = (copper / 100) % 100;
    let copper = copper % 100;
    if gold > 0 {
        format!("{gold}g {silver}s {copper}c")
    } else if silver > 0 {
        format!("{silver}s {copper}c")
    } else {
        format!("{copper}c")
    }
}

pub fn bar(fraction: f32, width: usize) -> String {
    let filled = ((fraction.clamp(0.0, 1.0) * width as f32).round()) as usize;
    let mut text = String::with_capacity(width);
    for index in 0..width {
        text.push(if index < filled { '|' } else { '.' });
    }
    text
}

pub fn night_amount(hour: f32) -> f32 {
    let hour = hour.rem_euclid(24.0);
    if hour < 5.0 || hour > 21.0 {
        1.0
    } else if hour < 7.0 {
        1.0 - (hour - 5.0) / 2.0
    } else if hour > 19.0 {
        (hour - 19.0) / 2.0
    } else {
        0.0
    }
}

pub fn offline_fraction(hour: f32) -> f32 {
    let hour = hour.rem_euclid(24.0);
    if !(6.0..=23.0).contains(&hour) {
        OFFLINE_FRACTION_NIGHT
    } else if hour > 20.5 || hour < 8.0 {
        OFFLINE_FRACTION_EVENING
    } else {
        0.0
    }
}

pub fn sheet(class: usize) -> Sheet {
    let data = &CALLINGS[class];
    Sheet {
        title: data.name.to_string(),
        blurb: data.blurb.to_string(),
        stats: [
            format!("Health {}", data.health),
            format!("Mana {}", data.mana),
            format!("Strength {}", data.strength),
            format!("Speed {:.1}", data.speed),
        ],
        abilities: data.abilities.map(|pick| ABILITIES[pick].name.to_string()),
    }
}

pub fn sun_pose(hour: f32) -> (Vec3, f32, Vec3) {
    let angle = (hour - 6.0) / 12.0 * std::f32::consts::PI;
    let raw = Vec3::new(-angle.cos(), angle.sin(), -0.3).normalize();
    if raw.y <= 0.0 {
        return (MOON_DIRECTION.normalize(), MOON_POWER, MOON_COLOR);
    }
    let lifted = Vec3::new(raw.x, raw.y.max(SUN_FLOOR), raw.z).normalize();
    let height = (raw.y / 0.35).clamp(0.0, 1.0);
    let blend = height * height * (3.0 - 2.0 * height);
    let color = SUN_WARM + (SUN_WHITE - SUN_WARM) * blend;
    (lifted, SUN_POWER * lifted.y.sqrt(), color)
}

pub fn item_index(name: &str) -> Option<usize> {
    crate::data::items::ITEMS
        .iter()
        .position(|item| item.name == name)
}

pub fn buy_price(item: usize) -> u32 {
    crate::data::items::ITEMS[item].value * crate::data::items::BUY_MARKUP
}

pub fn gear_totals(gear: &crate::resources::Gear) -> (f32, f32, f32) {
    let mut power = 0.0;
    let mut guard = 0.0;
    let mut spirit = 0.0;
    for worn in gear.worn.iter().flatten() {
        let item = &crate::data::items::ITEMS[*worn];
        power += item.power;
        guard += item.guard;
        spirit += item.spirit;
    }
    (power, guard, spirit)
}

pub fn item_note(item: usize) -> String {
    let data = &crate::data::items::ITEMS[item];
    let mut parts = Vec::new();
    if data.wear != crate::data::items::Wear::Carried {
        parts.push(data.wear.name().to_string());
    }
    if data.power > 0.0 {
        parts.push(format!("+{:.0} strength", data.power));
    }
    if data.guard > 0.0 {
        parts.push(format!("+{:.0} health", data.guard));
    }
    if data.spirit > 0.0 {
        parts.push(format!("+{:.0} mana", data.spirit));
    }
    match data.sip {
        crate::data::items::Sip::None => {}
        crate::data::items::Sip::Health(amount) => parts.push(format!("restores {amount} health")),
        crate::data::items::Sip::Mana(amount) => parts.push(format!("restores {amount} mana")),
    }
    if data.level > 1 {
        parts.push(format!("level {}", data.level));
    }
    parts.push(coin_text(data.value));
    parts.join("   ")
}
