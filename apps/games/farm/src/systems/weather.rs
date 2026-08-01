use crate::commands::effects::{spawn_rain, spawn_wind_leaves};
use crate::commands::pocket::show_toast;
use crate::data::WeatherKind;
use crate::queries::weather::announcement;
use crate::resources::{Diorama, Dungeon, Home, Toast, Weather};
use crate::theme;
use nalgebra_glm::Vec3;
use nightshade::prelude::*;
use nightshade_particles::contract::ParticleEmitter;
use nightshade_platform::prelude::*;
use nightshade_render::data::config::RendererState;

pub fn plan_the_weather(diorama: Res<Diorama>, mut weather: ResMut<Weather>) {
    if weather.planned_day == Some(diorama.day) {
        return;
    }
    weather.planned_day = Some(diorama.day);
    let roll = weather.rng.f32();
    let clear = theme::CLEAR_SHARE;
    let cloudy = clear + theme::CLOUDY_SHARE;
    let fog = cloudy + theme::FOG_SHARE;
    let wind = fog + theme::WIND_SHARE;
    let rain = wind + theme::RAIN_SHARE;
    let kind = if roll < clear {
        WeatherKind::Clear
    } else if roll < cloudy {
        WeatherKind::Cloudy
    } else if roll < fog {
        WeatherKind::Fog
    } else if roll < wind {
        WeatherKind::Wind
    } else if roll < rain {
        WeatherKind::Rain
    } else {
        WeatherKind::Storm
    };
    weather.kind = kind;
    weather.window = match kind {
        WeatherKind::Fog => Some(theme::FOG_HOURS),
        WeatherKind::Rain | WeatherKind::Storm => {
            let start = theme::RAIN_EARLIEST
                + weather.rng.f32() * (theme::RAIN_LATEST - theme::RAIN_EARLIEST);
            let length = theme::RAIN_MIN_HOURS
                + weather.rng.f32() * (theme::RAIN_MAX_HOURS - theme::RAIN_MIN_HOURS);
            Some((start, start + length))
        }
        WeatherKind::Clear | WeatherKind::Cloudy | WeatherKind::Wind => None,
    };
}

fn fade(value: &mut f32, target: f32, step: f32) {
    *value += (target - *value).clamp(-step, step);
}

pub fn rain_and_shine(
    time: Res<Time>,
    renderer_state: Res<RendererState>,
    diorama: Res<Diorama>,
    dungeon: Res<Dungeon>,
    home: Res<Home>,
    mut weather: ResMut<Weather>,
    mut toast: ResMut<Toast>,
    mut commands: Commands,
    emitters: Query<(&mut ParticleEmitter,)>,
) {
    let hour = renderer_state.day_night.hour;
    let delta_time = time.delta_time;
    weather.active = weather
        .window
        .is_none_or(|(start, end)| hour >= start && hour < end);
    let kind = weather.kind;
    let active = weather.active;
    weather.raining = active && kind.rains();
    weather.gust += delta_time;

    if weather.announced != Some((kind, active)) {
        weather.announced = Some((kind, active));
        if let Some(text) = announcement(kind, active) {
            show_toast(&mut toast, text);
        }
    }

    let step = delta_time / theme::RAIN_FADE;
    let raining = weather.raining;
    fade(&mut weather.rain, if raining { 1.0 } else { 0.0 }, step);
    fade(
        &mut weather.cloud,
        if active && kind.clouds() { 1.0 } else { 0.0 },
        step * 0.5,
    );
    fade(
        &mut weather.fog,
        if active && kind == WeatherKind::Fog {
            1.0
        } else {
            0.0
        },
        step * 0.5,
    );
    fade(
        &mut weather.wind,
        if active && kind.blows() { 1.0 } else { 0.0 },
        step * 0.5,
    );

    let above = diorama.feet + Vec3::new(0.0, theme::RAIN_HEIGHT, 0.0);
    let rain_emitter = match weather.rain_emitter {
        Some(entity) => entity,
        None => {
            let entity = spawn_rain(&mut commands, above);
            weather.rain_emitter = Some(entity);
            entity
        }
    };
    let upwind = diorama.feet + Vec3::new(-14.0, 3.0, 0.0);
    let leaf_emitter = match weather.leaf_emitter {
        Some(entity) => entity,
        None => {
            let entity = spawn_wind_leaves(&mut commands, upwind);
            weather.leaf_emitter = Some(entity);
            entity
        }
    };
    let sheltered = dungeon.active || home.active;
    let raining = raining && !sheltered;
    let blowing = weather.wind > 0.3 && !sheltered;
    let mut emitters = emitters;
    emitters.for_entity(rain_emitter, |(emitter,)| {
        emitter.position = above;
        emitter.enabled = raining;
    });
    emitters.for_entity(leaf_emitter, |(emitter,)| {
        emitter.position = upwind;
        emitter.enabled = blowing;
    });

    weather.flash = (weather.flash - delta_time).max(0.0);
    if raining && kind == WeatherKind::Storm {
        weather.until_lightning -= delta_time;
        if weather.until_lightning <= 0.0 {
            weather.flash = theme::LIGHTNING_SECONDS;
            weather.until_lightning = theme::LIGHTNING_GAP_MIN
                + weather.rng.f32() * (theme::LIGHTNING_GAP_MAX - theme::LIGHTNING_GAP_MIN);
        }
    } else {
        weather.until_lightning = theme::LIGHTNING_GAP_MIN;
    }
}
