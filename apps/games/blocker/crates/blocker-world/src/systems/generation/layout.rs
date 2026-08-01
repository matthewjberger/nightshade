use super::{Instance, Rampart, Site, ground_at, owns, set};
use crate::data::world::nodes::{Layout, SlotRole, slot_role};
use crate::queries::{hash, surface_height, terrace_reach, village_name};
use crate::resources::Bench;
use crate::theme;
use blocker_blocks::data::blocks::BlockKind;
use blocker_blocks::resources::ColumnKey;
use nalgebra_glm::IVec3;

pub(super) fn lay_out_the_village(
    site: &mut Site,
    queue: &mut Vec<Instance>,
    instance: &Instance,
    layout: &Layout,
) {
    let ground = if layout.wall > 0 {
        instance
            .origin
            .y
            .max(crate::data::world::SEA_LEVEL + theme::SETTLEMENT_LIFT + 1)
    } else {
        instance.origin.y.max(crate::data::world::SEA_LEVEL + 1)
    };
    let center = IVec3::new(instance.origin.x, ground, instance.origin.z);
    let floor = if layout.wall > 0 { ground - 1 } else { ground };
    queue.push(Instance {
        node: layout.center,
        origin: IVec3::new(center.x, floor, center.z),
        rot: 0,
        seed: instance.seed ^ 0xB105,
        paved: layout.wall > 0,
        held: true,
    });
    let heart = [center.x, floor + 1, center.z];
    match slot_role(layout.center) {
        SlotRole::Home => site.homes.push(([center.x, center.z], heart)),
        SlotRole::Guide => site.guides.push(([center.x, center.z], heart)),
        SlotRole::Work(kind) => site.posts.push(([center.x, center.z], heart, kind)),
        SlotRole::Light => {}
    }
    let level = (layout.wall > 0).then_some(floor);
    let sign_base = level.unwrap_or_else(|| ground_at(site.bench, center.x, center.z + 3));
    let sign = IVec3::new(center.x, sign_base + 1, center.z + 3);
    if owns(site.column, sign) {
        set(site.world, sign, BlockKind::Signpost, site.column);
        let name = crate::queries::seat_at(site.bench, center.x, center.z)
            .map(|capital| capital.name.to_string())
            .unwrap_or_else(|| village_name(site.seed, center.x, center.z));
        site.signs.push(([sign.x, sign.y, sign.z], name));
    }
    if layout.wall > 0 {
        level_the_terrace(site, center, terrace_reach(layout));
        site.walls.push(Rampart {
            center,
            reach: layout.wall,
            path: layout.path,
        });
        let board = IVec3::new(
            center.x + theme::GATE_BOARD[0],
            floor + 1,
            center.z + layout.wall + theme::GATE_BOARD[1],
        );
        if owns(site.column, board) {
            site.markers.push((board, BlockKind::Signpost));
            site.plaques
                .push(([board.x, board.y, board.z], crate::queries::gate_notice()));
        }
    }
    let reach = layout.span / 2;
    for (corner_x, corner_z) in [
        (-reach, -reach),
        (reach, -reach),
        (-reach, reach),
        (reach, reach),
    ] {
        let base = level
            .unwrap_or_else(|| ground_at(site.bench, center.x + corner_x, center.z + corner_z));
        let post = IVec3::new(center.x + corner_x, base + 1, center.z + corner_z);
        set(site.world, post, BlockKind::Torch, site.column);
    }
    for street in layout.streets {
        let from = IVec3::new(
            center.x + street.from[0],
            center.y,
            center.z + street.from[1],
        );
        let to = IVec3::new(center.x + street.to[0], center.y, center.z + street.to[1]);
        draw_lane(site, from, to, street.wide, layout.path, level);
    }
    for slot in layout.slots {
        let anchor_x = center.x + slot.dx;
        let anchor_z = center.z + slot.dz;
        let anchor = IVec3::new(
            anchor_x,
            level.unwrap_or_else(|| ground_at(site.bench, anchor_x, anchor_z)),
            anchor_z,
        );
        let face = slot.face;
        let step = door_step(anchor, face, site.bench, level);
        match slot_role(slot.child) {
            SlotRole::Home => site.homes.push(([center.x, center.z], step)),
            SlotRole::Guide => site.guides.push(([center.x, center.z], step)),
            SlotRole::Work(kind) => site.posts.push(([center.x, center.z], step, kind)),
            SlotRole::Light => {}
        }
        queue.push(Instance {
            node: slot.child,
            origin: anchor,
            rot: face,
            seed: hash(instance.seed, slot.dx, slot.dz),
            paved: layout.wall > 0,
            held: true,
        });
    }
}

