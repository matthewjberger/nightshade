use crate::contract::PortalSide;
use crate::data;
use crate::data::Ask;
use crate::queries;
use crate::resources::PortalsResources;
use nightshade::prelude::*;
use nightshade_input::Actions;
use nightshade_portal::contract::PortalPlane;
use nightshade_portal::prelude::Portal;
use nightshade_portal::queries::surface_up;

pub fn fire(actions: Res<Actions<Ask>>, resources: Res<PortalsResources>, mut commands: Commands) {
    if resources.player.grabbed_now {
        return;
    }

    let side = if actions.just_pressed(Ask::FireBlue) {
        PortalSide::Blue
    } else if actions.just_pressed(Ask::FireOrange) {
        PortalSide::Orange
    } else {
        return;
    };

    let surface = resources.surfaces[side.index()];
    if surface == Entity::default() {
        return;
    }

    let origin = queries::eye_position(&resources.player);
    let direction = queries::player_forward(&resources.player);
    let Some(hit) = queries::cast(origin, direction, &resources.solids) else {
        return;
    };

    commands.set(
        surface,
        Portal(PortalPlane {
            center: hit.point + hit.normal * data::PORTAL_SURFACE_OFFSET,
            normal: hit.normal,
            up: surface_up(hit.normal),
            width: data::PORTAL_WIDTH,
            height: data::PORTAL_HEIGHT,
        }),
    );
    commands.set(surface, side);
}
