mod hud;
mod theme;

use crate::ecs::TowerDefenseResources;
use nightshade::prelude::*;

pub fn build(game_world: &mut TowerDefenseResources, world: &mut World) {
    let mut tree = UiTreeBuilder::new(world);
    let handles = hud::build(&mut tree, game_world.selected_tower_type);
    tree.finish();
    game_world.ui_handles = handles;
}

pub fn poll(game_world: &mut TowerDefenseResources, world: &mut World) {
    hud::poll(game_world, world);
}

pub fn update(game_world: &mut TowerDefenseResources, world: &mut World) {
    hud::update(game_world, world);
}
