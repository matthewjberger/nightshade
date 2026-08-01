use crate::commands::{load_textures, lock_cursor};
use crate::data::theme;
use crate::resources::{Arguments, BlackmarrowResources, Screen};
use crate::systems::{hud, screens};
use crate::tags::MenuCamera;
use nightshade::prelude::*;
use nightshade_camera::commands::scene::spawn_camera;
use nightshade_camera::prelude::ActiveCamera;
use nightshade_camera::queries::camera_look::camera_look_rotation;
use nightshade_graphics::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_render::data::config::{Fog, FogMode};
use nightshade_scene::prelude::LocalTransform;
use nightshade_ui_menu::prelude::{MenuStyle, Menus};

pub fn initialize(
    arguments: Res<Arguments>,
    mut debug_draw: ResMut<DebugDraw>,
    mut settings: ResMut<RenderSettings>,
    mut window: ResMut<Window>,
    mut resources: ResMut<BlackmarrowResources>,
    style: Res<MenuStyle>,
    mut menus: ResMut<Menus<Screen>>,
    mut build: nightshade_ui::commands::builder::UiBuild,
    mut textures: nightshade_graphics::resources::loading::TextureLoad,
    mut commands: Commands,
) {
    debug_draw.show_grid = false;
    dress_the_air(&mut settings);
    load_textures(&mut textures);
    raise_menu_camera(&mut commands);
    lock_cursor(&mut window, false);

    let mut tree = nightshade_ui::prelude::UiTreeBuilder::new(build.context());
    resources.hud = hud::build(&mut tree);
    resources.screens = screens::build(&mut tree, &style, &mut menus);
    tree.finish();

    resources.player.carrying_key = arguments.with_key;
    if arguments.straight_in {
        resources.pending_screen = Some(Screen::Playing);
    }
}

fn dress_the_air(settings: &mut RenderSettings) {
    settings.atmosphere = Atmosphere::None;
    settings.show_sky = false;
    settings.clear_color = [0.0, 0.0, 0.0, 1.0];
    settings.ibl_blend_factor = 0.0;
    settings.ssao_enabled = true;
    settings.ssao_intensity = 0.9;
    settings.color_grading.exposure_compensation_ev = theme::EXPOSURE_EV;
    settings.fog = Some(Fog {
        color: theme::FOG_COLOR,
        start: theme::FOG_START,
        end: theme::FOG_END,
        mode: FogMode::Exponential,
        volumetric: true,
        density: theme::FOG_DENSITY,
        anisotropy: 0.55,
        height: 0.0,
        height_falloff: 0.0,
    });
}

fn raise_menu_camera(commands: &mut Commands<'_>) {
    let camera = spawn_camera(commands, theme::MENU_CAMERA_EYE, "Menu Camera".to_string());
    commands.add_tag_type::<MenuCamera>(camera);
    commands.queue(move |storage, resources| {
        if let Some(transform) = storage.get_mut::<LocalTransform>(camera) {
            transform.rotation =
                camera_look_rotation(theme::MENU_CAMERA_EYE, theme::MENU_CAMERA_TARGET, 0.0);
        }
        resources.res_mut::<ActiveCamera>().0 = Some(camera);
    });
}
