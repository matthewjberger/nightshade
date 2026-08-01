use nightshade::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_state::prelude::*;
use nightshade_ui::prelude::*;
use nightshade_ui::resources::RetainedUiFrame;
use nightshade_ui_menu::prelude::*;

use crate::resources::{LocomotionResources, Screen, ScreenHandles};

const PANEL_WIDTH: f32 = 340.0;
const TITLE_HEIGHT: f32 = 208.0;
const PAUSE_HEIGHT: f32 = 184.0;

pub fn menu_style() -> MenuStyle {
    MenuStyle {
        centered: true,
        ..Default::default()
    }
}

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
        "Locomotion",
        &[
            "Left stick to move, right stick to look.",
            "Hold the left trigger to face the camera.",
        ],
        &["Play", "Quit"],
    );
    let pause = add_menu(
        tree,
        style,
        PANEL_WIDTH,
        PAUSE_HEIGHT,
        "Paused",
        &["Start or escape to go back."],
        &["Resume", "Quit"],
    );

    menu_show_on(menus, Screen::Title, title.panel);
    menu_show_on(menus, Screen::Paused, pause.panel);

    ScreenHandles {
        play_button: title.buttons[0],
        title_quit_button: title.buttons[1],
        resume_button: pause.buttons[0],
        pause_quit_button: pause.buttons[1],
    }
}

pub fn watch_the_keys(
    input: Res<Input>,
    gamepad: Res<nightshade_gamepad::contract::Gamepad>,
    screen: Res<State<Screen>>,
    mut next: ResMut<NextState<Screen>>,
) {
    let start = nightshade_gamepad::query_active_gamepad(&gamepad, |pad| {
        pad.just_pressed(nightshade_gamepad::GamepadButton::Start)
    })
    .unwrap_or(false);
    if !input.keyboard.just_pressed(KeyCode::Escape) && !start {
        return;
    }
    match screen.current {
        Screen::Playing => next.pending = Some(Screen::Paused),
        Screen::Paused => next.pending = Some(Screen::Playing),
        Screen::Title => next.pending = Some(Screen::Playing),
    }
}

pub fn drive_the_menus(
    resources: Res<LocomotionResources>,
    frame: Res<RetainedUiFrame>,
    mut next: ResMut<NextState<Screen>>,
    mut window: ResMut<Window>,
) {
    let handles = resources.screens;
    for entity in ui_button_clicks(&frame) {
        if entity == handles.play_button || entity == handles.resume_button {
            next.pending = Some(Screen::Playing);
        } else if entity == handles.title_quit_button || entity == handles.pause_quit_button {
            window.should_exit = true;
        }
    }
}
