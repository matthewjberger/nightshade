use crate::data::*;
use crate::queries::*;
use nightshade::prelude::Vec2;
use nightshade_random::prelude::*;

pub fn random_line(random: &mut Random, lines: &[&str]) -> String {
    lines[random.usize(0..lines.len())].to_string()
}

pub fn random_name(random: &mut Random) -> String {
    let start = NAME_STARTS[random.usize(0..NAME_STARTS.len())];
    let end = NAME_ENDS[random.usize(0..NAME_ENDS.len())];
    format!("{start}{end}")
}

pub fn random_place_spot(random: &mut Random, place: usize, fraction: f32) -> Vec2 {
    let data = &PLACES[place];
    let angle = random.f32() * std::f32::consts::TAU;
    let radius = random.f32().sqrt() * data.radius * fraction;
    data.center + Vec2::new(angle.cos(), angle.sin()) * radius
}

pub fn random_zone_spot(random: &mut Random) -> Vec2 {
    let limit = ZONE_SIZE / 2.0 - 6.0;
    Vec2::new(
        between(random, -limit, limit),
        between(random, -limit, limit),
    )
}

pub fn roll_drop(random: &mut Random, prey: &str) -> Option<usize> {
    let mut found = None;
    for drop in crate::data::items::DROPS.iter() {
        if drop.prey != prey {
            continue;
        }
        if random.f32() < drop.chance {
            found = item_index(drop.item);
            break;
        }
    }
    found
}

pub fn spread_spot(
    random: &mut Random,
    taken: &[Vec2],
    gap: f32,
    mut pick: impl FnMut(&mut Random) -> Vec2,
) -> Vec2 {
    let mut spot = pick(random);
    for _ in 0..WILD_TRIES {
        if far_enough(spot, taken, gap) {
            break;
        }
        spot = pick(random);
    }
    spot
}

pub fn stylize(text: &str, voice: usize, random: &mut Random) -> String {
    let mut line = text.to_string();
    match voice {
        0 => {
            line = line.to_lowercase();
            while line.ends_with('.') {
                line.pop();
            }
        }
        1 => {
            let mut characters = line.chars();
            if let Some(first) = characters.next() {
                line = first.to_uppercase().collect::<String>() + characters.as_str();
            }
            if !line.ends_with(['.', '!', '?']) {
                line.push('.');
            }
        }
        2 => {
            while line.ends_with('.') {
                line.pop();
            }
            line.push_str(if random.f32() < 0.5 { "!" } else { "!!" });
        }
        3 => {
            line = line.to_lowercase();
            if random.f32() < 0.4 {
                line.push_str(" lol");
            }
        }
        _ => {}
    }
    line
}

pub fn wild_spot(
    random: &mut Random,
    taken: &[Vec2],
    gap: f32,
    mut pick: impl FnMut(&mut Random) -> Vec2,
) -> Vec2 {
    let mut spot = pick(random);
    for _ in 0..WILD_TRIES {
        if road_distance(spot) > ROAD_WIDTH * 2.2
            && !in_city(spot, 10.0)
            && !near_hamlet(spot)
            && !under_water(spot)
            && far_enough(spot, taken, gap)
        {
            break;
        }
        spot = pick(random);
    }
    spot
}
