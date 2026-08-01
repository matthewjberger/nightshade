use crate::components::{Generation, OfferIndex};
use crate::contract::Screen;
use crate::data::shop::Offer;
use crate::queries::table::card_depth;
use crate::resources::GatewardenResources;
use crate::systems::table::{card_stats, spawn_hand_node, spawn_relic_node};
use nightshade::prelude::*;
use nightshade_ui::prelude::*;

pub fn sync(
    mut resources: ResMut<GatewardenResources>,
    mut build: nightshade_ui::commands::builder::UiBuild,
    mut offers: Query<(&OfferIndex, &Generation)>,
) {
    let resources = &mut *resources;
    if resources.screen != Screen::Shop {
        return;
    }
    if resources.table.shop_generation == resources.run.shop_generation {
        return;
    }
    let generation = resources.run.shop_generation;
    resources.table.shop_generation = generation;
    let mut old = Vec::new();
    offers.for_each_mut(|entity, (_, offer_generation)| {
        if offer_generation.0 != generation {
            old.push(entity);
        }
    });
    if !old.is_empty() {
        let commands = build.context().commands;
        for entity in &old {
            commands.remove::<(OfferIndex, Generation)>(*entity);
        }
        resources.table.retired.extend(old);
    }
    let frames = resources.ui.frames;
    for (index, offer) in resources.run.offers.iter().enumerate() {
        let (Some(offer), Some(area)) = (offer, resources.ui.offer_areas[index]) else {
            continue;
        };
        let mut tree = UiTreeBuilder::from_parent(build.context(), area);
        let root = match offer {
            Offer::Card(card) => {
                spawn_hand_node(
                    &mut tree,
                    frames,
                    *card,
                    vec2(0.0, 0.0),
                    card_stats(*card),
                    card_depth(index as f32),
                    false,
                )
                .root
            }
            Offer::Forge(card) | Offer::Purge(card) => {
                spawn_hand_node(
                    &mut tree,
                    frames,
                    *card,
                    vec2(0.0, 0.0),
                    card_stats(*card),
                    card_depth(index as f32),
                    matches!(offer, Offer::Forge(_)),
                )
                .root
            }
            Offer::Relic(relic) => spawn_relic_node(&mut tree, *relic),
        };
        let commands = tree.commands();
        commands.set(root, OfferIndex(index));
        commands.set(root, Generation(generation));
        tree.finish();
    }
}
