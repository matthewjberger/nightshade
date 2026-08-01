use crate::components::{Pulse, Strength};
use crate::tags::Moon;

use crate::theme;
use nalgebra_glm::Vec3;
use nightshade::prelude::*;
use nightshade_graphics::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_render::components::light::Light;
use nightshade_render::data::config::RendererState;
use nightshade_render::prelude::{golden_factor, night_factor, smoothstep};

pub fn follow_the_hour(
    time: Res<Time>,
    renderer_state: Res<RendererState>,
    mut settings: ResMut<RenderSettings>,
    moons: Query<(&mut Light,), WithTag<Moon>>,
    lanterns: Query<(&mut Pulse, &Strength, &mut Light)>,
) {
    let hour = renderer_state.day_night.hour;
    let night = night_factor(hour, &theme::TWILIGHT);
    let golden = golden_factor(hour, &theme::TWILIGHT);
    let grading = &mut settings.color_grading;
    grading.exposure_compensation_ev = theme::NIGHT_EXPOSURE_EV * night + 0.15 * golden;
    grading.saturation = 1.0 + 0.25 * golden - 0.35 * night;
    grading.vibrance = 0.3 * golden;
    grading.contrast = 1.0 + 0.08 * night;
    grading.vignette_intensity = 0.12 + 0.3 * night;
    settings.bloom_intensity = theme::BLOOM_DAY + (theme::BLOOM_NIGHT - theme::BLOOM_DAY) * night;

    let day_fog = Vec3::new(0.74, 0.78, 0.88);
    let dusk_fog = Vec3::new(0.96, 0.62, 0.42);
    let night_fog = Vec3::new(0.05, 0.06, 0.12);
    let fog = nalgebra_glm::lerp(
        &nalgebra_glm::lerp(&day_fog, &dusk_fog, golden),
        &night_fog,
        night,
    );
    if let Some(settings_fog) = settings.fog.as_mut() {
        settings_fog.color = [fog.x, fog.y, fog.z];
    }

    moons.for_each(|_entity, (light,)| {
        light.intensity = theme::MOON_STRENGTH * night;
    });

    let delta_time = time.delta_time;
    let lit = smoothstep(night);
    lanterns.for_each(|_entity, (pulse, strength, light)| {
        pulse.0 += delta_time;
        let wave = (pulse.0 * 7.3).sin() * 0.5 + (pulse.0 * 13.1 + 1.3).sin() * 0.5;
        light.intensity = strength.0 * (0.88 + 0.12 * wave) * lit;
    });
}
