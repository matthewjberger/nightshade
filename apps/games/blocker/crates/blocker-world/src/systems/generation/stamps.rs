use super::layout::{open_the_gates, raise_the_wall};
use super::{Site, Stamping, owns, set};
use crate::data::world::schematics::{Schematic, schematic_block};
use crate::queries::{biome_for, rotate_block, rotate_local};
use crate::theme;
use blocker_blocks::data::blocks::BlockKind;
use nalgebra_glm::IVec3;

pub(super) fn queue_stamp(
    site: &mut Site,
    schematic: usize,
    origin: IVec3,
    rot: u8,
    paved: bool,
    held: bool,
) {
    site.stamps.push(Stamping {
        schematic,
        origin,
        rot,
        paved,
        held,
    });
}

fn footprint(stamp: &Stamping, piece: &Schematic) -> [i32; 4] {
    let reach = piece.span / 2
        + if piece.foundation {
            theme::BUILDING_APRON
        } else {
            0
        };
    [
        stamp.origin.x - reach,
        stamp.origin.x + reach,
        stamp.origin.z - reach,
        stamp.origin.z + reach,
    ]
}

fn footprints_cross(left: [i32; 4], right: [i32; 4]) -> bool {
    left[0] <= right[1] && right[0] <= left[1] && left[2] <= right[3] && right[2] <= left[3]
}

fn stamp_is_blocked(stamps: &[Stamping], kept: &[bool], index: usize) -> bool {
    let stamp = &stamps[index];
    let piece = &crate::data::world::schematics::SCHEMATICS[stamp.schematic];
    let mine = footprint(stamp, piece);
    stamps.iter().enumerate().any(|(other_index, other)| {
        if other_index == index || !kept[other_index] {
            return false;
        }
        let other_piece = &crate::data::world::schematics::SCHEMATICS[other.schematic];
        if piece.foundation {
            other_piece.foundation && footprints_cross(mine, footprint(other, other_piece))
        } else {
            pad_holds(other, other_piece, stamp.origin)
        }
    })
}

fn pad_holds(stamp: &Stamping, piece: &Schematic, at: IVec3) -> bool {
    let reach = piece.span / 2 + theme::BUILDING_APRON;
    piece.foundation
        && (at.x - stamp.origin.x).abs() <= reach
        && (at.z - stamp.origin.z).abs() <= reach
        && at.y >= stamp.origin.y - 1
        && at.y <= stamp.origin.y + piece.layers.len() as i32 + 1
}

pub(super) fn finish_the_stamps(site: &mut Site) {
    let stamps = std::mem::take(&mut site.stamps);
    let mut kept: Vec<bool> = stamps.iter().map(|stamp| stamp.held).collect();
    for index in 0..stamps.len() {
        if stamps[index].held {
            continue;
        }
        kept[index] = !stamp_is_blocked(&stamps, &kept, index);
    }
    for (stamp, keep) in stamps.iter().zip(kept.iter()) {
        let piece = &crate::data::world::schematics::SCHEMATICS[stamp.schematic];
        if *keep && piece.foundation {
            let top = piece.layers.len() as i32 - piece.sink;
            flatten(site, stamp.origin, piece.span, top, stamp.paved);
        }
    }
    let walls = std::mem::take(&mut site.walls);
    for rampart in walls {
        raise_the_wall(site, rampart.center, rampart.reach);
        open_the_gates(site, rampart.center, rampart.reach, rampart.path);
    }
    for (stamp, keep) in stamps.iter().zip(kept.iter()) {
        if !*keep {
            continue;
        }
        let piece = &crate::data::world::schematics::SCHEMATICS[stamp.schematic];
        raise_the_layers(site, piece, stamp.origin, stamp.rot);
    }
    let markers = std::mem::take(&mut site.markers);
    for (position, kind) in markers {
        set(site.world, position, kind, site.column);
    }
}

fn raise_the_layers(site: &mut Site, schematic: &Schematic, origin: IVec3, rot: u8) {
    let half = schematic.span / 2;
    for (layer_index, layer) in schematic.layers.iter().enumerate() {
        let world_y = origin.y - schematic.sink + layer_index as i32;
        for (row_index, row) in layer.iter().enumerate() {
            let cells = expand_row(row);
            for local_x in 0..schematic.span {
                let glyph = cells.get(local_x as usize).copied().unwrap_or(0);
                let Some(kind) = schematic_block(glyph) else {
                    continue;
                };
                let (rotated_x, rotated_z) =
                    rotate_local(local_x, row_index as i32, schematic.span, rot);
                let position = IVec3::new(
                    origin.x - half + rotated_x,
                    world_y,
                    origin.z - half + rotated_z,
                );
                set(site.world, position, rotate_block(kind, rot), site.column);
            }
        }
    }
    for plaque in schematic.plaques {
        let (rotated_x, rotated_z) = rotate_local(plaque.x, plaque.z, schematic.span, rot);
        let position = IVec3::new(
            origin.x - half + rotated_x,
            origin.y - schematic.sink + plaque.layer,
            origin.z - half + rotated_z,
        );
        if !owns(site.column, position) {
            continue;
        }
        set(site.world, position, BlockKind::Signpost, site.column);
        site.plaques.push((
            [position.x, position.y, position.z],
            plaque.text.to_string(),
        ));
    }
}

fn expand_row(row: &str) -> [u8; 48] {
    let mut cells = [0u8; 48];
    let mut count = 0usize;
    let bytes = row.as_bytes();
    let mut index = 0;
    while index < bytes.len() && count < cells.len() {
        let mut run = 0usize;
        while index < bytes.len() && bytes[index].is_ascii_digit() {
            run = run * 10 + (bytes[index] - b'0') as usize;
            index += 1;
        }
        let Some(&glyph) = bytes.get(index) else {
            break;
        };
        index += 1;
        for _ in 0..run.max(1) {
            if count < cells.len() {
                cells[count] = glyph;
                count += 1;
            }
        }
    }
    cells
}

fn flatten(site: &mut Site, origin: IVec3, span: i32, top: i32, paved: bool) {
    let half = span / 2;
    let apron = half + theme::BUILDING_APRON;
    for offset_z in -apron..=apron {
        for offset_x in -apron..=apron {
            let x = origin.x + offset_x;
            let z = origin.z + offset_z;
            let edge = offset_x.abs() > half || offset_z.abs() > half;
            let body = if paved {
                BlockKind::Stone
            } else {
                BlockKind::Dirt
            };
            let surface = if !edge {
                body
            } else if paved {
                BlockKind::Cobblestone
            } else {
                biome_for(site.bench, x, z, origin.y).info().surface
            };
            for y in (origin.y - 12)..=origin.y {
                let kind = if y == origin.y { surface } else { body };
                set(site.world, IVec3::new(x, y, z), kind, site.column);
            }
            for y in (origin.y + 1)..=(origin.y + top.max(7)) {
                set(site.world, IVec3::new(x, y, z), BlockKind::Air, site.column);
            }
        }
    }
}
