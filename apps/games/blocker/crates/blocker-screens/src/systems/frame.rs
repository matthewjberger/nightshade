use crate::resources::ShelfPanels;
use crate::systems::screens::chest::refresh_chest;
use crate::systems::screens::crafting::refresh_crafting;
use crate::systems::screens::pocket::refresh_pocket;
use blocker_pocket::resources::{Chests, Inventory, OpenChest};
use blocker_screen::data::screen::Screen;
use nightshade::prelude::*;
use nightshade_state::prelude::*;
use nightshade_ui::resources::RetainedUiDirty;

pub fn refresh_panels(
    shelf: ShelfPanels,
    state: Res<State<Screen>>,
    inventory: Res<Inventory>,
    chests: Res<Chests>,
    open: Res<OpenChest>,
    mut dirty: ResMut<RetainedUiDirty>,
    mut commands: Commands,
) {
    if shelf.pocket.panel == Entity::default() {
        return;
    }
    match state.current {
        Screen::Inventory => {
            refresh_pocket(
                &inventory,
                shelf.layers,
                shelf.pocket,
                &mut dirty,
                &mut commands,
            );
        }
        Screen::Crafting => {
            refresh_crafting(&inventory, shelf.crafting, &mut commands);
        }
        Screen::Chest => {
            refresh_chest(
                &inventory,
                &chests,
                &open,
                shelf.layers,
                shelf.chest,
                &mut dirty,
                &mut commands,
            );
        }
        Screen::Playing
        | Screen::Settings
        | Screen::Title
        | Screen::NewWorld
        | Screen::Loading
        | Screen::Setup
        | Screen::Map
        | Screen::Character
        | Screen::Journal
        | Screen::Dialogue
        | Screen::Shop => {}
    }
}
