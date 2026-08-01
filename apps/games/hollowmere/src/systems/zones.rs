use crate::commands::build_room;
use crate::data::{
    BARROW, BlockKind, CRYPT, DEEP, FARM, FIELD, FORT, HARBOR, MINE, STARTER, TOWN, WOOD,
};
use crate::queries::{
    along_segment, axis_of, block_at, direction, ground_top, harbor_pier, hash, in_bounds,
    in_the_rooms, in_the_tunnel, near_building, raw_height, road_at, zone_center, zone_signs,
};
use crate::resources::{ColumnKey, Crypt, Generation, Mine, VoxelWorld, ZonePlan};
use crate::theme;
use nalgebra_glm::{IVec3, Vec2};
use nightshade_voxels::prelude::write_block;

const TAU: f32 = std::f32::consts::TAU;

struct Column {
    x: i32,
    z: i32,
    top: i32,
    point: Vec2,
}

fn set(world: &mut VoxelWorld, x: i32, y: i32, z: i32, kind: BlockKind) {
    let position = IVec3::new(x, y, z);
    if in_bounds(position) {
        write_block(world, position, kind.index());
    }
}

fn fill(world: &mut VoxelWorld, x: i32, z: i32, from: i32, to: i32, kind: BlockKind) {
    for y in from..=to {
        set(world, x, y, z, kind);
    }
}

fn is_stone(world: &VoxelWorld, x: i32, y: i32, z: i32) -> bool {
    block_at(world, IVec3::new(x, y, z)) == BlockKind::Stone
}

fn lerp(low: i32, high: i32, share: f32) -> i32 {
    (low as f32 + (high - low) as f32 * share).round() as i32
}

pub fn zones(world: &mut VoxelWorld, generation: &mut Generation, column: ColumnKey) {
    if generation.zones.is_empty() {
        return;
    }
    let origin = IVec3::new(column[0] * theme::CHUNK, 0, column[1] * theme::CHUNK);
    let crypt = generation.crypt;
    let mine = generation.mine;
    let pier = harbor_pier(generation);
    let signs = zone_signs(generation);
    let zones = generation.zones.clone();
    for local_z in 0..theme::CHUNK {
        for local_x in 0..theme::CHUNK {
            let (x, z) = (origin.x + local_x, origin.z + local_z);
            let Some(top) = ground_top(generation, x, z) else {
                continue;
            };
            let here = Column {
                x,
                z,
                top,
                point: Vec2::new(x as f32 + 0.5, z as f32 + 0.5),
            };
            road(world, generation, &here);
            for (index, zone) in zones.iter().enumerate() {
                let offset = here.point - zone_center(zone);
                if offset.x.abs() > zone.radius + 44.0 || offset.y.abs() > zone.radius + 44.0 {
                    continue;
                }
                match index {
                    TOWN => town_wall(world, generation, zone, &here),
                    FORT => fort(world, zone, &here),
                    FARM => farm(world, zone, &here),
                    BARROW => barrow(world, generation, zone, &here),
                    CRYPT => {
                        if let Some(crypt) = &crypt {
                            crypt_rooms(world, crypt, &here);
                        }
                    }
                    MINE => {
                        if let Some(mine) = &mine {
                            mine_tunnel(world, generation, mine, &here);
                        }
                    }
                    DEEP => deep_chamber(world, generation, zone, &here),
                    HARBOR => {
                        if let Some((start, end)) = pier {
                            harbor(world, zone, start, end, &here);
                        }
                    }
                    STARTER => hamlet(world, zone, &here),
                    WOOD => clearing(world, zone, &here),
                    FIELD => field(world, zone, &here),
                    _ => {}
                }
            }
            for (spot, _) in &signs {
                if *spot == (x, z) && block_at(world, IVec3::new(x, top, z)).info().solid {
                    set(world, x, top + 1, z, BlockKind::Signpost);
                }
            }
        }
    }
}

