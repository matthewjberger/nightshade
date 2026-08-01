use blocker_pocket::resources::OpenChest;
use blocker_screen::data::screen::Screen;
use nightshade_state::prelude::NextState;

pub fn leave_the_chest(open: &mut OpenChest, next: &mut NextState<Screen>) {
    open.0 = None;
    next.pending = Some(Screen::Playing);
}
