use crate::components::InPhase;
use crate::data::GAIT_INTERVAL;
use crate::resources::{Gait, Model, Watch};
use nightshade::prelude::*;
use nightshade_animation::contract::AnimationPlayer;

pub fn change_gait(
    mut gait: ResMut<Gait>,
    model: Res<Model>,
    watch: Res<Watch>,
    mut players: Query<(&InPhase, &mut AnimationPlayer)>,
) {
    if !watch.frame.is_multiple_of(GAIT_INTERVAL) || watch.frame == 0 {
        return;
    }
    gait.on_run = !gait.on_run;
    let target = if gait.on_run { model.run } else { model.walk };
    let Some(index) = target else {
        return;
    };
    players.for_each_mut(|_entity, (in_phase, player)| {
        if in_phase.0 {
            player.blend_to_in_phase(index, 0.3);
        } else {
            player.blend_to(index, 0.3);
        }
    });
}