fn road(world: &mut VoxelWorld, generation: &Generation, here: &Column) {
    let Some((along, perpendicular)) = road_at(generation, here.point) else {
        return;
    };
    if perpendicular > theme::ROAD_HALF + 1.2 {
        return;
    }
    let deck = theme::SEA_LEVEL + 1;
    if here.top <= theme::SEA_LEVEL {
        if perpendicular <= theme::ROAD_HALF + 0.3 {
            fill(world, here.x, here.z, deck + 1, deck + 4, BlockKind::Air);
            set(world, here.x, deck, here.z, BlockKind::OakPlanks);
        } else if along.rem_euclid(3.0) < 1.0 {
            fill(
                world,
                here.x,
                here.z,
                here.top + 1,
                deck + 1,
                BlockKind::OakLog,
            );
            if along.rem_euclid(9.0) < 1.0 {
                set(world, here.x, deck + 2, here.z, BlockKind::Torch);
            }
        }
        return;
    }
    if perpendicular <= theme::ROAD_HALF {
        let top = block_at(world, IVec3::new(here.x, here.top, here.z));
        if matches!(
            top,
            BlockKind::Grass
                | BlockKind::Dirt
                | BlockKind::Sand
                | BlockKind::Gravel
                | BlockKind::SnowGrass
        ) {
            set(world, here.x, here.top, here.z, BlockKind::Gravel);
        }
        return;
    }
    if perpendicular > theme::ROAD_HALF + 0.5
        && along.rem_euclid(theme::LAMP_GAP) < 1.0
        && !near_building(generation, here.x, here.z, 2)
        && block_at(world, IVec3::new(here.x, here.top, here.z))
            .info()
            .solid
    {
        fill(
            world,
            here.x,
            here.z,
            here.top + 1,
            here.top + 2,
            BlockKind::OakLog,
        );
        set(world, here.x, here.top + 3, here.z, BlockKind::Torch);
    }
}

fn town_wall(world: &mut VoxelWorld, generation: &Generation, zone: &ZonePlan, here: &Column) {
    let center = zone_center(zone);
    let offset = here.point - center;
    let distance = offset.magnitude();
    let radius = zone.radius;
    for tower in 0..4 {
        let angle = TAU / 8.0 + tower as f32 * TAU / 4.0;
        let spot = center + direction(angle) * radius;
        let (tower_x, tower_z) = (spot.x.floor() as i32, spot.y.floor() as i32);
        if (here.x - tower_x).abs() > 2 || (here.z - tower_z).abs() > 2 {
            continue;
        }
        let rim = (here.x - tower_x).abs() == 2 || (here.z - tower_z).abs() == 2;
        let crown = here.top + theme::TOWN_TOWER_HEIGHT;
        fill(
            world,
            here.x,
            here.z,
            here.top + 1,
            crown,
            BlockKind::Cobblestone,
        );
        if rim && (here.x + here.z).rem_euclid(2) == 0 {
            set(world, here.x, crown + 1, here.z, BlockKind::Cobblestone);
        }
        if here.x == tower_x && here.z == tower_z {
            fill(
                world,
                here.x,
                here.z,
                crown + 1,
                crown + 3,
                BlockKind::OakLog,
            );
            set(world, here.x, crown + 4, here.z, BlockKind::WoolRed);
        }
        return;
    }
    if (distance - radius).abs() > 0.6 {
        return;
    }
    let gate = offset.x.abs() <= theme::TOWN_GATE_HALF as f32 + 0.5
        || offset.y.abs() <= theme::TOWN_GATE_HALF as f32 + 0.5;
    let crown = here.top + theme::TOWN_WALL_HEIGHT;
    if gate {
        fill(
            world,
            here.x,
            here.z,
            here.top + 1,
            here.top + 3,
            BlockKind::Air,
        );
        fill(
            world,
            here.x,
            here.z,
            here.top + 4,
            crown,
            BlockKind::Cobblestone,
        );
        return;
    }
    fill(
        world,
        here.x,
        here.z,
        here.top - 2,
        here.top,
        BlockKind::Stone,
    );
    fill(
        world,
        here.x,
        here.z,
        here.top + 1,
        crown,
        BlockKind::Cobblestone,
    );
    if (here.x + here.z).rem_euclid(2) == 0 {
        set(world, here.x, crown + 1, here.z, BlockKind::Cobblestone);
    } else if hash(generation.seed ^ 0x0701_1A11, here.x, here.z).is_multiple_of(9) {
        set(world, here.x, crown + 1, here.z, BlockKind::Torch);
    }
}

