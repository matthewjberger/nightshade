use crate::data::Ask;
use crate::resources::{Screen, ScreenHandles, TemplateResources};
use nightshade::prelude::*;
use nightshade_input::Actions;
use nightshade_platform::prelude::*;
use nightshade_state::prelude::*;
use nightshade_ui::prelude::*;
use nightshade_ui::resources::{RetainedUiDirty, RetainedUiFrame};
use nightshade_ui_menu::prelude::*;

const PANEL_WIDTH: f32 = 320.0;
const TITLE_HEIGHT: f32 = 200.0;
const PAUSE_HEIGHT: f32 = 176.0;

pub fn build(
    tree: &mut UiTreeBuilder,
    style: &MenuStyle,
    menus: &mut Menus<Screen>,
) -> ScreenHandles {
    let title = add_menu(
        tree,
        style,
        PANEL_WIDTH,
        TITLE_HEIGHT,
        "Template",
        &["A skeleton with screens in it."],
        &["Play", "Quit"],
    );
    let pause = add_menu(
        tree,
        style,
        PANEL_WIDTH,
        PAUSE_HEIGHT,
        "Paused",
        &["Escape to go back."],
        &["Resume", "Quit to title"],
    );

    menu_show_on(menus, Screen::Title, title.panel);
    menu_show_on(menus, Screen::Paused, pause.panel);

    ScreenHandles {
        play_button: title.buttons[0],
        title_quit_button: title.buttons[1],
        resume_button: pause.buttons[0],
        pause_title_button: pause.buttons[1],
    }
}

pub fn watch_the_keys(
    actions: Res<Actions<Ask>>,
    screen: Res<State<Screen>>,
    mut next: ResMut<NextState<Screen>>,
) {
    if !actions.just_pressed(Ask::Pause) {
        return;
    }
    match screen.current {
        Screen::Playing => next.pending = Some(Screen::Paused),
        Screen::Paused => next.pending = Some(Screen::Playing),
        Screen::Title => {}
    }
}

pub fn drive_the_menus(
    resources: Res<TemplateResources>,
    frame: Res<RetainedUiFrame>,
    mut next: ResMut<NextState<Screen>>,
    mut window: ResMut<Window>,
) {
    let handles = resources.screens;
    for entity in ui_button_clicks(&frame) {
        if entity == handles.play_button || entity == handles.resume_button {
            next.pending = Some(Screen::Playing);
        } else if entity == handles.pause_title_button {
            next.pending = Some(Screen::Title);
        } else if entity == handles.title_quit_button {
            window.should_exit = true;
        }
    }
}

pub fn show_hud(
    resources: Res<TemplateResources>,
    mut dirty: ResMut<RetainedUiDirty>,
    mut commands: Commands,
) {
    show(&mut dirty, &mut commands, resources.hud.root, true);
}

pub fn hide_hud(
    resources: Res<TemplateResources>,
    mut dirty: ResMut<RetainedUiDirty>,
    mut commands: Commands,
) {
    show(&mut dirty, &mut commands, resources.hud.root, false);
}

fn show(dirty: &mut RetainedUiDirty, commands: &mut Commands, node: Entity, visible: bool) {
    if node == Entity::default() {
        return;
    }
    ui_set_visible(dirty, commands, node, visible);
}
