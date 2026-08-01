use crate::components::PhaseTinted;
use crate::data::PhaseTurned;
use crate::queries::{look, turning};
use crate::resources::{Phase, PhaseSettings};
use nightshade::ecs::system_param::EventWriter;
use nightshade::prelude::*;
use nightshade_render::components::Light;
use nightshade_render::data::config::{Fog, FogMode, RenderSettings};

pub fn settle_the_phase(
    mut phase: ResMut<Phase>,
    settings: Res<PhaseSettings>,
    mut render: ResMut<RenderSettings>,
    mut turned: EventWriter<PhaseTurned>,
    lamps: Query<(&PhaseTinted, &mut Light)>,
) {
    if !turning(&phase) {
        return;
    }
    let Some(wanted) = look(&settings, phase.at) else {
        return;
    };
    phase.applied = Some(phase.at);
    render.fog = Some(Fog {
        color: wanted.fog_color,
        start: wanted.fog_start,
        end: wanted.fog_end,
        mode: FogMode::Linear,
        ..Fog::default()
    });
    render.color_grading.saturation = wanted.saturation;
    render.color_grading.contrast = wanted.contrast;
    lamps.for_each(|_entity, (tinted, light)| {
        light.color = wanted.light_color * tinted.0;
    });
    turned.send(PhaseTurned { at: phase.at });
}