fn fort(world: &mut VoxelWorld, zone: &ZonePlan, here: &Column) {
    let (u, v) = (here.x - zone.center.0, here.z - zone.center.1);
    let half = theme::FORT_HALF;
    let reach = u.abs().max(v.abs());
    if reach > half {
        return;
    }
    let level = zone.level;
    let (gate_x, gate_z) = axis_of(zone.facing);
    let toward_gate = u * gate_x + v * gate_z;
    let across_gate = if gate_x != 0 { v } else { u };
    fill(world, here.x, here.z, level - 2, level - 1, BlockKind::Dirt);
    set(world, here.x, level, here.z, BlockKind::Grass);
    fill(world, here.x, here.z, level + 1, level + 14, BlockKind::Air);
    if u.abs() >= half - 2 && v.abs() >= half - 2 {
        let crown = level + theme::FORT_TOWER_HEIGHT;
        fill(
            world,
            here.x,
            here.z,
            level + 1,
            crown,
            BlockKind::Cobblestone,
        );
        if reach == half && (u + v).rem_euclid(2) == 0 {
            set(world, here.x, crown + 1, here.z, BlockKind::Cobblestone);
        }
        if u.abs() == half - 1 && v.abs() == half - 1 {
            set(world, here.x, crown + 1, here.z, BlockKind::Torch);
        }
        return;
    }
    if reach == half {
        let crown = level + theme::FORT_WALL_HEIGHT;
        if toward_gate == half && across_gate.abs() <= 1 {
            fill(
                world,
                here.x,
                here.z,
                level + 4,
                crown,
                BlockKind::Cobblestone,
            );
            return;
        }
        fill(
            world,
            here.x,
            here.z,
            level + 1,
            crown,
            BlockKind::Cobblestone,
        );
        if (u + v).rem_euclid(2) == 0 {
            set(world, here.x, crown + 1, here.z, BlockKind::Cobblestone);
        } else if (u.abs() + v.abs()).rem_euclid(7) == 0 {
            set(world, here.x, crown + 1, here.z, BlockKind::Torch);
        }
        return;
    }
    if u.abs() <= 1 || v.abs() <= 1 {
        set(world, here.x, level, here.z, BlockKind::Cobblestone);
    }
    let keep_half = theme::FORT_KEEP_HALF;
    let keep_reach = u.abs().max(v.abs());
    if keep_reach <= keep_half {
        let crown = level + theme::FORT_KEEP_HEIGHT;
        set(world, here.x, level, here.z, BlockKind::OakPlanks);
        if keep_reach == keep_half {
            let door = toward_gate == keep_half && across_gate == 0;
            if door {
                fill(
                    world,
                    here.x,
                    here.z,
                    level + 3,
                    crown,
                    BlockKind::Cobblestone,
                );
            } else {
                fill(
                    world,
                    here.x,
                    here.z,
                    level + 1,
                    crown,
                    BlockKind::Cobblestone,
                );
                if (u + v).rem_euclid(3) == 0 {
                    set(world, here.x, level + 2, here.z, BlockKind::Glass);
                    set(world, here.x, level + 6, here.z, BlockKind::Glass);
                }
            }
            set(world, here.x, crown + 1, here.z, BlockKind::Cobblestone);
            if (u + v).rem_euclid(2) == 0 {
                set(world, here.x, crown + 2, here.z, BlockKind::Cobblestone);
            }
            return;
        }
        set(world, here.x, crown, here.z, BlockKind::OakPlanks);
        if u == 0 && v == 0 {
            set(world, here.x, crown - 1, here.z, BlockKind::Glowstone);
        }
        if u == 0 && v == 1 {
            set(world, here.x, level + 1, here.z, BlockKind::Chest);
        }
        return;
    }
    if (u - 5).abs() <= 1 && (v - 5).abs() <= 1 {
        set(world, here.x, level, here.z, BlockKind::Cobblestone);
        if u == 5 && v == 5 {
            set(world, here.x, level + 1, here.z, BlockKind::Campfire);
        }
    }
    if (u + 6).abs() <= 1 && v == 6 {
        set(world, here.x, level + 1, here.z, BlockKind::OakLog);
        if u == -6 {
            set(world, here.x, level + 2, here.z, BlockKind::OakLog);
        }
    }
    if (u.abs() == 8 && v == 0) || (v.abs() == 8 && u == 0) {
        set(world, here.x, level + 1, here.z, BlockKind::Torch);
    }
}

