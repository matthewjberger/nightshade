use super::stamps::queue_stamp;
use super::{Site, ground_at, owns};
use crate::data::world::nodes::NodeKind;
use crate::queries::{barrow_floor, surface_height};
use crate::theme;
use blocker_blocks::data::blocks::BlockKind;
use nalgebra_glm::IVec3;

fn within_column(site: &Site, at: [i32; 2], reach: i32) -> bool {
    let origin_x = site.column[0] * nightshade_voxels::data::CHUNK;
    let origin_z = site.column[1] * nightshade_voxels::data::CHUNK;
    at[0] + reach >= origin_x
        && at[0] - reach < origin_x + nightshade_voxels::data::CHUNK
        && at[1] + reach >= origin_z
        && at[1] - reach < origin_z + nightshade_voxels::data::CHUNK
}

fn sign_at(site: &mut Site, sign: IVec3, text: String) {
    if !owns(site.column, sign) {
        return;
    }
    site.markers.push((sign, BlockKind::Signpost));
    site.signs.push(([sign.x, sign.y, sign.z], text));
}

pub(super) fn stamp_the_landmarks(site: &mut Site) {
    for landmark in crate::data::world::places::LANDMARKS.iter() {
        let NodeKind::Stamp { schematic } = &crate::data::world::nodes::NODES[landmark.node].kind
        else {
            continue;
        };
        let piece = &crate::data::world::schematics::SCHEMATICS[*schematic];
        if !within_column(site, landmark.at, piece.span / 2 + theme::BUILDING_APRON) {
            continue;
        }
        let height = surface_height(site.bench, landmark.at[0], landmark.at[1]);
        let origin = IVec3::new(landmark.at[0], height, landmark.at[1]);
        queue_stamp(site, *schematic, origin, landmark.rot, false, true);
    }
}

pub(super) fn stamp_the_pois(site: &mut Site) {
    for poi in crate::data::world::places::POIS.iter() {
        let piece = &crate::data::world::schematics::SCHEMATICS[poi.schematic];
        if !within_column(site, poi.at, piece.span / 2 + theme::BUILDING_APRON) {
            continue;
        }
        let height =
            surface_height(site.bench, poi.at[0], poi.at[1]).max(crate::data::world::SEA_LEVEL + 1);
        let origin = IVec3::new(poi.at[0], height, poi.at[1]);
        queue_stamp(site, poi.schematic, origin, 0, false, true);
        site.signs
            .push(([poi.at[0], height + 1, poi.at[1]], poi.name.to_string()));
    }
}

pub(super) fn stamp_the_barrows(site: &mut Site) {
    let piece = &crate::data::world::schematics::SCHEMATICS
        [crate::data::world::schematics::BARROW_SCHEMATIC];
    let half = piece.span / 2 + theme::BUILDING_APRON;
    for (index, barrow) in crate::data::world::places::BARROWS.iter().enumerate() {
        let at = site.bench.seats[index].barrow;
        if !within_column(site, at, half) {
            continue;
        }
        let height = barrow_floor(site.bench, at);
        queue_stamp(
            site,
            crate::data::world::schematics::BARROW_SCHEMATIC,
            IVec3::new(at[0], height, at[1]),
            0,
            false,
            true,
        );
        let front = at[1] + piece.span / 2 + 1;
        sign_at(
            site,
            IVec3::new(at[0] + theme::BARROW_SIGN_OFFSET, height, front),
            barrow.name.to_string(),
        );
        for (index, lost) in crate::data::world::places::LOST.iter().enumerate() {
            sign_at(
                site,
                IVec3::new(at[0] + theme::MARKER_OFFSETS[index], height, front),
                format!("{}, {}", lost.name, lost.note),
            );
        }
    }
}

pub(super) fn stamp_the_camps(site: &mut Site) {
    let piece =
        &crate::data::world::schematics::SCHEMATICS[crate::data::world::schematics::CAMP_SCHEMATIC];
    for (index, camp) in crate::data::world::places::CAMPS.iter().enumerate() {
        let at = site.bench.seats[index].camp;
        if !within_column(site, at, theme::CAMP_REACH) {
            continue;
        }
        let height = ground_at(site.bench, at[0], at[1]);
        queue_stamp(
            site,
            crate::data::world::schematics::CAMP_SCHEMATIC,
            IVec3::new(at[0], height, at[1]),
            0,
            false,
            true,
        );
        for offset in crate::data::world::places::CAMP_TREES.iter() {
            let x = at[0] + offset[0];
            let z = at[1] + offset[1];
            let base = ground_at(site.bench, x, z);
            queue_stamp(
                site,
                crate::data::world::schematics::OAK_SCHEMATIC,
                IVec3::new(x, base, z),
                0,
                false,
                true,
            );
        }
        sign_at(
            site,
            IVec3::new(at[0], height, at[1] + piece.span / 2 + 1),
            camp.name.to_string(),
        );
    }
}
