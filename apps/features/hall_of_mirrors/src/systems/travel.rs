use crate::commands;
use crate::contract::PortalSide;
use crate::data;
use crate::queries;
use crate::resources::PortalsResources;
use nightshade::prelude::*;
use nightshade_platform::prelude::Time;
use nightshade_portal::contract::PortalPlane;
use nightshade_portal::prelude::Portal;

pub fn update(
    time: Res<Time>,
    mut resources: ResMut<PortalsResources>,
    portals: Query<(&Portal, &PortalSide)>,
) {
    let held = (resources.player.travel_hold - time.delta_time).max(0.0);
    resources.player.travel_hold = held;
    if held > 0.0 {
        return;
    }

    let mut placements: [Option<PortalPlane>; 2] = [None; 2];
    portals.for_each(|_entity, (portal, side)| {
        placements[side.index()] = Some(portal.0);
    });
    let (Some(blue), Some(orange)) = (placements[0], placements[1]) else {
        return;
    };

    let resources = &mut *resources;
    let solids = &resources.solids;
    let player = &mut resources.player;

    for side in PortalSide::BOTH {
        let (source, destination) = match side {
            PortalSide::Blue => (blue, orange),
            PortalSide::Orange => (orange, blue),
        };
        if !queries::touching_portal(player, &source) {
            continue;
        }
        if commands::carry_through(player, &source, &destination, solids) {
            player.travel_hold = data::TRAVEL_HOLD;
        }
        return;
    }
}
