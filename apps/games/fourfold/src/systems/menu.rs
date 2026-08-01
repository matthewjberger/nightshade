use crate::commands as game;
use crate::contract::Dimension;
use crate::data;
use crate::data::Ask;
use crate::resources::{FourfoldResources, MenuButtons, Screen};
use nightshade::prelude::*;
use nightshade_input::Actions;
use nightshade_platform::commands::window::set_pointer_grab;
use nightshade_platform::prelude::*;
use nightshade_state::prelude::*;
use nightshade_ui::prelude::*;
use nightshade_ui::resources::RetainedUiFrame;
use nightshade_ui_menu::prelude::*;

pub fn menu_style() -> MenuStyle {
    MenuStyle {
        background: Vec4::new(0.03, 0.03, 0.05, 0.94),
        border: Vec4::new(1.0, 1.0, 1.0, 0.14),
        corner_radius: 8.0,
        border_width: 1.0,
        padding: 16.0,
        spacing: 10.0,
        heading_color: Vec4::new(0.90, 0.92, 0.98, 1.0),
        row_size: 15.0,
        row_color: Vec4::new(0.64, 0.70, 0.80, 1.0),
        visible: true,
        ..Default::default()
    }
}

pub fn build(
    tree: &mut UiTreeBuilder,
    style: &MenuStyle,
    menus: &mut Menus<Screen>,
) -> MenuButtons {
    let title = add_menu(
        tree,
        style,
        data::MENU_WIDTH,
        data::TITLE_HEIGHT,
        data::TITLE,
        &[
            "Four dimensions. Each one plays as a different game.",
            "Nothing is locked by an item, only by what you work out.",
            "Mouse looks, WASD walks, Space jumps, Escape pauses.",
        ],
        &["Enter the rotunda", "Quit"],
    );
    let pause = add_menu(
        tree,
        style,
        data::MENU_WIDTH,
        data::PAUSE_HEIGHT,
        "Paused",
        &["Escape goes back."],
        &["Resume", "Back to the title", "Quit"],
    );

    menu_show_on(menus, Screen::Title, title.panel);
    menu_show_on(menus, Screen::Paused, pause.panel);

    MenuButtons {
        play: title.buttons[0],
        title_quit: title.buttons[1],
        resume: pause.buttons[0],
        to_title: pause.buttons[1],
        pause_quit: pause.buttons[2],
    }
}

pub fn update(
    actions: Res<Actions<Ask>>,
    frame: Res<RetainedUiFrame>,
    screen: Res<State<Screen>>,
    mut resources: ResMut<FourfoldResources>,
    mut next_screen: ResMut<NextState<Screen>>,
    mut next_dimension: ResMut<NextState<Dimension>>,
    mut window: ResMut<Window>,
) {
    let escape = actions.just_pressed(Ask::Pause);
    let clicked: Vec<Entity> = ui_button_clicks(&frame).collect();
    let buttons = resources.menu;

    match screen.current {
        Screen::Title => {
            if clicked.contains(&buttons.play) {
                next_screen.pending = Some(Screen::Playing);
            } else if clicked.contains(&buttons.title_quit) {
                window.should_exit = true;
            }
        }
        Screen::Playing => {
            if escape {
                next_screen.pending = Some(Screen::Paused);
            }
        }
        Screen::Paused => {
            if escape || clicked.contains(&buttons.resume) {
                next_screen.pending = Some(Screen::Playing);
            } else if clicked.contains(&buttons.to_title) {
                game::restart_hub(&mut resources.walker);
                next_dimension.pending = Some(Dimension::Hub);
                next_screen.pending = Some(Screen::Title);
            } else if clicked.contains(&buttons.pause_quit) {
                window.should_exit = true;
            }
        }
    }
}

pub fn hold_the_cursor(mut window: ResMut<Window>) {
    set_pointer_grab(&mut window, true);
}

pub fn release_the_cursor(mut window: ResMut<Window>) {
    set_pointer_grab(&mut window, false);
}

pub fn regrab(actions: Res<Actions<Ask>>, mut window: ResMut<Window>) {
    if window.cursor_locked {
        return;
    }
    if actions.just_pressed(Ask::Grab) {
        set_pointer_grab(&mut window, true);
    }
}
