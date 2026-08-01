use crate::components::Spin;
use crate::resources::Screen;
use crate::resources::TemplateResources;
use crate::systems::{hud, screens};
use nightshade::prelude::*;
use nightshade_camera::prelude::*;
use nightshade_camera_controller::commands::spawn_pan_orbit_camera_at;
use nightshade_graphics::prelude::*;
use nightshade_render::resources::viewport::{Viewport, ViewportExtent};
use nightshade_ui_menu::prelude::{MenuStyle, Menus};

pub fn initialize(
    mut debug_draw: ResMut<DebugDraw>,
    mut render_settings: ResMut<RenderSettings>,
    mut active_camera: ResMut<ActiveCamera>,
    mut viewport: ResMut<Viewport>,
    mut resources: ResMut<TemplateResources>,
    style: Res<MenuStyle>,
    mut menus: ResMut<Menus<Screen>>,
    mut build: nightshade_ui::commands::builder::UiBuild,
    mut commands: Commands,
) {
    debug_draw.show_grid = true;
    render_settings.atmosphere = Atmosphere::Nebula;

    viewport.extent = ViewportExtent::Unobstructed;

    spawn_sun(&mut commands);

    let camera_entity = spawn_pan_orbit_camera_at(
        &mut commands,
        Vec3::new(0.0, 0.5, 0.0),
        6.0,
        0.6,
        -0.45,
        "Pan Orbit Camera".to_string(),
    );
    active_camera.0 = Some(camera_entity);

    let cube = spawn_cube_at(&mut commands, Vec3::new(0.0, 0.5, 0.0));
    commands.set(cube, Spin { speed: 1.0 });

    let mut tree = nightshade_ui::prelude::UiTreeBuilder::new(build.context());
    resources.hud = hud::build(&mut tree);
    resources.screens = screens::build(&mut tree, &style, &mut menus);
    tree.finish();
    resources.menu_open = false;
}
