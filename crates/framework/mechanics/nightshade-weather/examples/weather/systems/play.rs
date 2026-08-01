use crate::resources::Drift;
use crate::theme;
use nalgebra_glm::vec3;
use nightshade::ecs::Storage;
use nightshade::ecs::system_param::{Res, ResMut};
use nightshade_platform::contract::input::Input;
use nightshade_platform::contract::time::Time;
use nightshade_platform::prelude::KeyCode;
use nightshade_random::Random;
use nightshade_readout::Readout;
use nightshade_render::data::config::{Fog, FogMode};
use nightshade_render::prelude::RenderSettings;
use nightshade_scene::contract::LocalTransform;
use nightshade_weather::commands::roll_the_sky;
use nightshade_weather::queries::{cover, raining};
use nightshade_weather::{Sky, Weather, WeatherSettings};

fn mix(low: [f32; 3], high: [f32; 3], share: f32) -> [f32; 3] {
    [
        low[0] + (high[0] - low[0]) * share,
        low[1] + (high[1] - low[1]) * share,
        low[2] + (high[2] - low[2]) * share,
    ]
}

pub fn take_the_orders(
    input: Res<Input>,
    settings: Res<WeatherSettings>,
    mut weather: ResMut<Weather>,
    mut roll: ResMut<Random>,
) {
    let asked = if input.keyboard.just_pressed(KeyCode::Digit1) {
        Some(Sky::Clear)
    } else if input.keyboard.just_pressed(KeyCode::Digit2) {
        Some(Sky::Cloudy)
    } else if input.keyboard.just_pressed(KeyCode::Digit3) {
        Some(Sky::Rain)
    } else if input.keyboard.just_pressed(KeyCode::Digit4) {
        Some(Sky::Storm)
    } else {
        None
    };
    if let Some(sky) = asked {
        weather.sky = sky;
        weather.remaining = theme::HELD;
        return;
    }
    if input.keyboard.just_pressed(KeyCode::Space) {
        roll_the_sky(&mut weather, &settings, &mut roll);
    }
}

pub fn watch_the_weather(
    time: Res<Time>,
    weather: Res<Weather>,
    mut drift: ResMut<Drift>,
    mut render_settings: ResMut<RenderSettings>,
    mut readout: ResMut<Readout>,
    storage: &mut Storage,
) {
    let delta = time.delta_time;
    let (sky, wetness, left, thick, wet) = {
        (
            weather.sky,
            weather.wetness,
            weather.remaining,
            cover(&weather),
            raining(&weather),
        )
    };

    let turn = {
        drift.turn += delta * theme::DRIFT;
        drift.turn
    };

    let deck = theme::CEILING - thick * theme::DESCENT;
    let clouds: Vec<_> = storage
        .query_ref::<(&LocalTransform,)>()
        .iter()
        .filter(|(_entity, (transform,))| transform.translation.y > theme::CLOUD_FLOOR)
        .map(|(entity, _)| entity)
        .collect();
    let count = clouds.len().max(1) as f32;
    for (index, cloud) in clouds.iter().enumerate() {
        let angle = index as f32 / count * std::f32::consts::TAU + turn;
        let spread = theme::SPREAD - thick * theme::GATHER;
        if let Some(transform) = storage.get_mut::<LocalTransform>(*cloud) {
            transform.translation = vec3(angle.cos() * spread, deck, angle.sin() * spread);
        }
    }

    let ground = mix(theme::SKY_CLEAR, theme::SKY_STORM, thick);
    let settings = &mut *render_settings;
    settings.clear_color = [ground[0], ground[1], ground[2], 1.0];
    settings.fog = Some(Fog {
        color: mix(ground, theme::SKY_STORM, wetness),
        start: theme::FOG_START,
        end: theme::FOG_END - thick * theme::FOG_CLOSE,
        mode: FogMode::Linear,
        volumetric: false,
        density: 0.0,
        anisotropy: 0.0,
        height: 0.0,
        height_falloff: 0.0,
    });

    let name = match sky {
        Sky::Clear => "clear",
        Sky::Cloudy => "cloudy",
        Sky::Rain => "rain",
        Sky::Storm => "storm",
    };
    readout.notes.clear();
    readout.notes.push(format!("{name}, {thick:.2} cover"));
    readout.notes.push(format!(
        "{} and {wetness:.2} wet",
        if wet { "raining" } else { "dry" }
    ));
    readout.notes.push(format!("turning over in {left:.1}"));
}
