use crate::commands::open_battle;
use crate::contract::Place;
use crate::resources::{Arguments, Battle, Field, Party, Rig, Sheets};
use crate::theme;
use nightshade::prelude::*;
use nightshade_camera::prelude::*;
use nightshade_graphics::prelude::*;
use nightshade_render::data::decoded_image::decode_to_rgba8;
use nightshade_sprite::prelude::*;

pub fn initialize(
    mut settings: ResMut<RenderSettings>,
    mut atlases: ResMut<SpriteAtlases>,
    mut active_camera: ResMut<ActiveCamera>,
    mut rig: ResMut<Rig>,
    mut field: ResMut<Field>,
    arguments: Res<Arguments>,
    mut commands: Commands,
) {
    settings.atmosphere = Atmosphere::None;
    spawn_sun_without_shadows(&mut commands);

    rig.rng = fastrand::Rng::with_seed(0x5eed_1234);
    rig.sheets = pack_the_atlas(&mut atlases);
    rig.camera = Some(nightshade_camera::commands::scene::spawn_flat_camera(
        &mut commands,
        "Field Camera".to_string(),
        Vec3::new(0.0, 0.0, theme::CAMERA_DEPTH),
        8.0,
        0.1,
        1000.0,
    ));
    active_camera.0 = rig.camera;

    field.pending = Some((opening_place(&arguments), None));
}

pub fn prime_the_battle(
    arguments: Res<Arguments>,
    mut battle: ResMut<Battle>,
    mut rig: ResMut<Rig>,
    party: Res<Party>,
) {
    if !arguments.battle {
        return;
    }
    open_battle(&mut battle, &mut rig, &party);
}

fn opening_place(arguments: &Arguments) -> Place {
    match arguments.place.as_deref() {
        Some("town") => Place::Town,
        Some("dungeon") | Some("vault") => Place::Dungeon,
        _ => Place::Overworld,
    }
}

fn pack_the_atlas(atlases: &mut SpriteAtlases) -> Sheets {
    sprite_atlas_create(
        atlases,
        theme::ATLAS,
        theme::ATLAS_SIZE,
        SpriteFilter::Nearest,
    );

    let town = insert_sheet(atlases, theme::TOWN_SHEET, theme::TOWN_BYTES);
    let dungeon = insert_sheet(atlases, theme::DUNGEON_SHEET, theme::DUNGEON_BYTES);
    let water = insert_tile(atlases, theme::WATER_SHEET, &water_tile());
    let gem = insert_tile(atlases, theme::GEM_SHEET, &gem_tile());
    let shade = insert_tile(atlases, theme::SHADE_SHEET, &shade_tile());

    Sheets {
        town,
        dungeon,
        water,
        gem,
        shade,
    }
}

fn insert_sheet(atlases: &mut SpriteAtlases, name: &str, bytes: &[u8]) -> SpriteRegion {
    let decoded = decode_to_rgba8(bytes).expect("the kenney sheet decodes");
    let (width, height) = (decoded.width, decoded.height);
    sprite_atlas_insert(atlases, theme::ATLAS, name, &decoded.rgba, width, height)
        .expect("the sheet fits the atlas")
}

fn insert_tile(atlases: &mut SpriteAtlases, name: &str, pixels: &[u8]) -> SpriteRegion {
    sprite_atlas_insert(
        atlases,
        theme::ATLAS,
        name,
        pixels,
        theme::TILE_TEXELS,
        theme::TILE_TEXELS,
    )
    .expect("the tile fits the atlas")
}

fn water_tile() -> Vec<u8> {
    let mut pixels = flat_tile(theme::WATER_COLOR);
    let glints: [(u32, u32); 6] = [(2, 3), (3, 3), (9, 6), (10, 6), (5, 11), (12, 12)];
    for (x, y) in glints {
        paint(&mut pixels, x, y, theme::WATER_GLINT);
    }
    pixels
}

fn gem_tile() -> Vec<u8> {
    let mut pixels = flat_tile([0, 0, 0, 0]);
    let middle = theme::TILE_TEXELS as i32 / 2;
    for y in 0..theme::TILE_TEXELS as i32 {
        for x in 0..theme::TILE_TEXELS as i32 {
            let span = (x - middle).abs() + ((y - middle).abs() * 3) / 4;
            if span > 6 {
                continue;
            }
            let color = if span > 4 {
                theme::GEM_EDGE
            } else {
                theme::GEM_CORE
            };
            paint(&mut pixels, x as u32, y as u32, color);
        }
    }
    for (x, y) in [(6, 5), (7, 5), (6, 6)] {
        paint(&mut pixels, x, y, theme::GEM_SHINE);
    }
    pixels
}

fn shade_tile() -> Vec<u8> {
    let mut pixels = flat_tile([0, 0, 0, 0]);
    let middle = theme::TILE_TEXELS as f32 / 2.0;
    for y in 0..theme::TILE_TEXELS {
        for x in 0..theme::TILE_TEXELS {
            let offset_x = (x as f32 + 0.5 - middle) / 7.0;
            let offset_y = (y as f32 + 0.5 - middle) / 3.4;
            if offset_x * offset_x + offset_y * offset_y <= 1.0 {
                paint(&mut pixels, x, y, theme::SHADE_COLOR);
            }
        }
    }
    pixels
}

fn flat_tile(color: [u8; 4]) -> Vec<u8> {
    let texels = (theme::TILE_TEXELS * theme::TILE_TEXELS) as usize;
    let mut pixels = Vec::with_capacity(texels * 4);
    for _ in 0..texels {
        pixels.extend_from_slice(&color);
    }
    pixels
}

fn paint(pixels: &mut [u8], x: u32, y: u32, color: [u8; 4]) {
    if x >= theme::TILE_TEXELS || y >= theme::TILE_TEXELS {
        return;
    }
    let offset = ((y * theme::TILE_TEXELS + x) * 4) as usize;
    pixels[offset..offset + 4].copy_from_slice(&color);
}