fn hut(
    world: &mut VoxelWorld,
    here: &Column,
    u: i32,
    v: i32,
    level: i32,
    half_u: i32,
    half_v: i32,
    door: (i32, i32),
    height: i32,
) -> bool {
    if u.abs() > half_u + 1 || v.abs() > half_v + 1 {
        return false;
    }
    let roof = level + height + 1;
    if u.abs() == half_u + 1 || v.abs() == half_v + 1 {
        fill(world, here.x, here.z, level + 1, roof + 3, BlockKind::Air);
        set(world, here.x, roof, here.z, BlockKind::OakPlanks);
        return true;
    }
    set(world, here.x, level, here.z, BlockKind::OakPlanks);
    fill(world, here.x, here.z, level + 1, roof + 3, BlockKind::Air);
    set(world, here.x, roof, here.z, BlockKind::OakPlanks);
    let rim = u.abs() == half_u || v.abs() == half_v;
    if rim {
        let corner = u.abs() == half_u && v.abs() == half_v;
        let toward = u * door.0 + v * door.1;
        let facing_half = if door.0 != 0 { half_u } else { half_v };
        let across = if door.0 != 0 { v } else { u };
        if toward == facing_half && across == 0 {
            fill(
                world,
                here.x,
                here.z,
                level + 3,
                level + height,
                BlockKind::OakPlanks,
            );
            return true;
        }
        for y in level + 1..=level + height {
            let kind = if corner {
                BlockKind::OakLog
            } else if y == level + 2 && (u + v).rem_euclid(3) == 0 {
                BlockKind::Glass
            } else {
                BlockKind::OakPlanks
            };
            set(world, here.x, y, here.z, kind);
        }
        return true;
    }
    if u == 0 && v == 0 {
        set(world, here.x, level + height, here.z, BlockKind::Glowstone);
    }
    true
}

fn farm(world: &mut VoxelWorld, zone: &ZonePlan, here: &Column) {
    let (u, v) = (here.x - zone.center.0, here.z - zone.center.1);
    let level = zone.level;
    let in_field = (-12..=-3).contains(&u) || (3..=12).contains(&u);
    if in_field && (-7..=7).contains(&v) {
        fill(world, here.x, here.z, level + 1, level + 4, BlockKind::Air);
        if v.rem_euclid(3) == 1 {
            set(world, here.x, level - 1, here.z, BlockKind::Dirt);
            set(world, here.x, level, here.z, BlockKind::Water);
        } else {
            set(world, here.x, level, here.z, BlockKind::Dirt);
        }
        return;
    }
    let post_u = u == -13 || u == -2 || u == 2 || u == 13;
    let post_v = v == -8 || v == 8;
    let along_u = (-13..=-2).contains(&u) || (2..=13).contains(&u);
    if (post_u && (-8..=8).contains(&v)) || (post_v && along_u) {
        set(world, here.x, level, here.z, BlockKind::Grass);
        let kind = if (u + v).rem_euclid(3) == 0 {
            BlockKind::OakLog
        } else {
            BlockKind::OakPlanks
        };
        set(world, here.x, level + 1, here.z, kind);
        return;
    }
    if hut(world, here, u, v + 12, level, 4, 3, (0, 1), 4) {
        return;
    }
    if u.abs() <= 1 && (v - 10).abs() <= 1 {
        if u == 0 && v == 10 {
            fill(world, here.x, here.z, level - 2, level, BlockKind::Water);
            fill(world, here.x, here.z, level + 1, level + 3, BlockKind::Air);
        } else {
            set(world, here.x, level + 1, here.z, BlockKind::Cobblestone);
            if u.abs() == 1 && (v - 10).abs() == 1 {
                fill(
                    world,
                    here.x,
                    here.z,
                    level + 2,
                    level + 3,
                    BlockKind::OakLog,
                );
            }
        }
        set(world, here.x, level + 4, here.z, BlockKind::OakPlanks);
    }
}

