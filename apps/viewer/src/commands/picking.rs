use std::collections::HashSet;

use nightshade::prelude::*;

use crate::data::ViewerAction;
use crate::queries::picking::{chain_from_root, find_root};
use crate::resources::ViewerResources;

pub fn cycle_select(
    viewer: &mut ViewerResources,
    storage: &nightshade::ecs::Storage,
    leaf: Entity,
) {
    let tracked: HashSet<Entity> = viewer.model.entities.iter().copied().collect();
    let root = find_root(storage, leaf, &tracked);
    let chain = chain_from_root(storage, root, leaf);
    if chain.is_empty() {
        return;
    }

    let same_target =
        viewer.picking.last_leaf == Some(leaf) && viewer.picking.last_root == Some(root);
    let on_chain = viewer
        .selection
        .selected
        .is_some_and(|selected| chain.contains(&selected));
    let depth = if same_target && on_chain {
        (viewer.picking.cycle_depth + 1) % chain.len()
    } else {
        0
    };

    viewer.picking.last_leaf = Some(leaf);
    viewer.picking.last_root = Some(root);
    viewer.picking.cycle_depth = depth;
    viewer
        .actions
        .push(ViewerAction::Select(Some(chain[depth])));
}

pub fn reset_cycle(viewer: &mut ViewerResources) {
    viewer.picking.last_leaf = None;
    viewer.picking.last_root = None;
    viewer.picking.cycle_depth = 0;
}
