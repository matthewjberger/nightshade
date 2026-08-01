use crate::resources::Weather;
use crate::theme;
use nalgebra_glm::Vec3;
use nightshade::prelude::*;
use nightshade_render::data::config::{Fog, FogMode, RenderSettings, RendererState};

pub fn paint_the_sky(
    renderer_state: Res<RendererState>,
    weather: Res<Weather>,
    mut settings: ResMut<RenderSettings>,
) {
    let hour = renderer_state.day_night.hour;
    let clear = fog_color(hour);
    let overcast = theme::SKY_OVERCAST * if is_night(hour) { 0.35 } else { 1.0 };
    let cloud = weather.cloud.clamp(0.0, 1.0);
    let mut color = clear + (overcast - clear) * cloud * 0.8;
    let mut bloom = if is_night(hour) {
        theme::BLOOM_NIGHT
    } else {
        theme::BLOOM_DAY
    };
    if weather.flash > 0.0 {
        color = color + (Vec3::new(1.0, 1.0, 1.0) - color) * 0.65;
        bloom = theme::BLOOM_FLASH;
    }
    let falling = weather.rain.max(weather.snow).clamp(0.0, 1.0);
    let start = theme::FOG_START + (theme::RAIN_FOG_START - theme::FOG_START) * falling;
    let end = theme::FOG_END + (theme::RAIN_FOG_END - theme::FOG_END) * falling;
    settings.fog = Some(Fog {
        color: [color.x, color.y, color.z],
        start,
        end,
        mode: FogMode::Linear,
        height: theme::FOG_HEIGHT,
        height_falloff: theme::FOG_FALLOFF,
        ..Default::default()
    });
    settings.bloom_intensity = bloom;
}

fn is_night(hour: f32) -> bool {
    !(theme::DAWN_HOUR..theme::DUSK_HOUR + 1.0).contains(&hour)
}

fn fog_color(hour: f32) -> Vec3 {
    let stops = [
        (0.0, theme::SKY_NIGHT),
        (theme::DAWN_HOUR - 1.0, theme::SKY_NIGHT),
        (theme::DAWN_HOUR + 0.5, theme::SKY_DAWN),
        (theme::DAWN_HOUR + 2.5, theme::SKY_DAY),
        (theme::DUSK_HOUR - 1.5, theme::SKY_DAY),
        (theme::DUSK_HOUR + 0.5, theme::SKY_DUSK),
        (theme::DUSK_HOUR + 2.0, theme::SKY_NIGHT),
        (24.0, theme::SKY_NIGHT),
    ];
    let mut found = stops[0].1;
    for pair in stops.windows(2) {
        let (start, first) = pair[0];
        let (end, second) = pair[1];
        if hour >= start && hour <= end {
            let span = (end - start).max(0.001);
            let share = ((hour - start) / span).clamp(0.0, 1.0);
            found = first + (second - first) * share;
            break;
        }
    }
    found
}
