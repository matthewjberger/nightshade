use crate::commands::camera::spawn_camera;
use crate::commands::cursor::spawn_cursor;
use crate::commands::daylight::dress_the_frame;
use crate::commands::look_at;
use crate::commands::textures::pack_the_sheet;
use crate::commands::{daylight, textures};
use crate::resources::{Battlefield, CameraRig, Cursor, Sheet};
use crate::theme;
use nalgebra_glm::Vec3;
use nightshade::prelude::*;
use nightshade_camera::prelude::*;
use nightshade_graphics::commands::builtins::spawn_sun;
use nightshade_graphics::prelude::*;
use nightshade_graphics::resources::loading::TextureLoad;
use nightshade_render::components::light::Light;
use nightshade_render::data::config::RendererState;
use nightshade_render::data::decoded_image::decode_to_rgba8;
use nightshade_sprite::prelude::*;

pub fn initialize(
    mut settings: ResMut<RenderSettings>,
    mut renderer_state: ResMut<RendererState>,
    mut registry: ResMut<MaterialRegistry>,
    mut atlases: ResMut<SpriteAtlases>,
    mut active_camera: ResMut<ActiveCamera>,
    mut battlefield: ResMut<Battlefield>,
    mut sheet: ResMut<Sheet>,
    mut rig: ResMut<CameraRig>,
    mut cursor: ResMut<Cursor>,
    mut textures: TextureLoad,
    mut commands: Commands,
) {
    dress_the_frame(&mut settings);
    let sun = spawn_sun(&mut commands);
    commands.set(
        sun,
        Light {
            shadow_softness: theme::SUN_SHADOW_SOFTNESS,
            ..Light::default().with_shadows(0.0005)
        },
    );
    let hour = battlefield.start_hour;
    daylight::start_the_clock(&mut settings, &mut renderer_state, sun, hour);
    daylight::spawn_moon(&mut commands);
    battlefield.sun = Some(sun);

    let decoded = decode_to_rgba8(theme::SHEET_BYTES).expect("the tiny battle sheet decodes");
    let (sheet_width, sheet_height) = (decoded.width, decoded.height);
    let town = decode_to_rgba8(theme::TOWN_BYTES).expect("the tiny town sheet decodes");
    let (town_width, town_height) = (town.width, town.height);
    textures::load_block_textures(&mut textures, &decoded.rgba, sheet_width);
    textures::load_town_textures(&mut textures, &town.rgba, town_width);
    textures::insert_materials(&mut registry);

    *sheet = pack_the_sheet(
        &mut atlases,
        (&decoded.rgba, sheet_width, sheet_height),
        (&town.rgba, town_width, town_height),
    );
    spawn_cursor(&mut commands, &sheet);
    cursor.column = theme::TERRAIN_COLUMNS / 2;
    cursor.row = theme::TERRAIN_ROWS / 2;

    let center = Vec3::new(0.0, 3.0, 0.0);
    rig.focus = center;
    look_at(&mut rig, center);
    battlefield.camera = Some(spawn_camera(&mut commands, center));
    active_camera.0 = battlefield.camera;
}
