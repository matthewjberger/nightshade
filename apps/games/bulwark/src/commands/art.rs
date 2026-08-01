use crate::contract::{SCENERY_KINDS, TOWER_KINDS, TOWER_LEVELS};
use crate::data::{self, Piece};
use crate::resources::Art;
use crate::theme;
use nalgebra_glm::Vec2;
use nightshade_render::data::decoded_image::decode_to_rgba8;
use nightshade_sprite::prelude::*;

macro_rules! sheet {
    ($name:literal) => {
        (
            $name,
            include_bytes!(concat!("../../assets/kenney_tower_defense/", $name, ".png")) as &[u8],
        )
    };
}

const GRASS_SHEET: (&str, &[u8]) = sheet!("grass");

const ROAD_SHEETS: [(&str, &[u8]); 11] = [
    sheet!("road_03"),
    sheet!("road_05"),
    sheet!("road_06"),
    sheet!("road_07"),
    sheet!("road_09"),
    sheet!("road_10"),
    sheet!("road_11"),
    sheet!("road_12"),
    sheet!("road_13"),
    sheet!("road_14"),
    sheet!("road_15"),
];

const SCENERY_SHEETS: [(&str, &[u8]); SCENERY_KINDS] = [
    sheet!("pine_pair"),
    sheet!("pine_tall"),
    sheet!("pine_trio"),
    sheet!("oak_broad"),
    sheet!("oak_pair"),
    sheet!("rocks_low"),
    sheet!("rocks_tall"),
    sheet!("rocks_pair"),
    sheet!("rocks_dome"),
    sheet!("rocks_trio"),
    sheet!("rocks_slab"),
    sheet!("scrub_pair"),
    sheet!("scrub_low"),
    sheet!("pine_stand"),
    sheet!("oak_lone"),
];

const TOWER_SHEETS: [[(&str, &[u8]); TOWER_LEVELS]; TOWER_KINDS] = [
    [
        sheet!("bastion_1"),
        sheet!("bastion_2"),
        sheet!("bastion_3"),
    ],
    [
        sheet!("longbow_1"),
        sheet!("longbow_2"),
        sheet!("longbow_3"),
    ],
    [sheet!("hearth_1"), sheet!("hearth_2"), sheet!("hearth_3")],
    [
        sheet!("barbican_1"),
        sheet!("barbican_2"),
        sheet!("barbican_3"),
    ],
    [sheet!("beacon_1"), sheet!("beacon_2"), sheet!("beacon_3")],
    [sheet!("cairn_1"), sheet!("cairn_2"), sheet!("cairn_3")],
];

const SHARD_SHEETS: [(&str, &[u8]); 3] = [
    sheet!("shard_small"),
    sheet!("shard_mid"),
    sheet!("shard_large"),
];

const KEEP_SHEET: (&str, &[u8]) = sheet!("keep");

fn block_above_bottom(rgba: &[u8], width: u32, height: u32) -> f32 {
    let painted = |x: u32, y: u32| {
        rgba.get(((y * width + x) * 4 + 3) as usize)
            .is_some_and(|alpha| *alpha > 8)
    };
    let span = |y: u32| {
        let left = (0..width).find(|x| painted(*x, y));
        let right = (0..width).rev().find(|x| painted(*x, y));
        left.zip(right).map_or(0, |(left, right)| right - left + 1)
    };
    let floor = (0..height).rev().find(|y| span(*y) > 0).unwrap_or(0);
    let mut base = floor;
    while base > 0 && span(base - 1) > span(base) {
        base -= 1;
    }
    (height - base) as f32
}

fn packed(
    atlases: &mut SpriteAtlases,
    sheet: (&str, &[u8]),
    above_bottom: Option<f32>,
    scale: f32,
) -> Piece {
    let decoded = decode_to_rgba8(sheet.1).expect("the packed picture decodes");
    let region = sprite_atlas_insert(
        atlases,
        theme::ATLAS,
        sheet.0,
        &decoded.rgba,
        decoded.width,
        decoded.height,
    )
    .expect("the picture fits the atlas");
    let width = decoded.width as f32;
    let height = decoded.height as f32;
    let lift = above_bottom
        .unwrap_or_else(|| block_above_bottom(&decoded.rgba, decoded.width, decoded.height));
    Piece {
        region,
        size: Vec2::new(width, height) * scale / theme::PIXELS_PER_UNIT,
        anchor: Vec2::new(0.5, lift / height),
    }
}

fn painted(
    atlases: &mut SpriteAtlases,
    name: &str,
    side: u32,
    rgba: Vec<u8>,
    anchor: Vec2,
) -> Piece {
    let region = sprite_atlas_insert(atlases, theme::ATLAS, name, &rgba, side, side)
        .expect("the painted picture fits the atlas");
    let side = side as f32;
    Piece {
        region,
        size: Vec2::new(side, side) / theme::PIXELS_PER_UNIT,
        anchor,
    }
}

fn marker_paint() -> Vec<u8> {
    let side = theme::MARKER_TEXELS as f32;
    let half = side * 0.5;
    nightshade_texture::queries::paint(theme::MARKER_TEXELS, |x, y| {
        let across = (x as f32 + 0.5 - half) / half;
        let into = (y as f32 + 0.5 - half) / (half * 0.5);
        let reach = across.abs() + into.abs();
        if (0.74..=1.0).contains(&reach) {
            [255, 255, 255, 255]
        } else if reach < 0.74 {
            [255, 255, 255, 84]
        } else {
            [255, 255, 255, 0]
        }
    })
}