fn column_bounds(column: ColumnKey, center: IVec3, reach: i32) -> (i32, i32, i32, i32) {
    let chunk = nightshade_voxels::data::CHUNK;
    let low_x = (column[0] * chunk).max(center.x - reach);
    let high_x = (column[0] * chunk + chunk - 1).min(center.x + reach);
    let low_z = (column[1] * chunk).max(center.z - reach);
    let high_z = (column[1] * chunk + chunk - 1).min(center.z + reach);
    (low_x, high_x, low_z, high_z)
}

fn level_the_terrace(site: &mut Site, center: IVec3, reach: i32) {
    let (low_x, high_x, low_z, high_z) = column_bounds(site.column, center, reach + 2);
    for z in low_z..=high_z {
        for x in low_x..=high_x {
            let edge = (x - center.x).abs() > reach || (z - center.z).abs() > reach;
            let cap = if edge {
                BlockKind::Grass
            } else {
                BlockKind::Stone
            };
            let natural = surface_height(site.bench, x, z);
            let floor = (natural - 2).min(center.y - theme::CITY_TERRACE_FILL);
            for y in floor..center.y {
                set(
                    site.world,
                    IVec3::new(x, y, z),
                    BlockKind::Stone,
                    site.column,
                );
            }
            set(site.world, IVec3::new(x, center.y - 1, z), cap, site.column);
            let ceiling = (natural + 2).max(center.y + theme::CITY_TERRACE_CLEAR);
            for y in center.y..=ceiling {
                set(site.world, IVec3::new(x, y, z), BlockKind::Air, site.column);
            }
        }
    }
}

fn is_gate(offset: i32) -> bool {
    offset.abs() <= theme::CITY_GATE_HALF
}

pub(super) fn raise_the_wall(site: &mut Site, center: IVec3, reach: i32) {
    let (low_x, high_x, low_z, high_z) = column_bounds(site.column, center, reach);
    for z in low_z..=high_z {
        for x in low_x..=high_x {
            let offset_x = x - center.x;
            let offset_z = z - center.z;
            let on_x_side = offset_x.abs() == reach;
            let on_z_side = offset_z.abs() == reach;
            if !on_x_side && !on_z_side {
                continue;
            }
            if (on_x_side && is_gate(offset_z)) || (on_z_side && is_gate(offset_x)) {
                continue;
            }
            let corner = on_x_side && on_z_side;
            let top = if corner {
                theme::CITY_WALL_HEIGHT + theme::CITY_TOWER_RISE
            } else {
                theme::CITY_WALL_HEIGHT
            };
            for lift in 0..top {
                set(
                    site.world,
                    IVec3::new(x, center.y + lift, z),
                    BlockKind::Stone,
                    site.column,
                );
            }
            let crown = if corner || (offset_x + offset_z) % 2 == 0 {
                BlockKind::Cobblestone
            } else {
                BlockKind::Air
            };
            set(
                site.world,
                IVec3::new(x, center.y + top, z),
                crown,
                site.column,
            );
        }
    }
    for (tower_x, tower_z) in [
        (-reach, -reach),
        (reach, -reach),
        (-reach, reach),
        (reach, reach),
    ] {
        raise_the_tower(site, center, tower_x, tower_z);
    }
}

