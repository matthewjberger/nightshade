use crate::data::{NAME_ENDS, NAME_STARTS, VILLAGE_ENDS, VILLAGE_STARTS};
use crate::queries::*;
use crate::resources::{Generation, Places};
use crate::theme;
use Random;
use nalgebra_glm::Vec2;
use nightshade_random::prelude::*;

pub fn random_line(random: &mut Random, lines: &[&str]) -> String {
    lines[random.usize(0..lines.len())].to_string()
}

pub fn random_name(random: &mut Random) -> String {
    let start = NAME_STARTS[random.usize(0..NAME_STARTS.len())];
    let end = NAME_ENDS[random.usize(0..NAME_ENDS.len())];
    format!("{start}{end}")
}

pub fn random_place_spot(
    random: &mut Random,
    places: &Places,
    place: usize,
    fraction: f32,
) -> Vec2 {
    let data = place_spot(places, place);
    let angle = random.f32() * std::f32::consts::TAU;
    let radius = random.f32().sqrt() * data.radius * fraction;
    data.center + Vec2::new(angle.cos(), angle.sin()) * radius
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

pub fn village_name(rng: &mut Random) -> String {
    let start = VILLAGE_STARTS[rng.usize(0..VILLAGE_STARTS.len())];
    let end = VILLAGE_ENDS[rng.usize(0..VILLAGE_ENDS.len())];
    format!("{start}{end}")
}

pub fn wild_spot(
    random: &mut Random,
    generation: &Generation,
    mut pick: impl FnMut(&mut Random) -> Vec2,
) -> Vec2 {
    let mut spot = pick(random);
    for _ in 0..theme::WILD_TRIES {
        let (x, z) = (spot.x.floor() as i32, spot.y.floor() as i32);
        if !under_water(generation, spot) && !reserved(generation, x, z) {
            break;
        }
        spot = pick(random);
    }
    spot
}
