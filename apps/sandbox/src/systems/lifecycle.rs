use crate::state::{Action, Sandbox, Screen};
use crate::systems::{hud, level, pause, player, textures, title};
use nightshade::prelude::*;

pub fn initialize(sandbox: &mut Sandbox, world: &mut World) {
    world.resources.user_interface.enabled = true;
    world.resources.retained_ui.enabled = true;
    world.resources.graphics.atmosphere = Atmosphere::DayNight;
    world.resources.graphics.show_grid = false;
    world.resources.graphics.use_fullscreen = true;
    world.resources.graphics.day_night.hour = 9.5;
    world.resources.graphics.day_night.auto_cycle = false;
    capture_procedural_atmosphere_ibl(world, Atmosphere::DayNight, 9.5);

    textures::load_prototype_textures(world);
    level::load_or_build_level(world);
    player::spawn(sandbox, world);

    let mut tree = UiTreeBuilder::new(world);
    sandbox.title = title::build(&mut tree);
    sandbox.pause = pause::build(&mut tree);
    sandbox.hud = hud::build(&mut tree);
    tree.finish();

    enter_title(sandbox, world);
}

pub fn drain_actions(sandbox: &mut Sandbox, world: &mut World) {
    let actions = std::mem::take(&mut sandbox.pending_actions);
    for action in actions {
        apply_action(sandbox, world, action);
    }
}

pub fn apply_action(sandbox: &mut Sandbox, world: &mut World, action: Action) {
    match action {
        Action::StartGame | Action::ResumeGame => enter_game(sandbox, world),
        Action::PauseGame => enter_paused(sandbox, world),
        Action::ReturnToTitle => {
            player::reset(sandbox, world);
            enter_title(sandbox, world);
        }
        Action::ResetPlayer => player::reset(sandbox, world),
        Action::Quit => world.resources.window.should_exit = true,
    }
}

pub fn enter_title(sandbox: &mut Sandbox, world: &mut World) {
    sandbox.screen = Screen::Title;
    apply_visibility(sandbox, world);
    world.resources.physics.enabled = false;
    set_cursor_locked(world, false);
    set_cursor_visible(world, true);
    world.resources.retained_ui.gamepad_nav.enabled = true;
}

pub fn enter_game(sandbox: &mut Sandbox, world: &mut World) {
    sandbox.screen = Screen::InGame;
    apply_visibility(sandbox, world);
    world.resources.physics.enabled = true;
    set_cursor_locked(world, true);
    set_cursor_visible(world, false);
    world.resources.retained_ui.gamepad_nav.enabled = false;
    world.resources.retained_ui.gamepad_nav.held_direction = None;
    world
        .resources
        .retained_ui
        .interaction_for_active_mut()
        .focused_entity = None;
    world.resources.retained_ui.overlays.focus_ring_visible = false;
}

pub fn enter_paused(sandbox: &mut Sandbox, world: &mut World) {
    sandbox.screen = Screen::Paused;
    apply_visibility(sandbox, world);
    world.resources.physics.enabled = false;
    set_cursor_locked(world, false);
    set_cursor_visible(world, true);
    world.resources.retained_ui.gamepad_nav.enabled = true;
}

pub fn apply_visibility(sandbox: &Sandbox, world: &mut World) {
    ui_set_visible(
        world,
        sandbox.title.root,
        matches!(sandbox.screen, Screen::Title),
    );
    ui_set_visible(
        world,
        sandbox.pause.root,
        matches!(sandbox.screen, Screen::Paused),
    );
    ui_set_visible(
        world,
        sandbox.hud.root,
        matches!(sandbox.screen, Screen::InGame | Screen::Paused),
    );
}
