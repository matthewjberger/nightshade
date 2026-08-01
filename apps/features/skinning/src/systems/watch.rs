use crate::components::InPhase;
use crate::data::{BLEND_FRAMES, REPORT_INTERVAL, REPORT_UNTIL};
use crate::resources::Watch;
use nightshade::prelude::*;
use nightshade_animation::contract::AnimationPlayer;
use nightshade_render::data::config::RendererState;

pub fn report(
    mut watch: ResMut<Watch>,
    renderer_state: Res<RendererState>,
    mut players: Query<(&InPhase, &AnimationPlayer)>,
) {
    watch.frame += 1;
    let through_a_blend = BLEND_FRAMES.contains(&watch.frame);
    if !through_a_blend
        && (!watch.frame.is_multiple_of(REPORT_INTERVAL) || watch.frame > REPORT_UNTIL)
    {
        return;
    }

    let mut clip_time = -1.0;
    let mut active_clips = 0;
    let mut blend_factor = -1.0;
    players.for_each_mut(|_entity, (in_phase, player)| {
        if !in_phase.0 {
            return;
        }
        clip_time = player.time;
        active_clips = player.active_clips.len();
        blend_factor = player.blend_factor;
    });

    let skinning_state = &renderer_state.render_skinning;
    let bones = skinning_state.bone_transforms.len();
    let generation = skinning_state.bone_transforms_generation;
    let palette_sum: f64 = skinning_state
        .bone_transforms
        .iter()
        .map(|matrix| matrix.iter().map(|value| *value as f64).sum::<f64>())
        .sum();

    tracing::info!(
        "frame {} clip_time {clip_time:.3} active {active_clips} blend {blend_factor:.2} palette_step {:.4} bones {bones} generation {generation}",
        watch.frame,
        palette_sum - watch.previous_palette_sum,
    );

    watch.previous_palette_sum = palette_sum;
}
