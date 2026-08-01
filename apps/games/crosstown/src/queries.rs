use crate::data;
use crate::data::{Ask, Look, StagePlan, Wave};
use crate::resources::{Blow, Mode, Ring};
use nalgebra_glm::{Vec2, Vec3};
use nightshade_input::Actions;

pub fn walk(actions: &Actions<Ask>) -> Vec2 {
    let wish = actions.axis2(Ask::Walk);
    let mut direction = Vec2::new(
        wish.x + actions.axis(Ask::WalkAcross),
        -wish.y - actions.axis(Ask::WalkAway),
    );
    if direction.magnitude() > 1.0 {
        direction = direction.normalize();
    }
    direction
}

pub fn tapped(actions: &Actions<Ask>) -> f32 {
    if actions.just_pressed(Ask::TapRight) {
        1.0
    } else if actions.just_pressed(Ask::TapLeft) {
        -1.0
    } else {
        0.0
    }
}

pub fn noise(seed: u32) -> f32 {
    let mut value = seed.wrapping_mul(747_796_405).wrapping_add(2_891_336_453);
    value = ((value >> ((value >> 28).wrapping_add(4))) ^ value).wrapping_mul(277_803_737);
    value = (value >> 22) ^ value;
    (value % 100_000) as f32 / 100_000.0
}

pub fn in_reach(blow: &Blow, at: Vec3, radius: f32) -> bool {
    let along = (at.x - blow.at.x) * blow.facing;
    (-0.3 - radius..blow.reach + radius).contains(&along)
        && (at.z - blow.at.z).abs() < data::LANE_REACH + radius * 0.5
}

pub fn plan(ring: &Ring) -> &'static StagePlan {
    &data::STAGES[ring.stage.min(data::STAGE_COUNT - 1)]
}

pub fn wave_count(ring: &Ring) -> usize {
    match ring.mode {
        Mode::Story => plan(ring).waves.len(),
        Mode::Arcade => 1,
    }
}

pub fn wave(ring: &Ring) -> Option<&'static Wave> {
    match ring.mode {
        Mode::Story => plan(ring)
            .waves
            .get(ring.wave.min(plan(ring).waves.len() - 1)),
        Mode::Arcade => None,
    }
}

pub fn look(ring: &Ring) -> Look {
    match ring.mode {
        Mode::Story => plan(ring).look,
        Mode::Arcade => data::ARCADE_LOOKS[ring.round as usize % data::ARCADE_LOOKS.len()],
    }
}

pub fn gate(ring: &Ring) -> f32 {
    match wave(ring) {
        Some(wave) => wave.gate,
        None => data::STREET_END,
    }
}

pub fn far_end(ring: &Ring) -> f32 {
    match ring.mode {
        Mode::Story => plan(ring).end,
        Mode::Arcade => data::STREET_END,
    }
}

pub fn at_once(ring: &Ring) -> u32 {
    match wave(ring) {
        Some(wave) => wave.at_once,
        None => data::ARCADE_AT_ONCE + ring.round / 3,
    }
}

pub fn chief_wave(ring: &Ring) -> bool {
    match wave(ring) {
        Some(wave) => wave.chief,
        None => {
            ring.round >= data::ARCADE_CHIEF_ROUND
                && (ring.round + 1).is_multiple_of(data::ARCADE_CHIEF_ROUND + 1)
        }
    }
}

pub fn wave_line(ring: &Ring) -> String {
    match wave(ring) {
        Some(wave) => wave.line.to_string(),
        None => format!("ROUND {}", ring.round + 1),
    }
}