fn barrow(world: &mut VoxelWorld, generation: &Generation, zone: &ZonePlan, here: &Column) {
    let center = zone_center(zone);
    for ring in 0..theme::STONE_RING_COUNT {
        let angle = ring as f32 / theme::STONE_RING_COUNT as f32 * TAU;
        let spot = center + direction(angle) * theme::STONE_RING;
        if (spot.x.floor() as i32, spot.y.floor() as i32) == (here.x, here.z) {
            let tall = if ring % 2 == 0 { 3 } else { 2 };
            fill(
                world,
                here.x,
                here.z,
                here.top + 1,
                here.top + tall,
                BlockKind::Stone,
            );
            return;
        }
    }
    for (mound_x, mound_z) in theme::MOUNDS {
        let mound = (
            zone.center.0 + mound_x as i32,
            zone.center.1 + mound_z as i32,
        );
        let (u, v) = (here.x - mound.0, here.z - mound.1);
        if u.abs() > 2 || !(-2..=5).contains(&v) {
            continue;
        }
        let base = raw_height(generation, mound.0, mound.1);
        if u.abs() <= 1 && v.abs() <= 1 {
            fill(world, here.x, here.z, base + 1, base + 2, BlockKind::Air);
            set(world, here.x, base, here.z, BlockKind::Cobblestone);
            set(world, here.x, base + 3, here.z, BlockKind::Cobblestone);
            if u == 0 && v == 0 {
                set(world, here.x, base + 1, here.z, BlockKind::Chest);
            }
            if u == 0 && v == 1 {
                set(world, here.x, base + 3, here.z, BlockKind::Glowstone);
            }
            return;
        }
        if u == 0 && (2..=5).contains(&v) {
            fill(world, here.x, here.z, base + 1, base + 2, BlockKind::Air);
            set(world, here.x, base, here.z, BlockKind::Cobblestone);
            set(world, here.x, base + 3, here.z, BlockKind::Cobblestone);
            return;
        }
        if u.abs() == 1 && (2..=5).contains(&v) {
            fill(
                world,
                here.x,
                here.z,
                base + 1,
                base + 3,
                BlockKind::Cobblestone,
            );
            return;
        }
        if u.abs() == 2 && (-2..=2).contains(&v) {
            fill(
                world,
                here.x,
                here.z,
                base + 1,
                base + 3,
                BlockKind::Cobblestone,
            );
            return;
        }
        if v == -2 && u.abs() <= 2 {
            fill(
                world,
                here.x,
                here.z,
                base + 1,
                base + 3,
                BlockKind::Cobblestone,
            );
            return;
        }
    }
}

fn crypt_rooms(world: &mut VoxelWorld, crypt: &Crypt, here: &Column) {
    let inside = (here.x - crypt.corner.x, here.z - crypt.corner.z);
    let reach = -theme::DUNGEON_TUNNEL..(crypt.span + theme::DUNGEON_TUNNEL);
    let chest = Some(IVec3::new(
        crypt.corner.x + crypt.span / 2,
        crypt.corner.y + 1,
        crypt.corner.z + crypt.span / 2,
    ));
    if reach.contains(&inside.0) && reach.contains(&inside.1) {
        if in_the_rooms(inside, crypt.span) {
            build_room(
                world,
                IVec3::new(here.x, crypt.corner.y, here.z),
                inside,
                crypt.span,
                chest,
            );
            return;
        }
        if in_the_tunnel(inside, crypt.span) {
            set(
                world,
                here.x,
                crypt.corner.y,
                here.z,
                BlockKind::Cobblestone,
            );
            fill(
                world,
                here.x,
                here.z,
                crypt.corner.y + 1,
                crypt.corner.y + theme::DUNGEON_TALL - 1,
                BlockKind::Air,
            );
            set(
                world,
                here.x,
                crypt.corner.y + theme::DUNGEON_TALL,
                here.z,
                BlockKind::Cobblestone,
            );
            let along = inside
                .0
                .max(inside.1)
                .min(crypt.span - 1 - inside.0.min(inside.1))
                .abs();
            if along.rem_euclid(4) == 0 {
                set(
                    world,
                    here.x,
                    crypt.corner.y + theme::DUNGEON_TALL - 1,
                    here.z,
                    BlockKind::Glowstone,
                );
            }
            return;
        }
    }
    let (along, perpendicular) = along_segment(here.point, crypt.stair_bottom, crypt.stair_top);
    let run = theme::STAIR_RUN as f32;
    if along < 0.0 || along > run + 1.5 {
        return;
    }
    let share = (along / run).clamp(0.0, 1.0);
    let floor = lerp(crypt.corner.y, crypt.top, share);
    if perpendicular <= 1.0 {
        fill(
            world,
            here.x,
            here.z,
            floor - 2,
            floor,
            BlockKind::Cobblestone,
        );
        let ceiling = if along >= run - 3.0 {
            here.top.max(floor + theme::TUNNEL_HEIGHT)
        } else {
            floor + theme::TUNNEL_HEIGHT
        };
        fill(world, here.x, here.z, floor + 1, ceiling, BlockKind::Air);
        if along.rem_euclid(5.0) < 1.0 && perpendicular > 0.5 {
            set(world, here.x, floor + 1, here.z, BlockKind::Torch);
        }
        if along >= run {
            set(world, here.x, crypt.top + 4, here.z, BlockKind::Cobblestone);
        }
    } else if perpendicular <= 2.0 && along >= run {
        fill(
            world,
            here.x,
            here.z,
            crypt.top + 1,
            crypt.top + 4,
            BlockKind::Cobblestone,
        );
    }
}

