use crate::commands::{spawn_rainfall, spawn_snowfall, toast_show};
use crate::data::{Biome, WeatherKind};
use crate::queries::{biome_at, ground_height};
use crate::resources::{Generation, Player, Toast, VoxelWorld, Weather};
use crate::theme;
use fastrand::Rng;
use nalgebra_glm::Vec3;
use nightshade::prelude::*;
use nightshade_particles::contract::ParticleEmitter;
use nightshade_platform::prelude::Time;

pub fn turn_the_weather(
    time: Res<Time>,
    world: Res<VoxelWorld>,
    generation: Res<Generation>,
    player: Res<Player>,
    mut weather: ResMut<Weather>,
    mut toast: ResMut<Toast>,
    mut commands: Commands,
    mut emitters: Query<(&mut ParticleEmitter,)>,
) {
    if !generation.plan.weather {
        return;
    }
    let delta_time = time.delta_time.min(0.05);
    let (x, z) = (
        player.position.x.floor() as i32,
        player.position.z.floor() as i32,
    );
    let biome = biome_at(&generation, x, z);

    weather.until -= delta_time;
    if weather.until <= 0.0 {
        weather.kind = roll_the_sky(&mut weather.rng, biome);
        weather.until = theme::WEATHER_SHORTEST
            + weather.rng.f32() * (theme::WEATHER_LONGEST - theme::WEATHER_SHORTEST);
    }

    let kind = for_the_ground(weather.kind, biome);
    if weather.told != Some(kind) {
        weather.told = Some(kind);
        toast_show(&mut toast, kind.word());
    }

    let open = sky_is_open(&world, x, z, player.position.y);
    let step = delta_time / theme::WEATHER_FADE;
    let wanted_rain = if kind.wets() && open { 1.0 } else { 0.0 };
    let wanted_snow = if kind.snows() && open { 1.0 } else { 0.0 };
    fade(&mut weather.rain, wanted_rain, step);
    fade(&mut weather.snow, wanted_snow, step);
    fade(&mut weather.cloud, kind.clouds(), step * 0.5);

    let above = player.position + Vec3::new(0.0, theme::RAIN_HEIGHT, 0.0);
    let rainfall = match weather.rain_emitter {
        Some(entity) => entity,
        None => {
            let entity = spawn_rainfall(&mut commands, above);
            weather.rain_emitter = Some(entity);
            entity
        }
    };
    let snowfall = match weather.snow_emitter {
        Some(entity) => entity,
        None => {
            let entity = spawn_snowfall(&mut commands, above);
            weather.snow_emitter = Some(entity);
            entity
        }
    };
    let raining = weather.rain > 0.05;
    let snowing = weather.snow > 0.05;
    emitters.for_entity(rainfall, |(emitter,)| {
        emitter.position = above;
        emitter.enabled = raining;
    });
    emitters.for_entity(snowfall, |(emitter,)| {
        emitter.position = above;
        emitter.enabled = snowing;
    });

    weather.flash = (weather.flash - delta_time).max(0.0);
    if raining && kind == WeatherKind::Storm {
        weather.until_lightning -= delta_time;
        if weather.until_lightning <= 0.0 {
            weather.flash = theme::FLASH_SECONDS;
            weather.until_lightning = theme::LIGHTNING_GAP_MIN
                + weather.rng.f32() * (theme::LIGHTNING_GAP_MAX - theme::LIGHTNING_GAP_MIN);
        }
    } else {
        weather.until_lightning = theme::LIGHTNING_GAP_MIN;
    }
}

fn for_the_ground(kind: WeatherKind, biome: Biome) -> WeatherKind {
    match kind {
        WeatherKind::Rain | WeatherKind::Storm if biome.is_cold() => WeatherKind::Snowfall,
        WeatherKind::Snowfall if !biome.is_cold() => WeatherKind::Rain,
        other => other,
    }
}

fn roll_the_sky(rng: &mut Rng, biome: Biome) -> WeatherKind {
    let roll = rng.f32();
    if biome.is_parched() {
        return if roll < theme::PARCHED_CLEAR {
            WeatherKind::Clear
        } else {
            WeatherKind::Cloudy
        };
    }
    let clear = theme::CLEAR_SHARE;
    let cloudy = clear + theme::CLOUDY_SHARE;
    let rain = cloudy + theme::RAIN_SHARE;
    if roll < clear {
        WeatherKind::Clear
    } else if roll < cloudy {
        WeatherKind::Cloudy
    } else if roll < rain {
        WeatherKind::Rain
    } else {
        WeatherKind::Storm
    }
}

fn sky_is_open(world: &VoxelWorld, x: i32, z: i32, height: f32) -> bool {
    ground_height(world, x, z).is_none_or(|top| top as f32 <= height + 1.0)
}

fn fade(value: &mut f32, target: f32, step: f32) {
    *value += (target - *value).clamp(-step, step);
}