pub fn kinds(ring: &Ring) -> [u32; data::KIND_COUNT] {
    match wave(ring) {
        Some(wave) => wave.kinds,
        None => {
            let growth = 1.0 + ring.round as f32 * data::ARCADE_STEP;
            let mut counts = [0u32; data::KIND_COUNT];
            for (index, base) in data::ARCADE_WAVE.iter().enumerate() {
                counts[index] = (*base as f32 * growth) as u32;
            }
            counts[data::RUNNER as usize] += ring.round / 2;
            if ring.round >= 2 {
                counts[data::BRUTE as usize] += ring.round / 3;
            }
            if ring.round >= 4 {
                counts[data::ENFORCER as usize] += ring.round / 5;
            }
            if chief_wave(ring) {
                counts[data::BOSS as usize] += 1;
            }
            counts
        }
    }
}

pub fn walk_side(look: Look) -> f32 {
    if look == Look::Rooftop {
        data::ROAD_HALF + 0.8
    } else {
        data::ROAD_HALF + 0.5 + data::WALK_WIDTH * 0.5
    }
}

fn sidewalk_model(look: Look, pick: f32) -> usize {
    match look {
        Look::Street => {
            if pick < 0.28 {
                data::BENCH
            } else if pick < 0.52 {
                data::TREE
            } else if pick < 0.74 {
                data::DUMPSTER
            } else if pick < 0.9 {
                data::TRAFFIC
            } else {
                data::BARRIER
            }
        }
        Look::Alley => {
            if pick < 0.4 {
                data::DUMPSTER
            } else if pick < 0.7 {
                data::PALLET
            } else if pick < 0.88 {
                data::BARRIER
            } else {
                data::BENCH
            }
        }
        Look::Subway => {
            if pick < 0.42 {
                data::BENCH
            } else if pick < 0.72 {
                data::BARRIER
            } else if pick < 0.9 {
                data::PALLET
            } else {
                data::DUMPSTER
            }
        }
        Look::Rooftop => {
            if pick < 0.45 {
                data::PALLET
            } else if pick < 0.8 {
                data::BARRIER
            } else {
                data::DUMPSTER
            }
        }
        Look::Yard => {
            if pick < 0.34 {
                data::PALLET
            } else if pick < 0.62 {
                data::BARRIER
            } else if pick < 0.86 {
                data::DUMPSTER
            } else {
                data::TREE
            }
        }
    }
}

fn lane_model(look: Look, pick: f32) -> usize {
    match look {
        Look::Street => data::BARRIER,
        Look::Alley => {
            if pick < 0.55 {
                data::BARRIER
            } else {
                data::DUMPSTER
            }
        }
        Look::Subway => data::BARRIER,
        Look::Rooftop => data::BARRIER,
        Look::Yard => {
            if pick < 0.6 {
                data::BARRIER
            } else {
                data::DUMPSTER
            }
        }
    }
}

pub fn prop_spots(look: Look, end: f32) -> Vec<(usize, Vec3, f32, f32)> {
    let mut spots = Vec::new();
    let near = walk_side(look);
    let ground = if look == Look::Rooftop {
        0.0
    } else {
        data::CURB_HEIGHT
    };
    let mut at = data::STREET_START + 4.0;
    let mut index = 0u32;
    while at < end - 2.0 {
        let side = if noise(index * 19 + 1) > 0.5 {
            -near
        } else {
            near
        };
        let yaw = if side < 0.0 {
            0.0
        } else {
            std::f32::consts::PI
        };
        let model = sidewalk_model(look, noise(index * 53 + 7));
        spots.push((model, Vec3::new(at, ground, side), yaw, 0.0));
        at += 4.5 + noise(index * 29) * 4.5;
        index += 1;
    }

    let mut lane = data::STREET_START + 12.0;
    let mut step = 0u32;
    while lane < end - 6.0 {
        let model = lane_model(look, noise(step * 61 + 3));
        let depth = (noise(step * 43 + 11) - 0.5) * 2.0 * (data::LANE_HALF - 0.9);
        let yaw = noise(step * 71) * std::f32::consts::PI;
        spots.push((
            model,
            Vec3::new(lane, 0.0, depth),
            yaw,
            data::LANE_PROP_RADIUS,
        ));
        lane += 11.0 + noise(step * 83) * 6.0;
        step += 1;
    }
    spots
}

