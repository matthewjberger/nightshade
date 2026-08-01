use crate::commands::clear_run;
use crate::components::{Hook, LineLength, Speed};
use crate::data::Ask;
use crate::resources::{Screen, ScreenHandles, TetherResources};
use crate::systems::motion::return_to_start;
use crate::theme::{PAUSE_HEIGHT, PAUSE_WIDTH, TITLE_HEIGHT, TITLE_WIDTH};
use nightshade::prelude::*;
use nightshade_input::Actions;
use nightshade_physics::resources::PhysicsWorld;
use nightshade_platform::prelude::*;
use nightshade_state::prelude::*;
use nightshade_ui::prelude::*;
use nightshade_ui::resources::{
    RetainedUiDirty, RetainedUiFrame, RetainedUiGamepadNav, RetainedUiInteraction,
    RetainedUiOverlays,
};
use nightshade_ui_menu::prelude::*;

pub fn build(
    tree: &mut UiTreeBuilder,
    style: &MenuStyle,
    menus: &mut Menus<Screen>,
) -> ScreenHandles {
    let title = add_menu(
        tree,
        style,
        TITLE_WIDTH,
        TITLE_HEIGHT,
        "Tether",
        &[
            "A canyon crossed on one line.",
            "Swing late and keep it all.",
        ],
        &["Play", "Quit"],
    );
    let pause = add_menu(
        tree,
        style,
        PAUSE_WIDTH,
        PAUSE_HEIGHT,
        "Paused",
        &[],
        &["Resume", "Restart", "Quit to title", "Quit"],
    );

    menu_show_on(menus, Screen::Title, title.panel);
    menu_show_on(menus, Screen::Paused, pause.panel);

    ScreenHandles {
        play_button: title.buttons[0],
        title_quit_button: title.buttons[1],
        resume_button: pause.buttons[0],
        restart_button: pause.buttons[1],
        pause_title_button: pause.buttons[2],
        pause_quit_button: pause.buttons[3],
    }
}

pub fn watch_the_keys(
    actions: Res<Actions<Ask>>,
    screen: Res<State<Screen>>,
    mut next: ResMut<NextState<Screen>>,
) {
    let toggled = actions.just_pressed(Ask::Pause);
    let backed = actions.just_pressed(Ask::Back);

    match screen.current {
        Screen::Playing if toggled => next.pending = Some(Screen::Paused),
        Screen::Paused if toggled || backed => next.pending = Some(Screen::Playing),
        _ => {}
    }
}

pub fn drive_the_menus(
    frame: Res<RetainedUiFrame>,
    mut physics: ResMut<PhysicsWorld>,
    mut resources: ResMut<TetherResources>,
    mut next: ResMut<NextState<Screen>>,
    mut window: ResMut<Window>,
    mut commands: Commands,
    mut runners: Query<(&mut Speed,)>,
) {
    let handles = resources.screens;
    for entity in ui_button_clicks(&frame) {
        if entity == handles.play_button || entity == handles.restart_button {
            if let Some(runner) = resources.scene.runner {
                return_to_start(&mut physics, &mut commands, &mut runners, runner);
            }
            clear_run(&mut resources.run);
            next.pending = Some(Screen::Playing);
        } else if entity == handles.resume_button {
            next.pending = Some(Screen::Playing);
        } else if entity == handles.pause_title_button {
            next.pending = Some(Screen::Title);
        } else if entity == handles.title_quit_button || entity == handles.pause_quit_button {
            window.should_exit = true;
        }
    }
}

pub fn start_the_world(
    mut physics: ResMut<PhysicsWorld>,
    mut navigation: ResMut<RetainedUiGamepadNav>,
    mut interaction: ResMut<RetainedUiInteraction>,
    mut overlays: ResMut<RetainedUiOverlays>,
    mut window: ResMut<Window>,
) {
    physics.enabled = true;
    navigation.enabled = false;
    interaction.focused_entity = None;
    overlays.focus_ring_visible = false;
    set_cursor_locked(&mut window, true);
    set_cursor_visible(&mut window, false);
}

pub fn stop_the_world(
    mut physics: ResMut<PhysicsWorld>,
    mut navigation: ResMut<RetainedUiGamepadNav>,
    resources: Res<TetherResources>,
    mut window: ResMut<Window>,
    mut commands: Commands,
) {
    physics.enabled = false;
    navigation.enabled = true;
    if let Some(runner) = resources.scene.runner {
        commands.remove::<(Hook, LineLength)>(runner);
    }
    set_cursor_locked(&mut window, false);
    set_cursor_visible(&mut window, true);
}

pub fn show_hud(
    resources: Res<TetherResources>,
    mut dirty: ResMut<RetainedUiDirty>,
    mut commands: Commands,
) {
    show(&mut dirty, &mut commands, resources.ui.root, true);
}

pub fn hide_hud(
    resources: Res<TetherResources>,
    mut dirty: ResMut<RetainedUiDirty>,
    mut commands: Commands,
) {
    show(&mut dirty, &mut commands, resources.ui.root, false);
}

fn show(dirty: &mut RetainedUiDirty, commands: &mut Commands, node: Option<Entity>, visible: bool) {
    let Some(node) = node else {
        return;
    };
    ui_set_visible(dirty, commands, node, visible);
}
