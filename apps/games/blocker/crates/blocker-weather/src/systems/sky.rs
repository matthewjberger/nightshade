use crate::resources::Sky;
use crate::theme;
use blocker_screen::data::screen::Screen;
use blocker_screen::queries::is_pregame;
use nalgebra_glm::Vec3;
use nightshade::prelude::*;
use nightshade_render::data::config::{Fog, FogMode, RenderSettings, RendererState};
use nightshade_state::prelude::State;

pub fn paint_the_sky(
    state: Res<State<Screen>>,
    mut renderer_state: ResMut<RendererState>,
    sky: Res<Sky>,
    mut settings: ResMut<RenderSettings>,
) {
    let pregame = is_pregame(state.current);
    if pregame {
        renderer_state.day_night.hour = theme::MENU_HOUR;
    }
    let hour = renderer_state.day_night.hour;
    let clear = fog_color(hour);
    let overcast = theme::SKY_OVERCAST * if is_night(hour) { 0.35 } else { 1.0 };
    let cloud = if pregame {
        0.0
    } else {
        sky.cloud.clamp(0.0, 1.0)
    };
    let mut color = clear + (overcast - clear) * cloud * 0.8;
    let mut bloom = if is_night(hour) {
        theme::BLOOM_NIGHT
    } else {
        theme::BLOOM_DAY
    };
    if !pregame && sky.flash > 0.0 {
        color = color + (Vec3::new(1.0, 1.0, 1.0) - color) * 0.65;
        bloom = theme::BLOOM_FLASH;
    }
    let falling = if pregame {
        0.0
    } else {
        sky.rain.max(sky.snow).clamp(0.0, 1.0)
    };
    let mist = if pregame {
        0.0
    } else {
        sky.fog.clamp(0.0, 1.0)
    };
    let rain_start = theme::FOG_START + (theme::RAIN_FOG_START - theme::FOG_START) * falling;
    let rain_end = theme::FOG_END + (theme::RAIN_FOG_END - theme::FOG_END) * falling;
    let start = rain_start + (theme::FOG_NEAR_START - rain_start) * mist;
    let end = rain_end + (theme::FOG_NEAR_END - rain_end) * mist;
    let fog = Fog {
        color: [color.x, color.y, color.z],
        start,
        end,
        mode: FogMode::Linear,
        height: theme::FOG_HEIGHT,
        height_falloff: theme::FOG_FALLOFF,
        ..Default::default()
    };
    if settings.fog != Some(fog) {
        settings.fog = Some(fog);
    }
    if settings.bloom_intensity != bloom {
        settings.bloom_intensity = bloom;
    }
}

fn is_night(hour: f32) -> bool {
    !(crate::data::weather::DAWN_HOUR..crate::data::weather::DUSK_HOUR + 1.0).contains(&hour)
}

fn fog_color(hour: f32) -> Vec3 {
    let stops = [
        (0.0, theme::SKY_NIGHT),
        (crate::data::weather::DAWN_HOUR - 1.0, theme::SKY_NIGHT),
        (crate::data::weather::DAWN_HOUR + 0.5, theme::SKY_DAWN),
        (crate::data::weather::DAWN_HOUR + 2.5, theme::SKY_DAY),
        (crate::data::weather::DUSK_HOUR - 1.5, theme::SKY_DAY),
        (crate::data::weather::DUSK_HOUR + 0.5, theme::SKY_DUSK),
        (crate::data::weather::DUSK_HOUR + 2.0, theme::SKY_NIGHT),
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
