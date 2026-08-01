use crate::components::Offset;
use crate::contract::WorldSide;
use crate::data;
use nightshade::prelude::*;
use nightshade_particles::contract::ParticleEmitter;
use nightshade_platform::prelude::Time;
use nightshade_portal::prelude::{Portal, PortalPlane};
use nightshade_portal::queries::rim_point;

pub fn update(
    time: Res<Time>,
    portals: Query<(&Portal, &WorldSide)>,
    emitters: Query<(&WorldSide, &Offset, &mut ParticleEmitter)>,
) {
    let mut placements: [Option<PortalPlane>; 2] = [None; 2];
    portals.for_each(|_entity, (portal, side)| {
        placements[side.index()] = Some(portal.0);
    });

    let turned = time.uptime_seconds as f32 * data::SPARK_TURNS;
    emitters.for_each(|_entity, (side, offset, emitter)| {
        let Some(portal) = placements[side.index()] else {
            emitter.enabled = false;
            return;
        };
        let (position, direction) = rim_point(
            &portal,
            offset.0 + turned,
            data::SPARK_STAND_OFF,
            data::SPARK_OUTWARD,
        );
        emitter.enabled = true;
        emitter.position = position;
        emitter.direction = direction;
    });
}
