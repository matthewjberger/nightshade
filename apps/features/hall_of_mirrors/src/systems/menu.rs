use crate::data;
use crate::resources::{MenuHandles, PortalsResources, Screen};
use nightshade::prelude::*;
use nightshade_platform::commands::window::set_pointer_grab;
use nightshade_platform::prelude::*;
use nightshade_state::prelude::*;
use nightshade_ui::prelude::*;
use nightshade_ui::resources::RetainedUiFrame;
pub use nightshade_ui_menu::data::dark_menu_style as menu_style;
use nightshade_ui_menu::prelude::*;

pub fn build(
    tree: &mut UiTreeBuilder,
    style: &MenuStyle,
    menus: &mut Menus<Screen>,
) -> MenuHandles {
    let menu = add_menu(
        tree,
        style,
        data::MENU_WIDTH,
        data::MENU_HEIGHT,
        "Paused",
        &["Escape to go back"],
        &["Resume", "Quit"],
    );
    menu_show_on(menus, Screen::Paused, menu.panel);
    MenuHandles {
        resume: menu.buttons[0],
        quit: menu.buttons[1],
    }
}

pub fn update(
    input: Res<Input>,
    resources: Res<PortalsResources>,
    screen: Res<State<Screen>>,
    mut next: ResMut<NextState<Screen>>,
    frame: Res<RetainedUiFrame>,
    mut window: ResMut<Window>,
) {
    let escape = input.keyboard.just_pressed(KeyCode::Escape);
    let clicked: Vec<Entity> = ui_button_clicks(&frame).collect();
    match screen.current {
        Screen::Playing => {
            if escape {
                next.pending = Some(Screen::Paused);
            }
        }
        Screen::Paused => {
            if escape || clicked.contains(&resources.menu.resume) {
                next.pending = Some(Screen::Playing);
            } else if clicked.contains(&resources.menu.quit) {
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
