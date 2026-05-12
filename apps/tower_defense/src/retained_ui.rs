mod hud;
mod theme;

use crate::ecs::GameWorld;
use nightshade::prelude::*;

pub fn build(game_world: &mut GameWorld, world: &mut World) {
    let mut tree = UiTreeBuilder::new(world);
    let handles = hud::build(&mut tree, game_world.resources.selected_tower_type);
    tree.finish();
    game_world.resources.ui_handles = handles;
}

pub fn poll(game_world: &mut GameWorld, world: &mut World) {
    hud::poll(game_world, world);
}

pub fn update(game_world: &mut GameWorld, world: &mut World) {
    hud::update(game_world, world);
}
