use crate::commands::{want_level, want_screen};
use crate::resources::{Arguments, CoinfallResources, Screen};
use crate::systems::hud;
use crate::theme;
use nightshade::prelude::*;
use nightshade_camera::prelude::*;
use nightshade_graphics::prelude::*;
use nightshade_render::data::decoded_image::decode_to_rgba8;
use nightshade_sprite::prelude::*;
use nightshade_ui::prelude::UiTreeBuilder;

pub fn initialize(
    arguments: Res<Arguments>,
    mut resources: ResMut<CoinfallResources>,
    mut settings: ResMut<RenderSettings>,
    mut atlases: ResMut<SpriteAtlases>,
    mut active_camera: ResMut<ActiveCamera>,
    mut commands: Commands,
) {
    settings.atmosphere = Atmosphere::None;
    spawn_sun_without_shadows(&mut commands);

    pack_sheets(&mut resources, &mut atlases);
    spawn_haze(resources.sheets.blank, &mut commands);

    let camera = nightshade_camera::commands::scene::spawn_flat_camera(
        &mut commands,
        "Camera".to_string(),
        Vec3::new(0.0, 0.0, theme::CAMERA_DEPTH),
        6.0,
        0.1,
        100.0,
    );
    resources.camera = Some(camera);
    active_camera.0 = Some(camera);

    if let Some(level) = arguments.level {
        let chosen = level.min(crate::data::LEVELS.len() - 1);
        resources.run.selected = chosen;
        resources.run.unlocked = crate::data::LEVELS.len();
        want_level(&mut resources.run, chosen);
    }
    if arguments.play || arguments.level.is_some() {
        want_screen(&mut resources.run, Screen::Playing);
    } else if arguments.map {
        resources.run.unlocked = crate::data::LEVELS.len();
        want_screen(&mut resources.run, Screen::Overworld);
    }
}

pub fn build_interface(
    mut resources: ResMut<CoinfallResources>,
    mut banner: ResMut<nightshade_ui_banner::BannerHandles>,
    mut build: nightshade_ui::commands::builder::UiBuild,
) {
    let mut tree = UiTreeBuilder::new(build.context());
    let (handles, found, dots) = hud::build(&mut tree);
    tree.finish();
    resources.hud = handles;
    resources.dots = dots;
    *banner = found;
}

fn pack_sheets(resources: &mut CoinfallResources, atlases: &mut SpriteAtlases) {
    sprite_atlas_create(
        atlases,
        theme::ATLAS,
        theme::ATLAS_SIZE,
        SpriteFilter::Nearest,
    );
    resources.sheets.tiles = insert_picture(atlases, theme::TILE_SHEET, theme::TILE_BYTES);
    resources.sheets.characters =
        insert_picture(atlases, theme::CHARACTER_SHEET, theme::CHARACTER_BYTES);
    resources.sheets.backgrounds =
        insert_picture(atlases, theme::BACKGROUND_SHEET, theme::BACKGROUND_BYTES);
    resources.sheets.blank = sprite_atlas_insert(
        atlases,
        theme::ATLAS,
        theme::BLANK,
        &[255, 255, 255, 255],
        1,
        1,
    )
    .expect("the blank texel fits the atlas");
}

fn insert_picture(atlases: &mut SpriteAtlases, name: &str, bytes: &[u8]) -> SpriteRegion {
    let decoded = decode_to_rgba8(bytes).expect("the packed sheet decodes");
    let (width, height) = (decoded.width, decoded.height);
    sprite_atlas_insert(atlases, theme::ATLAS, name, &decoded.rgba, width, height)
        .expect("the packed sheet fits the atlas")
}

fn spawn_haze(blank: SpriteRegion, commands: &mut Commands) {
    let sprite = Sprite::new(theme::ATLAS, blank)
        .with_size(theme::HAZE_SPAN, theme::HAZE_SPAN)
        .with_color(theme::SKY)
        .with_layer(theme::LAYER_BACKDROP);
    spawn_sprite(commands, sprite, Vec3::new(0.0, 0.0, theme::BACKDROP_DEPTH));
}