fn mine_tunnel(world: &mut VoxelWorld, generation: &Generation, mine: &Mine, here: &Column) {
    let length = (mine.end - mine.mouth).magnitude();
    let (along, perpendicular) = along_segment(here.point, mine.mouth, mine.end);
    if along < -3.0 || along > length || perpendicular > 2.2 {
        return;
    }
    let share = (along / length).clamp(0.0, 1.0);
    let floor = lerp(mine.mouth_y, mine.end_y, share);
    if perpendicular <= 1.0 {
        if along >= 0.0 {
            set(world, here.x, floor, here.z, BlockKind::Gravel);
        }
        let ceiling = if along <= 3.0 {
            here.top.max(floor + theme::TUNNEL_HEIGHT)
        } else {
            floor + theme::TUNNEL_HEIGHT
        };
        fill(world, here.x, here.z, floor + 1, ceiling, BlockKind::Air);
        if along > 3.0 && along.rem_euclid(4.0) < 1.0 {
            set(
                world,
                here.x,
                floor + theme::TUNNEL_HEIGHT + 1,
                here.z,
                BlockKind::OakPlanks,
            );
        }
        if along > 3.0 && along.rem_euclid(6.0) < 1.0 && perpendicular > 0.5 {
            set(world, here.x, floor + 1, here.z, BlockKind::Torch);
        }
        return;
    }
    if along <= 3.0 {
        return;
    }
    if perpendicular <= 1.7 && along.rem_euclid(4.0) < 1.0 {
        fill(
            world,
            here.x,
            here.z,
            floor + 1,
            floor + theme::TUNNEL_HEIGHT,
            BlockKind::OakLog,
        );
        return;
    }
    if hash(generation.seed ^ 0x0A1E_0B1D, here.x, here.z) % 100 < 22 {
        let ore = if share < 0.4 {
            BlockKind::CoalOre
        } else if share < 0.75 {
            BlockKind::IronOre
        } else {
            BlockKind::GoldOre
        };
        for y in floor + 1..=floor + theme::TUNNEL_HEIGHT {
            if is_stone(world, here.x, y, here.z) {
                set(world, here.x, y, here.z, ore);
            }
        }
    }
}

fn deep_chamber(world: &mut VoxelWorld, generation: &Generation, zone: &ZonePlan, here: &Column) {
    let center = zone_center(zone);
    let distance = (here.point - center).magnitude();
    if distance > theme::DEEP_RADIUS + 1.5 {
        return;
    }
    let level = zone.level;
    if distance > theme::DEEP_RADIUS {
        if hash(generation.seed ^ 0x601D_0DE5, here.x, here.z) % 100 < 20 {
            for y in level + 1..=level + 5 {
                if is_stone(world, here.x, y, here.z) {
                    set(world, here.x, y, here.z, BlockKind::GoldOre);
                }
            }
        }
        return;
    }
    set(world, here.x, level, here.z, BlockKind::Cobblestone);
    fill(world, here.x, here.z, level + 1, level + 6, BlockKind::Air);
    set(world, here.x, level + 7, here.z, BlockKind::Stone);
    for pillar in 0..4 {
        let angle = TAU / 8.0 + pillar as f32 * TAU / 4.0;
        let spot = center + direction(angle) * 5.0;
        if (spot.x.floor() as i32, spot.y.floor() as i32) == (here.x, here.z) {
            fill(
                world,
                here.x,
                here.z,
                level + 1,
                level + 5,
                BlockKind::Stone,
            );
            set(world, here.x, level + 6, here.z, BlockKind::Glowstone);
            return;
        }
    }
    if here.x == zone.center.0 + 3 && here.z == zone.center.1 {
        set(world, here.x, level + 1, here.z, BlockKind::Chest);
    }
}

