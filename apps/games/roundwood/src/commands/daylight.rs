use crate::data;
use nightshade_graphics::prelude::*;
use nightshade_render::data::config::{Fog, FogMode};

pub fn start_the_day(settings: &mut RenderSettings) {
    settings.atmosphere = Atmosphere::Sunset;
    settings.bloom_enabled = true;
    settings.bloom_intensity = data::BLOOM_INTENSITY;
    settings.bloom_threshold = 1.25;
    settings.bloom_knee = 0.5;
    settings.fog = Some(Fog {
        color: data::CLEAR_FOG,
        start: data::FOG_START,
        end: data::FOG_END,
        mode: FogMode::Linear,
        ..Default::default()
    });
}
