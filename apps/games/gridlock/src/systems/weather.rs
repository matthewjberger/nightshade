use crate::data::look::RAIN_HEIGHT;
use crate::resources::{Indoors, Player};
use nightshade::prelude::*;
use nightshade_particles::contract::ParticleEmitter;
use nightshade_scene::prelude::LocalTransform;

pub fn rain(
    player: Res<Player>,
    indoors: Res<Indoors>,
    mut anchors: Query<(&LocalTransform,)>,
    mut emitters: Query<(&mut ParticleEmitter,)>,
) {
    let (Some(body), Some(rain)) = (player.body, player.rain) else {
        return;
    };
    if indoors.visit.is_some() {
        return;
    }
    let anchor = player.car.unwrap_or(body);
    let mut above = None;
    anchors.for_entity(anchor, |(transform,)| {
        above = Some(transform.translation + Vec3::new(0.0, RAIN_HEIGHT, 0.0));
    });
    let Some(above) = above else {
        return;
    };
    emitters.for_entity(rain, |(emitter,)| {
        emitter.position = above;
    });
}