fn harbor(world: &mut VoxelWorld, zone: &ZonePlan, start: Vec2, end: Vec2, here: &Column) {
    let (along, perpendicular) = along_segment(here.point, start, end);
    let deck = theme::SEA_LEVEL + 1;
    let on_pier = (0.0..=theme::PIER_LENGTH).contains(&along)
        && (perpendicular <= 1.0 || (along >= theme::PIER_LENGTH - 3.0 && perpendicular <= 2.0));
    if on_pier {
        fill(world, here.x, here.z, deck + 1, deck + 5, BlockKind::Air);
        set(world, here.x, deck, here.z, BlockKind::OakPlanks);
        return;
    }
    if (0.0..=theme::PIER_LENGTH).contains(&along)
        && perpendicular <= 1.6
        && along.rem_euclid(3.0) < 1.0
    {
        fill(world, here.x, here.z, deck - 3, deck + 1, BlockKind::OakLog);
        fill(world, here.x, here.z, deck + 2, deck + 5, BlockKind::Air);
        if along.rem_euclid(6.0) < 1.0 {
            set(world, here.x, deck + 2, here.z, BlockKind::Torch);
        }
        return;
    }
    let toward = (end - start).normalize();
    let house = zone_center(zone) - toward * 4.0;
    let (u, v) = (
        here.x - house.x.floor() as i32,
        here.z - house.y.floor() as i32,
    );
    let door = axis_of(toward.y.atan2(toward.x));
    if hut(world, here, u, v, zone.level, 3, 2, door, 3) {
        return;
    }
    let across = Vec2::new(-toward.y, toward.x);
    let crates = zone_center(zone) + across * 4.0;
    let (cu, cv) = (
        here.x - crates.x.floor() as i32,
        here.z - crates.y.floor() as i32,
    );
    if (0..=1).contains(&cu) && (0..=1).contains(&cv) {
        fill(
            world,
            here.x,
            here.z,
            zone.level + 1,
            zone.level + 1 + cu,
            BlockKind::OakPlanks,
        );
    }
}

fn hamlet(world: &mut VoxelWorld, zone: &ZonePlan, here: &Column) {
    let (u, v) = (here.x - zone.center.0, here.z - zone.center.1);
    let level = zone.level;
    for (hut_u, hut_v) in [(-8, -6), (8, -6), (9, 7)] {
        let door = axis_of((-hut_v as f32).atan2(-hut_u as f32));
        if hut(world, here, u - hut_u, v - hut_v, level, 2, 2, door, 3) {
            return;
        }
    }
    campfire(world, u, v, level, here);
    for (post_u, post_v) in [(5, 0), (-5, 0), (0, 5), (0, -5)] {
        if u == post_u && v == post_v {
            fill(
                world,
                here.x,
                here.z,
                level + 1,
                level + 2,
                BlockKind::OakLog,
            );
            set(world, here.x, level + 3, here.z, BlockKind::Torch);
        }
    }
}

fn campfire(world: &mut VoxelWorld, u: i32, v: i32, level: i32, here: &Column) {
    if u.abs() <= 1 && v.abs() <= 1 {
        set(world, here.x, level, here.z, BlockKind::Cobblestone);
        fill(world, here.x, here.z, level + 1, level + 3, BlockKind::Air);
        if u == 0 && v == 0 {
            set(world, here.x, level + 1, here.z, BlockKind::Campfire);
        }
    }
}

fn clearing(world: &mut VoxelWorld, zone: &ZonePlan, here: &Column) {
    let (u, v) = (here.x - zone.center.0, here.z - zone.center.1);
    let level = zone.level;
    campfire(world, u, v, level, here);
    if ((-3..=-1).contains(&u) && v == 3) || ((1..=3).contains(&u) && v == -3) {
        set(world, here.x, level + 1, here.z, BlockKind::OakLog);
    }
}

fn field(world: &mut VoxelWorld, zone: &ZonePlan, here: &Column) {
    let (u, v) = (here.x - zone.center.0, here.z - zone.center.1);
    for (rock_u, rock_v) in [(7, 4), (-8, -6), (3, -9)] {
        if (u - rock_u).abs() <= 1 && v == rock_v {
            set(world, here.x, here.top + 1, here.z, BlockKind::Cobblestone);
            if u == rock_u {
                set(world, here.x, here.top + 2, here.z, BlockKind::Cobblestone);
            }
        }
    }
}
