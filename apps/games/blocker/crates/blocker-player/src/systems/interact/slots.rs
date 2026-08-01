use crate::components::Motion;
use crate::data::ask::Ask;
use blocker_figure::queries::read_the_hero;
use blocker_figure::resources::Hero;
use blocker_pocket::commands::{select_next_slot, select_slot};
use blocker_pocket::resources::Inventory;
use nightshade::prelude::*;
use nightshade_input::Actions;
use nightshade_platform::prelude::*;

pub fn pick_the_slot(
    actions: Res<Actions<Ask>>,
    window: Res<Window>,
    hero: Res<Hero>,
    mut inventory: ResMut<Inventory>,
    mut bodies: Query<(&Motion,)>,
) {
    let Some(motion) = read_the_hero(&hero, &mut bodies, |(motion,)| *motion) else {
        return;
    };
    if motion == Motion::Watch {
        return;
    }
    if window.cursor_locked {
        for (index, ask) in Ask::SLOTS.iter().enumerate() {
            if actions.just_pressed(*ask) {
                select_slot(&mut inventory, index);
            }
        }
    }
    if actions.just_pressed(Ask::PreviousSlot) {
        select_next_slot(&mut inventory, -1);
    }
    if actions.just_pressed(Ask::NextSlot) {
        select_next_slot(&mut inventory, 1);
    }
}