fn ring_paint() -> Vec<u8> {
    let half = theme::RING_TEXELS as f32 * 0.5;
    nightshade_texture::queries::paint(theme::RING_TEXELS, |x, y| {
        let across = x as f32 + 0.5 - half;
        let into = y as f32 + 0.5 - half;
        let reach = (across * across + into * into).sqrt() / half;
        if (0.93..=1.0).contains(&reach) {
            [255, 255, 255, 255]
        } else if reach < 0.93 {
            [255, 255, 255, 46]
        } else {
            [255, 255, 255, 0]
        }
    })
}

fn shade_paint() -> Vec<u8> {
    let half = theme::SHADE_TEXELS as f32 * 0.5;
    nightshade_texture::queries::paint(theme::SHADE_TEXELS, |x, y| {
        let across = (x as f32 + 0.5 - half) / half;
        let into = (y as f32 + 0.5 - half) / half;
        let reach = (across * across + into * into).sqrt();
        let softness = (1.0 - reach).clamp(0.0, 1.0);
        [255, 255, 255, (softness.powf(1.1) * 255.0) as u8]
    })
}

fn backdrop_paint() -> Vec<u8> {
    let half = theme::BACKDROP_TEXELS as f32 * 0.5;
    nightshade_texture::queries::paint(theme::BACKDROP_TEXELS, |x, y| {
        let across = (x as f32 + 0.5 - half) / half;
        let into = (y as f32 + 0.5 - half) / half;
        let reach = (across * across + into * into).sqrt().clamp(0.0, 1.0);
        let share = reach * reach * (3.0 - 2.0 * reach);
        let mixed = |slot: usize| {
            let core = theme::BACKDROP_CORE[slot] as f32;
            let edge = theme::BACKDROP_EDGE[slot] as f32;
            (core + (edge - core) * share) as u8
        };
        [mixed(0), mixed(1), mixed(2), 255]
    })
}

fn bead_paint() -> Vec<u8> {
    let half = theme::BEAD_TEXELS as f32 * 0.5;
    nightshade_texture::queries::paint(theme::BEAD_TEXELS, |x, y| {
        let across = x as f32 + 0.5 - half;
        let into = y as f32 + 0.5 - half;
        let reach = (across * across + into * into).sqrt() / half;
        if reach <= 0.55 {
            theme::BEAD_CORE
        } else if reach <= 1.0 {
            theme::BEAD_EDGE
        } else {
            [0, 0, 0, 0]
        }
    })
}

pub fn pack_the_art(atlases: &mut SpriteAtlases) -> Art {
    sprite_atlas_create(
        atlases,
        theme::ATLAS,
        theme::ATLAS_SIZE,
        SpriteFilter::Nearest,
    );
    let ground = Some(theme::GROUND_ABOVE_BOTTOM);

    let mut art = Art {
        grass: packed(atlases, GRASS_SHEET, ground, 1.0),
        ..Default::default()
    };
    for (slot, sheet) in ROAD_SHEETS.into_iter().enumerate() {
        let mask = data::ROAD_MASKS[slot] as usize;
        art.road[mask] = packed(atlases, sheet, ground, 1.0);
    }
    for (slot, sheet) in SCENERY_SHEETS.into_iter().enumerate() {
        art.scenery[slot] = packed(atlases, sheet, ground, 1.0);
    }
    for (family, sheets) in TOWER_SHEETS.into_iter().enumerate() {
        for (level, sheet) in sheets.into_iter().enumerate() {
            art.towers[family][level] = packed(atlases, sheet, None, 1.0);
        }
    }
    for (slot, sheet) in SHARD_SHEETS.into_iter().enumerate() {
        art.shards[slot] = packed(
            atlases,
            sheet,
            Some(theme::SHARD_ABOVE_BOTTOM),
            data::SHARD_SCALES[slot],
        );
    }
    art.keep = packed(atlases, KEEP_SHEET, None, 1.0);
    art.marker = painted(
        atlases,
        theme::MARKER,
        theme::MARKER_TEXELS,
        marker_paint(),
        Vec2::new(0.5, 0.5),
    );
    art.ring = painted(
        atlases,
        theme::RING,
        theme::RING_TEXELS,
        ring_paint(),
        Vec2::new(0.5, 0.5),
    );
    art.bead = painted(
        atlases,
        theme::BEAD,
        theme::BEAD_TEXELS,
        bead_paint(),
        Vec2::new(0.5, 0.5),
    );
    art.bar = painted(
        atlases,
        theme::BAR,
        theme::BAR_TEXELS,
        nightshade_texture::queries::paint(theme::BAR_TEXELS, |_, _| [255, 255, 255, 255]),
        Vec2::new(0.0, 0.5),
    );
    art.shade = painted(
        atlases,
        theme::SHADE,
        theme::SHADE_TEXELS,
        shade_paint(),
        Vec2::new(0.5, 0.5),
    );
    art.backdrop = painted(
        atlases,
        theme::BACKDROP,
        theme::BACKDROP_TEXELS,
        backdrop_paint(),
        Vec2::new(0.5, 0.5),
    );

    for mask in 0..art.road.len() {
        if art.road[mask].size.x <= 0.0 {
            art.road[mask] = art.road[(data::ROAD_MASKS[0]) as usize];
        }
    }
    art
}

pub fn sprite_of(piece: Piece, layer: i32) -> Sprite {
    Sprite::new(theme::ATLAS, piece.region)
        .with_size(piece.size.x, piece.size.y)
        .with_anchor(piece.anchor)
        .with_layer(layer)
}
