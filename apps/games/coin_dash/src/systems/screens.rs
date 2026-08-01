use crate::data::Avatar;
use crate::resources::{CoinDashResources, Screen};
use crate::systems::world::{PLAYER_SPEED, ROUND_SECONDS, generate_world};
use nightshade::prelude::*;
use nightshade_camera::commands::spawn_first_person_camera;
use nightshade_camera::prelude::*;
use nightshade_physics::commands::spawn_character_body;
use nightshade_physics::contract::components::CharacterControllerComponent;
use nightshade_platform::prelude::*;
use nightshade_ui::prelude::ui_set_visible;
use nightshade_ui::resources::RetainedUiDirty;

pub fn refresh(
    resources: &CoinDashResources,
    dirty: &mut RetainedUiDirty,
    commands: &mut Commands,
) {
    let screen = resources.round.screen;
    let handles = resources.ui;
    ui_set_visible(dirty, commands, handles.title, screen == Screen::Title);
    ui_set_visible(dirty, commands, handles.pause, screen == Screen::Paused);
    ui_set_visible(dirty, commands, handles.over, screen == Screen::Over);
    ui_set_visible(
        dirty,
        commands,
        handles.hud,
        matches!(screen, Screen::Playing | Screen::Paused),
    );
}

pub fn set_screen(
    resources: &mut CoinDashResources,
    dirty: &mut RetainedUiDirty,
    commands: &mut Commands,
    screen: Screen,
) {
    resources.round.screen = screen;
    refresh(resources, dirty, commands);
}

pub fn start_game(
    resources: &mut CoinDashResources,
    window: &mut Window,
    dirty: &mut RetainedUiDirty,
    commands: &mut Commands,
    avatars: &[Entity],
) {
    {
        let round = &mut resources.round;
        round.round = 1;
        round.score = 0;
        round.goal = 8;
        round.time = ROUND_SECONDS;
    }
    generate_world(resources, commands);

    despawn_avatar(commands, avatars);
    let player = spawn_character_body(commands, Vec3::new(0.0, 2.0, 0.0), "Player");
    spawn_first_person_camera(commands, player, Vec3::new(0.0, 1.3, 0.0), 85.0);
    commands.queue(move |storage, _| {
        if let Some(controller) = storage.get_mut::<CharacterControllerComponent>(player) {
            controller.max_speed = PLAYER_SPEED;
        }
        storage.add_tag_type::<Avatar>(player);
    });
    set_cursor_locked(window, true);

    set_screen(resources, dirty, commands, Screen::Playing);
}

pub fn leave_run(
    resources: &mut CoinDashResources,
    window: &mut Window,
    dirty: &mut RetainedUiDirty,
    commands: &mut Commands,
    avatars: &[Entity],
    menu_camera: Option<Entity>,
    screen: Screen,
) {
    despawn_avatar(commands, avatars);
    if let Some(camera) = menu_camera {
        commands.queue(move |_, resources| {
            resources.res_mut::<ActiveCamera>().0 = Some(camera);
        });
    }
    set_cursor_locked(window, false);
    set_screen(resources, dirty, commands, screen);
}

fn despawn_avatar(commands: &mut Commands, avatars: &[Entity]) {
    for &avatar in avatars {
        commands.despawn_recursive(avatar);
    }
}

pub fn set_cursor_locked(window: &mut Window, locked: bool) {
    window.cursor_locked = locked;
    window.show_cursor = !locked;
}
