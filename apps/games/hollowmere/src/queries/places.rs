use crate::data::{BARROW, CRYPT, DEEP, Ground, LAKE, MINE, PLACES, TOWN};
use crate::queries::{direction, land_height_with, raw_height, villages_near, zone_center};
use crate::resources::{Generation, Village, ZonePlan};
use crate::theme;
use nalgebra_glm::Vec2;

fn height_at(generation: &Generation, spot: Vec2) -> i32 {
    raw_height(generation, spot.x.floor() as i32, spot.y.floor() as i32)
}

fn dry_around(generation: &Generation, center: Vec2, radius: f32) -> bool {
    let samples = [
        center,
        center + Vec2::new(radius * 0.6, 0.0),
        center - Vec2::new(radius * 0.6, 0.0),
        center + Vec2::new(0.0, radius * 0.6),
        center - Vec2::new(0.0, radius * 0.6),
    ];
    samples
        .iter()
        .all(|spot| height_at(generation, *spot) > theme::SEA_LEVEL + 1)
}

fn clear_of(zones: &[ZonePlan], center: Vec2, radius: f32) -> bool {
    zones.iter().all(|placed| {
        (zone_center(placed) - center).magnitude() > placed.radius + radius + theme::PLACE_GAP
    })
}

fn candidates(anchor: Vec2, reach: f32, angle: f32) -> Vec<Vec2> {
    let mut found = Vec::new();
    for stretch in [1.0, 1.18, 1.36] {
        for step in 0..theme::PLACE_TRIES {
            let sign = if step % 2 == 0 { 1.0 } else { -1.0 };
            let turn = (step as f32 / 2.0).ceil() * theme::PLACE_TURN * sign;
            found.push(anchor + direction(angle + turn) * reach * stretch);
        }
    }
    found
}

fn shore_candidates(lake: &ZonePlan) -> Vec<Vec2> {
    (0..24)
        .map(|step| {
            let angle = step as f32 / 24.0 * std::f32::consts::TAU;
            zone_center(lake) + direction(angle) * (lake.radius + 4.0)
        })
        .collect()
}

fn facing_toward(from: Vec2, to: Vec2) -> f32 {
    let toward = to - from;
    toward.y.atan2(toward.x)
}

fn plan_zone(center: Vec2, radius: f32, level: i32, facing: f32) -> ZonePlan {
    ZonePlan {
        center: (center.x.floor() as i32, center.y.floor() as i32),
        radius,
        level,
        facing,
    }
}

pub fn lay_out_zones(generation: &Generation, town: (i32, i32), wall_radius: f32) -> Vec<ZonePlan> {
    let mut zones: Vec<ZonePlan> = Vec::with_capacity(PLACES.len());
    let town_center = Vec2::new(town.0 as f32 + 0.5, town.1 as f32 + 0.5);
    for (index, plan) in PLACES.iter().enumerate() {
        if index == TOWN {
            zones.push(plan_zone(
                town_center,
                wall_radius,
                height_at(generation, town_center).max(theme::TOWN_LEVEL_MIN),
                0.0,
            ));
            continue;
        }
        if index == DEEP {
            let mine = zones[MINE];
            let mouth = zone_center(&mine) + direction(mine.facing) * (mine.radius * 0.55);
            let mouth_y = land_height_with(
                generation,
                &zones,
                mouth.x.floor() as i32,
                mouth.y.floor() as i32,
            );
            let center = zone_center(&mine) + direction(mine.facing + std::f32::consts::PI) * 8.0;
            zones.push(plan_zone(
                center,
                plan.radius,
                mouth_y - theme::DEEP_DEPTH,
                mine.facing,
            ));
            continue;
        }
        let anchor = zones[plan.anchor];
        let options = match plan.ground {
            Ground::Shore => shore_candidates(&anchor),
            _ => candidates(zone_center(&anchor), plan.reach, plan.angle),
        };
        let fallback = zone_center(&anchor) + direction(plan.angle) * plan.reach;
        let chosen = match plan.ground {
            Ground::Land => options
                .iter()
                .copied()
                .find(|center| {
                    dry_around(generation, *center, plan.radius)
                        && clear_of(&zones, *center, plan.radius)
                })
                .unwrap_or(fallback),
            Ground::Water => options
                .iter()
                .copied()
                .find(|center| {
                    height_at(generation, *center) < theme::SEA_LEVEL - 1
                        && clear_of(&zones, *center, plan.radius)
                })
                .or_else(|| {
                    options
                        .iter()
                        .copied()
                        .find(|center| clear_of(&zones, *center, plan.radius))
                })
                .unwrap_or(fallback),
            Ground::High => options
                .iter()
                .copied()
                .filter(|center| {
                    dry_around(generation, *center, plan.radius)
                        && clear_of(&zones, *center, plan.radius)
                })
                .max_by_key(|center| height_at(generation, *center))
                .unwrap_or(fallback),
            Ground::Shore => options
                .iter()
                .copied()
                .filter(|center| {
                    height_at(generation, *center) > theme::SEA_LEVEL + 1
                        && clear_of(&zones[1..], *center, plan.radius)
                })
                .min_by_key(|center| ((center - town_center).magnitude() * 10.0) as i64)
                .unwrap_or(fallback),
        };
        let facing = match index {
            HARBOR_INDEX => facing_toward(chosen, zone_center(&zones[LAKE])),
            _ => facing_toward(chosen, town_center),
        };
        let level = match index {
            LAKE => theme::SEA_LEVEL,
            CRYPT => zones[BARROW].level - theme::CRYPT_DEPTH,
            BARROW | MINE => height_at(generation, chosen),
            _ => land_height_with(
                generation,
                &zones,
                chosen.x.floor() as i32,
                chosen.y.floor() as i32,
            )
            .max(theme::SEA_LEVEL + 2),
        };
        zones.push(plan_zone(chosen, plan.radius, level, facing));
    }
    zones
}

const HARBOR_INDEX: usize = crate::data::HARBOR;

pub fn town_of(generation: &Generation, spawn: (i32, i32)) -> (i32, i32) {
    villages_near(generation, spawn.0, spawn.1)
        .min_by_key(|village| {
            (village.square.0 - spawn.0).abs() + (village.square.1 - spawn.1).abs()
        })
        .map(|village| village.square)
        .unwrap_or(spawn)
}

pub fn wall_radius_of(village: &Village, town: (i32, i32)) -> f32 {
    let mut farthest = 0.0f32;
    for footprint in &village.buildings {
        for (x, z) in [
            (footprint.x, footprint.z),
            (footprint.x + footprint.width, footprint.z),
            (footprint.x, footprint.z + footprint.depth),
            (footprint.x + footprint.width, footprint.z + footprint.depth),
        ] {
            let reach = Vec2::new((x - town.0) as f32, (z - town.1) as f32).magnitude();
            farthest = farthest.max(reach);
        }
    }
    (farthest + 7.0).clamp(theme::TOWN_WALL_MIN as f32, theme::TOWN_WALL_MAX as f32)
}

pub fn village_clear_of_zones(zones: &[ZonePlan], center: (i32, i32)) -> bool {
    let spot = Vec2::new(center.0 as f32 + 0.5, center.1 as f32 + 0.5);
    zones.iter().enumerate().all(|(index, zone)| {
        index == TOWN
            || (zone_center(zone) - spot).magnitude() > zone.radius + theme::ZONE_VILLAGE_GAP
    })
}
