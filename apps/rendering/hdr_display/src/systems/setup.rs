use nightshade::ecs::system_param::ResMut;
use nightshade_graphics::prelude::*;

pub fn settle_the_look(mut settings: ResMut<RenderSettings>) {
    settings.atmosphere = Atmosphere::Hdr;
    settings.hdr_output = true;
    settings.hdr_paper_white = 200.0;
    settings.bloom_enabled = true;
    settings.bloom_intensity = 0.5;
    settings.anti_aliasing = AntiAliasing::Taa;
    settings.ssr_enabled = true;
}
