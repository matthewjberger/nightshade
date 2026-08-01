use crate::resources::{Chests, Inventory, OpenChest};
use blocker_screen::data::fresh::NewWorld;
use nightshade::ecs::system_param::EventReader;
use nightshade::prelude::*;

pub fn empty_the_pockets(
    fresh: EventReader<NewWorld>,
    mut inventory: ResMut<Inventory>,
    mut chests: ResMut<Chests>,
    mut open: ResMut<OpenChest>,
) {
    if fresh.into_iter().next().is_none() {
        return;
    }
    *inventory = Inventory::default();
    *chests = Chests::default();
    *open = OpenChest::default();
}