pub fn break_spots(look: Look, end: f32, seed: u32) -> Vec<(u32, Vec3)> {
    let mut spots = Vec::new();
    let mut at = data::STREET_START + 14.0;
    let mut index = 0u32;
    while at < end - 3.0 {
        let cluster = 1 + (noise(index * 97 + seed) * 2.6) as u32;
        let depth = (noise(index * 71 + seed * 13) - 0.5) * 2.0 * (data::LANE_HALF - 0.6);
        for step in 0..cluster {
            let pick = noise(index * 31 + step * 7 + seed);
            let kind = match look {
                Look::Street | Look::Alley => {
                    if pick < 0.72 {
                        data::CRATE
                    } else {
                        data::BARREL
                    }
                }
                Look::Subway => {
                    if pick < 0.5 {
                        data::CRATE
                    } else {
                        data::BARREL
                    }
                }
                Look::Rooftop | Look::Yard => {
                    if pick < 0.35 {
                        data::CRATE
                    } else {
                        data::BARREL
                    }
                }
            };
            let along = at + step as f32 * data::BREAK_CLUSTER;
            let across = depth + (noise(index * 17 + step * 23) - 0.5) * data::BREAK_CLUSTER;
            spots.push((
                kind,
                Vec3::new(along, 0.0, across.clamp(-data::LANE_HALF, data::LANE_HALF)),
            ));
        }
        at += data::BREAK_SPACING + noise(index * 37 + seed) * 4.0;
        index += 1;
    }
    spots
}

pub fn pickup_spots(end: f32, seed: u32) -> Vec<(u32, Vec3)> {
    let mut spots = Vec::new();
    let mut at = data::STREET_START + 13.0;
    let mut index = 0u32;
    while at < end - 5.0 {
        let kind = if noise(index * 59 + seed) < 0.4 {
            data::ROAST
        } else {
            data::DRINK
        };
        let depth = (noise(index * 89 + seed * 3) - 0.5) * 2.0 * (data::LANE_HALF - 0.5);
        spots.push((kind, Vec3::new(at, 0.0, depth)));
        at += data::PICKUP_SPACING + noise(index * 41 + seed) * 6.0;
        index += 1;
    }
    spots
}

pub fn eye_at(x: f32, shake: f32) -> Vec3 {
    Vec3::new(
        x + shake,
        data::CAMERA_HEIGHT + shake * 0.4,
        data::CAMERA_BACK,
    )
}

pub fn look_at(x: f32) -> Vec3 {
    Vec3::new(x, data::CAMERA_LOOK, 0.0)
}

pub fn hero_limit(camera_x: f32, cleared: bool, gate: f32, end: f32, last: bool) -> (f32, f32) {
    let wall = if !cleared {
        gate - 1.1
    } else if last {
        end - 2.0
    } else {
        gate + 3.5
    };
    (
        (camera_x - data::CAMERA_MARGIN).max(data::STREET_START + 1.5),
        (camera_x + data::CAMERA_MARGIN).min(wall),
    )
}

pub fn camera_limit(cleared: bool, gate: f32, end: f32, last: bool) -> (f32, f32) {
    let far = if !cleared {
        gate - 8.0
    } else if last {
        end - 9.0
    } else {
        gate - 5.0
    };
    (data::STREET_START + 9.0, far.max(data::STREET_START + 9.0))
}

pub fn walk_pose(stride: f32) -> u32 {
    let count = data::POSE_WALK.len();
    data::POSE_WALK[(stride.rem_euclid(count as f32)) as usize % count]
}

pub fn run_pose(stride: f32) -> u32 {
    let count = data::POSE_RUN.len();
    data::POSE_RUN[(stride.rem_euclid(count as f32)) as usize % count]
}
