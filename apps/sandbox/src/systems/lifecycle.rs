use crate::ecs::{SandboxWorld, Screen};
use crate::systems::screens::{hud, pause, title};
use crate::systems::world::{enemies, grass, level, player, textures};
use nightshade::prelude::*;

pub fn initialize(sandbox_world: &mut SandboxWorld, world: &mut World) {
    world.resources.user_interface.enabled = true;
    world.resources.retained_ui.enabled = true;
    world.resources.render_settings.atmosphere = Atmosphere::DayNight;
    world.resources.debug_draw.show_grid = false;
    world.resources.renderer_state.use_fullscreen = true;
    world.resources.renderer_state.day_night.hour = 9.5;
    world.resources.renderer_state.day_night.auto_cycle = false;
    capture_procedural_atmosphere_ibl(world, Atmosphere::DayNight, 9.5);

    textures::load_prototype_textures(world);
    level::load_or_build_level(world);
    player::spawn(sandbox_world, world);
    enemies::spawn(sandbox_world, world);
    grass::set_enabled(sandbox_world, world, true);

    let mut tree = UiTreeBuilder::new(world);
    let title_handles = title::build(&mut tree);
    let pause_handles = pause::build(&mut tree);
    let hud_handles = hud::build(&mut tree);
    tree.finish();
    sandbox_world.resources.ui_handles.title = title_handles;
    sandbox_world.resources.ui_handles.pause = pause_handles;
    sandbox_world.resources.ui_handles.hud = hud_handles;

    enter(sandbox_world, world, Screen::Title);
}

/// Drive every screen transition through one function so the
/// physics/cursor/gamepad-nav booleans and the initial focused
/// button all live in a single place. Each screen contributes a
/// small config block and the body just applies it.
pub fn enter(sandbox_world: &mut SandboxWorld, world: &mut World, screen: Screen) {
    let config = screen_config(&sandbox_world.resources.ui_handles, screen);

    sandbox_world.resources.screen.current = screen;
    apply_visibility(sandbox_world, world);

    world.resources.physics.enabled = config.physics_enabled;
    set_cursor_locked(world, config.cursor_locked);
    set_cursor_visible(world, !config.cursor_locked);
    world.resources.retained_ui.gamepad_nav.enabled = config.gamepad_nav;

    let interaction = world.resources.retained_ui.interaction_for_active_mut();
    interaction.focused_entity = config.focus;
    world.resources.retained_ui.overlays.focus_ring_visible = config.focus.is_some();
    if config.focus.is_none() {
        world.resources.retained_ui.gamepad_nav.held_direction = None;
    }
}

struct ScreenConfig {
    physics_enabled: bool,
    cursor_locked: bool,
    gamepad_nav: bool,
    focus: Option<Entity>,
}

fn screen_config(handles: &crate::ecs::UiHandles, screen: Screen) -> ScreenConfig {
    match screen {
        Screen::Title => ScreenConfig {
            physics_enabled: false,
            cursor_locked: false,
            gamepad_nav: true,
            focus: Some(handles.title.play_button),
        },
        Screen::Paused => ScreenConfig {
            physics_enabled: false,
            cursor_locked: false,
            gamepad_nav: true,
            focus: Some(handles.pause.resume_button),
        },
        Screen::InGame => ScreenConfig {
            physics_enabled: true,
            cursor_locked: true,
            gamepad_nav: false,
            focus: None,
        },
    }
}

pub fn apply_visibility(sandbox_world: &SandboxWorld, world: &mut World) {
    let handles = &sandbox_world.resources.ui_handles;
    let screen = sandbox_world.resources.screen.current;
    ui_set_visible(world, handles.title.root, matches!(screen, Screen::Title));
    ui_set_visible(world, handles.pause.root, matches!(screen, Screen::Paused));
    ui_set_visible(
        world,
        handles.hud.root,
        matches!(screen, Screen::InGame | Screen::Paused),
    );
    ui_set_visible(
        world,
        handles.hud.crosshair,
        matches!(screen, Screen::InGame),
    );
}
