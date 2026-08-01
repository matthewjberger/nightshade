use crate::data::ask::Ask;
use crate::resources::Cursor;
use blocker_screen::data::screen::Screen;
use nightshade::prelude::*;
use nightshade_input::Actions;
use nightshade_platform::prelude::*;
use nightshade_state::prelude::*;

pub fn keep_the_cursor(
    actions: Res<Actions<Ask>>,
    state: Res<State<Screen>>,
    capture: Res<InputCapture>,
    mut cursor: ResMut<Cursor>,
    mut window: ResMut<Window>,
) {
    let playing = state.current == Screen::Playing;
    let returned = playing && !cursor.playing;
    cursor.playing = playing;
    let typing = keyboard_captured(&capture);
    let free =
        !playing || typing || actions.just_pressed(Ask::Unlock) || actions.pressed(Ask::Cursor);
    let take = actions.just_pressed(Ask::Lock) || actions.just_released(Ask::Cursor) || returned;
    let locked = window.cursor_locked;
    let wanted = if free {
        false
    } else if take && window.is_focused {
        true
    } else {
        locked
    };
    if locked != wanted {
        set_cursor_locked(&mut window, wanted);
    }
    if cursor.shown == wanted {
        set_cursor_visible(&mut window, !wanted);
        cursor.shown = !wanted;
    }
}
