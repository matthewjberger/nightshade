use crate::resources::{Spot, Wanted};
use nalgebra_glm::{Vec3, vec3};
use nightshade_kit::prelude::*;
use nightshade_random::Random;
use nightshade_random::commands::{between, index, pick, whole};

const QUARTER: f32 = std::f32::consts::FRAC_PI_2;
const HALF: f32 = std::f32::consts::PI;
const TURN: f32 = std::f32::consts::TAU;

pub fn pieces(wanted: impl Fn(&str) -> bool) -> Vec<&'static str> {
    MODELS
        .iter()
        .filter(|file| file.kit == Kit::Town && wanted(file.name))
        .map(|file| file.name)
        .collect()
}

pub fn lay_out(wanted: &Wanted) -> Vec<Spot> {
    let mut roll = Random::seeded(wanted.seed as u64);
    let mut spots = Vec::new();
    pave(&mut spots, wanted);
    hold_the_market(&mut spots, &mut roll, wanted);
    for (side, along) in plots_of(wanted) {
        raise_a_house(&mut spots, &mut roll, wanted, side, along);
    }
    plant_the_country(&mut spots, &mut roll, wanted);
    stand_the_mills(&mut spots, wanted);
    spots
}

pub fn named(spots: &[Spot]) -> Vec<&'static str> {
    let mut names: Vec<&'static str> = Vec::new();
    for spot in spots {
        if !names.contains(&spot.model) {
            names.push(spot.model);
        }
    }
    names
}

fn place(spots: &mut Vec<Spot>, model: &'static str, position: Vec3, yaw: f32) {
    spots.push(Spot {
        model,
        position,
        yaw,
        squeeze: 1.0,
    });
}

fn place_short(spots: &mut Vec<Spot>, model: &'static str, position: Vec3, yaw: f32) {
    spots.push(Spot {
        model,
        position,
        yaw,
        squeeze: 0.8,
    });
}

fn at(across: i64, along: i64, lift: f32) -> Vec3 {
    vec3(across as f32, lift, along as f32)
}

fn pave(spots: &mut Vec<Spot>, wanted: &Wanted) {
    for along in -wanted.street..=wanted.street {
        for across in -wanted.square..=wanted.square {
            let street = across.abs() <= 1;
            let square = along.abs() < wanted.square;
            if street || square {
                place(spots, "road", at(across, along, 0.0), 0.0);
            }
        }
    }
}

fn hold_the_market(spots: &mut Vec<Spot>, roll: &mut Random, wanted: &Wanted) {
    place(spots, "fountain-round", at(0, 0, 0.0), 0.0);
    let stalls = pieces(|name| name.starts_with("stall-"));
    for side in [-1, 1] {
        for along in [-1, 1] {
            let stall = stalls[index(roll, stalls.len())];
            let yaw = if side > 0 { HALF } else { 0.0 };
            place(spots, stall, at(side * wanted.square, along, 0.0), yaw);
        }
    }
    place(
        spots,
        "cart",
        at(2, wanted.square - 1, 0.0),
        between(roll, 0.0, HALF),
    );
}

fn plots_of(wanted: &Wanted) -> Vec<(i64, i64)> {
    let mut plots = Vec::new();
    let mut along = wanted.square + 1;
    while plots.len() < wanted.houses as usize && along <= wanted.street {
        for side in [-1, 1] {
            for step in [along, -along] {
                if plots.len() < wanted.houses as usize {
                    plots.push((side, step));
                }
            }
        }
        along += wanted.spacing;
    }
    plots
}

fn raise_a_house(spots: &mut Vec<Spot>, roll: &mut Random, wanted: &Wanted, side: i64, along: i64) {
    let across = side * (wanted.square + 1);
    let door_yaw = if side > 0 { HALF } else { 0.0 };
    let doors = pieces(|name| name.starts_with("wall-") && name.ends_with("-door"));
    let door = doors[index(roll, doors.len())];
    let wooden = door.contains("wood");
    place(spots, door, at(across, along, 0.0), door_yaw);
    let windows = pieces(|name| name.contains("-window-") && name.contains("wood") == wooden);
    for turn in [QUARTER, -QUARTER] {
        let window = pick(roll, &windows).copied().unwrap_or(door);
        place_short(spots, window, at(across, along, 0.0), door_yaw + turn);
    }
    let back = if wooden { "wall-wood" } else { "wall" };
    place(spots, back, at(across, along, 0.0), door_yaw + HALF);
    place(spots, "roof-point", at(across, along, 1.0), 0.0);
    place(
        spots,
        "chimney",
        at(across, along, 1.0) + vec3(0.0, 0.0, 0.2),
        QUARTER,
    );
    place(
        spots,
        "lantern",
        vec3(side as f32 * 1.5, 0.0, along as f32),
        0.0,
    );
    fence_the_garden(spots, wanted, side, along);
}

fn fence_the_garden(spots: &mut Vec<Spot>, wanted: &Wanted, side: i64, along: i64) {
    let across = side * wanted.square;
    let yaw = if side > 0 { HALF } else { 0.0 };
    for step in -1..=1 {
        let model = if step == 0 { "fence-gate" } else { "fence" };
        place(spots, model, at(across, along + step, 0.0), yaw);
    }
}

fn plant_the_country(spots: &mut Vec<Spot>, roll: &mut Random, wanted: &Wanted) {
    let near = wanted.square as f32 + 3.0;
    let trees = pieces(|name| name.starts_with("tree"));
    let rocks = pieces(|name| name.starts_with("rock"));
    for _ in 0..wanted.trees {
        let spot = out_of_town(roll, wanted, near);
        place(
            spots,
            trees[index(roll, trees.len())],
            spot,
            between(roll, 0.0, TURN),
        );
    }
    for _ in 0..wanted.rocks {
        let spot = out_of_town(roll, wanted, near - 1.0);
        place(
            spots,
            rocks[index(roll, rocks.len())],
            spot,
            between(roll, 0.0, TURN),
        );
    }
}

fn out_of_town(roll: &mut Random, wanted: &Wanted, near: f32) -> Vec3 {
    let side = if whole(roll, 0, 1) == 0 { -1.0 } else { 1.0 };
    vec3(
        between(roll, near, wanted.reach) * side,
        0.0,
        between(roll, -wanted.reach, wanted.reach),
    )
}

fn stand_the_mills(spots: &mut Vec<Spot>, wanted: &Wanted) {
    let out = wanted.street as f32 + 1.0;
    place(spots, "windmill", vec3(-9.0, 0.0, -out), QUARTER);
    place(spots, "watermill", vec3(9.0, 0.0, out), -QUARTER);
}