fn raise_the_tower(site: &mut Site, center: IVec3, offset_x: i32, offset_z: i32) {
    let half = theme::CITY_TOWER_HALF;
    let top = theme::CITY_WALL_HEIGHT + theme::CITY_TOWER_RISE;
    for step_z in -half..=half {
        for step_x in -half..=half {
            let x = center.x + offset_x + step_x;
            let z = center.z + offset_z + step_z;
            for lift in 0..top {
                set(
                    site.world,
                    IVec3::new(x, center.y + lift, z),
                    BlockKind::Bricks,
                    site.column,
                );
            }
            let rim = step_x.abs() == half || step_z.abs() == half;
            let crown = if rim {
                BlockKind::Cobblestone
            } else {
                BlockKind::Glowstone
            };
            set(
                site.world,
                IVec3::new(x, center.y + top, z),
                crown,
                site.column,
            );
        }
    }
}

pub(super) fn open_the_gates(site: &mut Site, center: IVec3, reach: i32, path: BlockKind) {
    for (gate_x, gate_z) in [(-reach, 0), (reach, 0), (0, -reach), (0, reach)] {
        let gate = IVec3::new(center.x + gate_x, center.y, center.z + gate_z);
        draw_path_at(site, center, gate, path, Some(center.y - 1));
        let outside = IVec3::new(
            center.x + gate_x * 3 / 2,
            center.y,
            center.z + gate_z * 3 / 2,
        );
        draw_path_at(site, gate, outside, path, None);
        let side = if gate_x == 0 { 1 } else { 0 };
        for lean in [-(theme::CITY_GATE_HALF + 2), theme::CITY_GATE_HALF + 2] {
            let (tower_x, tower_z) = if side == 1 {
                (gate_x + lean, gate_z)
            } else {
                (gate_x, gate_z + lean)
            };
            raise_the_tower(site, center, tower_x, tower_z);
        }
        for lift in 0..theme::CITY_WALL_HEIGHT {
            for span in -theme::CITY_GATE_HALF..=theme::CITY_GATE_HALF {
                let (x, z) = if side == 1 {
                    (center.x + gate_x + span, center.z + gate_z)
                } else {
                    (center.x + gate_x, center.z + gate_z + span)
                };
                let arch = lift >= theme::CITY_WALL_HEIGHT - 1;
                let kind = if arch {
                    BlockKind::Stone
                } else {
                    BlockKind::Air
                };
                set(
                    site.world,
                    IVec3::new(x, center.y + lift, z),
                    kind,
                    site.column,
                );
            }
        }
    }
}

fn door_step(anchor: IVec3, face: u8, bench: &Bench, level: Option<i32>) -> [i32; 3] {
    let (step_x, step_z) = match face {
        1 => (theme::DOOR_STEP, 0),
        2 => (0, theme::DOOR_STEP),
        3 => (-theme::DOOR_STEP, 0),
        _ => (0, -theme::DOOR_STEP),
    };
    let x = anchor.x + step_x;
    let z = anchor.z + step_z;
    [x, level.unwrap_or_else(|| ground_at(bench, x, z)) + 1, z]
}

fn draw_lane(
    site: &mut Site,
    from: IVec3,
    to: IVec3,
    wide: i32,
    block: BlockKind,
    level: Option<i32>,
) {
    let half = wide / 2;
    let along_x = from.z == to.z;
    for offset in -half..=half {
        let shift = if along_x {
            IVec3::new(0, 0, offset)
        } else {
            IVec3::new(offset, 0, 0)
        };
        draw_path_at(site, from + shift, to + shift, block, level);
    }
}

fn draw_path_at(site: &mut Site, from: IVec3, to: IVec3, block: BlockKind, level: Option<i32>) {
    let steps = (to.x - from.x).abs().max((to.z - from.z).abs()).max(1);
    for step in 0..=steps {
        let x = from.x + (to.x - from.x) * step / steps;
        let z = from.z + (to.z - from.z) * step / steps;
        let top = level.unwrap_or_else(|| ground_at(site.bench, x, z));
        set(site.world, IVec3::new(x, top, z), block, site.column);
        for lift in 1..=2 {
            set(
                site.world,
                IVec3::new(x, top + lift, z),
                BlockKind::Air,
                site.column,
            );
        }
    }
}
