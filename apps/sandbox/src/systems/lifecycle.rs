use crate::ecs::{SandboxResources, Screen, register_sandbox_components};
use crate::systems::screens::{hud, pause, title};
use crate::systems::world::{
    character, enemies, grass, infinite, level, player, rocks, terrain, textures,
};
use nightshade::prelude::*;

pub fn initialize(sandbox_world: &mut SandboxResources, world: &mut World) {
    world.ecs.add_world_at(GAME, register_sandbox_components());

    world.expect_resource_mut::<RenderSettings>().atmosphere = Atmosphere::DayNight;
    world.expect_resource_mut::<DebugDraw>().show_grid = false;
    world.resources.renderer_state.use_fullscreen = true;
    world.resources.renderer_state.day_night.hour = 9.5;
    world.resources.renderer_state.day_night.auto_cycle = false;

    textures::load_prototype_textures(world);
    level::load_or_build_level(world);
    player::spawn(sandbox_world, world);
    character::spawn(sandbox_world, world);
    character::set_third_person(sandbox_world, world, true);
    enemies::spawn(world);
    infinite::set_enabled(sandbox_world, world, true);
    grass::set_enabled(sandbox_world, world, true);
    terrain::set_enabled(sandbox_world, world, true);
    rocks::initialize(sandbox_world, world);
    sandbox_world.screen.show_fps = true;

    build_ui(sandbox_world, world);
}

/// Builds the retained UI tree and stores the handles. Runs at initialize
/// and again after a snapshot restore, which recreates the UI member
/// world empty.
pub fn build_ui(sandbox_world: &mut SandboxResources, world: &mut World) {
    let mut tree = UiTreeBuilder::new(world);
    let title_handles = title::build(&mut tree);
    let pause_handles = pause::build(&mut tree);
    let hud_handles = hud::build(&mut tree);
    tree.finish();
    sandbox_world.ui_handles.title = title_handles;
    sandbox_world.ui_handles.pause = pause_handles;
    sandbox_world.ui_handles.hud = hud_handles;
}

/// Applies one screen's configuration, so the physics/cursor/gamepad-nav
/// booleans and the initial focused button all live in a single place.
/// Registered as the `on_enter` hook for every [`Screen`], and called
/// directly after a snapshot restore re-applies the current screen.
pub fn enter(sandbox_world: &mut SandboxResources, world: &mut World, screen: Screen) {
    let config = screen_config(&sandbox_world.ui_handles, screen);

    apply_visibility(sandbox_world, world, screen);

    world.plugin_resource_mut::<PhysicsWorld>().enabled = config.physics_enabled;
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

pub fn apply_visibility(sandbox_world: &SandboxResources, world: &mut World, screen: Screen) {
    let handles = &sandbox_world.ui_handles;
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
        matches!(screen, Screen::InGame) && !sandbox_world.character.third_person,
    );
}
