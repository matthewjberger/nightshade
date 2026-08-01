use crate::resources::{SandboxResources, Screen};
use crate::systems::screens::{hud, pause, title};
use crate::systems::world::{character, enemies, level, player, textures};
use nightshade_graphics::prelude::*;
use nightshade_physics::prelude::*;
use nightshade_ui::prelude::*;

pub fn initialize(
    sandbox_world: &mut SandboxResources,
    scene: &mut crate::commands::Scene<'_, '_>,
) {
    scene.stage.settings.atmosphere = Atmosphere::DayNight;
    scene.stage.debug_draw.show_grid = false;
    scene.stage.window.use_fullscreen = true;
    scene.stage.renderer_state.day_night.hour = 9.5;
    scene.stage.renderer_state.day_night.auto_cycle = false;

    textures::load_prototype_textures(&mut scene.texture_load());
    level::build_level(scene);
    player::spawn(sandbox_world, scene);
    character::spawn(sandbox_world, scene);
    character::set_third_person(sandbox_world, scene, true);
    enemies::spawn(scene);
    sandbox_world.screen.show_fps = true;
}

pub fn build_ui(mut sandbox_world: ResMut<SandboxResources>, mut build: UiBuild<'_>) {
    let mut tree = UiTreeBuilder::new(build.context());
    let title_handles = title::build(&mut tree);
    let pause_handles = pause::build(&mut tree);
    let hud_handles = hud::build(&mut tree);
    tree.finish();

    sandbox_world.ui_handles.title = title_handles;
    sandbox_world.ui_handles.pause = pause_handles;
    sandbox_world.ui_handles.hud = hud_handles;
}

pub fn enter(
    sandbox_world: &mut SandboxResources,
    scene: &mut crate::commands::Scene<'_, '_>,
    screen: Screen,
) {
    let config = screen_config(&sandbox_world.ui_handles, screen);

    apply_visibility(sandbox_world, scene, screen);

    scene.stage.physics.enabled = config.physics_enabled;
    let window = &mut *scene.stage.window;
    set_cursor_locked(window, config.cursor_locked);
    set_cursor_visible(window, !config.cursor_locked);
    scene.stage.gamepad_nav.enabled = config.gamepad_nav;

    let interaction = &mut *scene.stage.interaction;
    interaction.focused_entity = config.focus;
    scene.stage.overlays.focus_ring_visible = config.focus.is_some();
    if config.focus.is_none() {
        scene.stage.gamepad_nav.held_direction = None;
    }
}

struct ScreenConfig {
    physics_enabled: bool,
    cursor_locked: bool,
    gamepad_nav: bool,
    focus: Option<Entity>,
}

fn screen_config(handles: &crate::resources::UiHandles, screen: Screen) -> ScreenConfig {
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

pub fn apply_visibility(
    sandbox_world: &SandboxResources,
    scene: &mut crate::commands::Scene<'_, '_>,
    screen: Screen,
) {
    let title_root = sandbox_world.ui_handles.title.root;
    let pause_root = sandbox_world.ui_handles.pause.root;
    let hud_root = sandbox_world.ui_handles.hud.root;
    let crosshair = sandbox_world.ui_handles.hud.crosshair;
    let third_person = sandbox_world.character.third_person;
    {
        let commands = &mut *scene.commands;
        let dirty = &mut *scene.stage.dirty;
        ui_set_visible(dirty, commands, title_root, matches!(screen, Screen::Title));
        ui_set_visible(
            dirty,
            commands,
            pause_root,
            matches!(screen, Screen::Paused),
        );
        ui_set_visible(
            dirty,
            commands,
            hud_root,
            matches!(screen, Screen::InGame | Screen::Paused),
        );
        ui_set_visible(
            dirty,
            commands,
            crosshair,
            matches!(screen, Screen::InGame) && !third_person,
        );
    }
}
