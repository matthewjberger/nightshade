use crate::commands::{spawn_rainfall, spawn_snowfall, spawn_splashes};
use crate::data::settings::Settings;
use crate::data::weather::{SEASONS, WeatherKind};
use crate::queries::{cell_corner, cell_floors, cloud_height};
use crate::resources::{Falling, Overhead, Season, Sky};
use crate::theme;
use blocker_blocks::data::blocks::BlockKind;
use blocker_blocks::queries::{block_at, ground_height};
use blocker_blocks::resources::VoxelWorld;
use blocker_screen::data::notice::Notice;
use nalgebra_glm::{IVec3, Vec3};
use nightshade::ecs::system_param::EventWriter;
use nightshade::prelude::*;
use nightshade_config::prelude::Config;
use nightshade_particles::contract::ParticleEmitter;
use nightshade_platform::prelude::Time;
use nightshade_random::Random;
use nightshade_random::commands::{between, chance};
use nightshade_render::data::config::RendererState;
use std::f32::consts::TAU;

pub fn turn_the_weather(
    time: Res<Time>,
    world: Res<VoxelWorld>,
    renderer_state: Res<RendererState>,
    config: Res<Config<Settings>>,
    overhead: Res<Overhead>,
    mut sky: ResMut<Sky>,
    mut season: ResMut<Season>,
    mut falling: ResMut<Falling>,
    mut random: ResMut<Random>,
    mut notices: EventWriter<Notice>,
    mut commands: Commands,
    mut emitters: Query<(&mut ParticleEmitter,)>,
) {
    let delta_time = time.delta_time.min(0.05);
    let over = overhead.0;
    let (x, z) = (over.x.floor() as i32, over.z.floor() as i32);
    let hour = renderer_state.day_night.hour;
    let climate = climate_under(&world, x, z);

    season.elapsed += delta_time;
    season.share = (season.elapsed / config.number(Settings::SeasonSeconds)).fract();
    let target = target_temperature(&config, season.share, climate, hour);
    season.temperature += (target - season.temperature) * (theme::TEMP_EASE * delta_time).min(1.0);

    let season_index = (season.share * SEASONS.len() as f32) as usize % SEASONS.len();
    if season.told != Some(season_index) {
        let first = season.told.is_none();
        season.told = Some(season_index);
        if !first {
            notices.send(Notice(format!("{} arrives", SEASONS[season_index])));
        }
    }

    sky.until -= delta_time;
    if sky.until <= 0.0 {
        let cloudy = config.number(Settings::CloudyShare);
        sky.kind = choose_the_sky(&mut random, climate, season.temperature, hour, cloudy);
        let shortest = config.number(Settings::WeatherShortest);
        let longest = config.number(Settings::WeatherLongest).max(shortest);
        sky.until = between(&mut random, shortest, longest);
    }

    let kind = sky.kind;
    if sky.told != Some(kind) {
        sky.told = Some(kind);
        notices.send(Notice(kind.word().into()));
    }

    let open = sky_is_open(&world, x, z, over.y);
    let step = delta_time / theme::WEATHER_FADE;
    let switched = |key: Settings| if config.toggle(key) { 1.0 } else { 0.0 };
    let wanted_fog = if open { kind.fog() } else { kind.fog() * 0.3 } * switched(Settings::Fog);
    fade(&mut sky.rain, kind.rain() * switched(Settings::Rain), step);
    fade(&mut sky.snow, kind.snow() * switched(Settings::Snow), step);
    fade(&mut sky.fog, wanted_fog, step);
    fade(&mut sky.cloud, kind.clouds(), step * 0.5);
    fade(&mut sky.wind, kind.wind() * switched(Settings::Wind), step);
    sky.wind_heading += between(&mut random, -1.0, 1.0) * theme::WIND_WANDER * delta_time;
    let blow = Vec3::new(sky.wind_heading.cos(), 0.0, sky.wind_heading.sin()) * sky.wind;

    if falling.rain.is_empty() {
        for _ in 0..theme::CELLS_ACROSS * theme::CELLS_ACROSS {
            let rain = spawn_rainfall(&mut commands);
            let snow = spawn_snowfall(&mut commands);
            let splash = spawn_splashes(&mut commands);
            falling.rain.push(rain);
            falling.snow.push(snow);
            falling.splash.push(splash);
        }
    }
    let splashing = config.toggle(Settings::Splashes);
    let raining = sky.rain > 0.05;
    let snowing = sky.snow > 0.05;
    if !raining && !snowing {
        falling.anchor = None;
    }
    let anchor = IVec3::new(x, over.y.floor() as i32, z);
    if (raining || snowing) && falling.anchor != Some(anchor) {
        falling.anchor = Some(anchor);
        falling.floors = cell_floors(&world, anchor);
    }
    let cloud = cloud_height(anchor) as f32;
    let half = theme::CELL_SIZE as f32 * 0.5;
    for index in 0..falling.rain.len() {
        let (rainfall, snowfall, splash) = (
            falling.rain[index],
            falling.snow[index],
            falling.splash[index],
        );
        let corner = cell_corner(anchor, index);
        let (floor, flat) = falling.floors.get(index).copied().unwrap_or((cloud, false));
        let drop = cloud - floor;
        let clear = drop > theme::SHORTEST_DROP;
        let center = Vec3::new(corner.x as f32 + half, cloud, corner.y as f32 + half);
        emitters.for_entity(splash, |(emitter,)| {
            emitter.position = Vec3::new(center.x, floor + theme::SPLASH_RISE, center.z);
            emitter.enabled = raining && clear && flat && splashing;
        });
        for (entity, falling_now, speed, slant) in [
            (
                rainfall,
                raining,
                theme::RAIN_SPEED,
                blow * theme::RAIN_SLANT,
            ),
            (
                snowfall,
                snowing,
                theme::SNOW_SPEED,
                blow * theme::SNOW_SLANT,
            ),
        ] {
            let path = Vec3::new(slant.x, -1.0, slant.z);
            let lifetime = drop * path.magnitude() / speed;
            emitters.for_entity(entity, |(emitter,)| {
                emitter.position = center - slant * drop;
                emitter.direction = path.normalize();
                emitter.enabled = falling_now && clear;
                emitter.particle_lifetime_min = lifetime;
                emitter.particle_lifetime_max = lifetime;
            });
        }
    }

    sky.flash = (sky.flash - delta_time).max(0.0);
    let gap_min = config.number(Settings::LightningGapMin);
    if raining && kind == WeatherKind::Storm && config.toggle(Settings::Lightning) {
        sky.until_lightning -= delta_time;
        if sky.until_lightning <= 0.0 {
            sky.flash = theme::FLASH_SECONDS;
            let gap_max = config.number(Settings::LightningGapMax).max(gap_min);
            sky.until_lightning = between(&mut random, gap_min, gap_max);
        }
    } else {
        sky.until_lightning = gap_min;
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Climate {
    Temperate,
    Cold,
    Arid,
}

fn climate_under(world: &VoxelWorld, x: i32, z: i32) -> Climate {
    let Some(top) = ground_height(world, x, z) else {
        return Climate::Temperate;
    };
    match block_at(world, IVec3::new(x, top, z)) {
        BlockKind::Snow | BlockKind::SnowGrass | BlockKind::Ice => Climate::Cold,
        BlockKind::Sand | BlockKind::RedSand | BlockKind::Terracotta => Climate::Arid,
        _ => Climate::Temperate,
    }
}

fn humidity(climate: Climate) -> f32 {
    match climate {
        Climate::Arid => 0.25,
        Climate::Cold => 0.6,
        Climate::Temperate => 0.72,
    }
}

fn target_temperature(config: &Config<Settings>, season: f32, climate: Climate, hour: f32) -> f32 {
    let base = config.number(Settings::SeasonMid)
        + config.number(Settings::SeasonSwing) * -(season * TAU).cos();
    let offset = match climate {
        Climate::Arid => config.number(Settings::AridWarmth),
        Climate::Cold => -config.number(Settings::ColdChill),
        Climate::Temperate => 0.0,
    };
    let night =
        if (crate::data::weather::DAWN_HOUR..crate::data::weather::DUSK_HOUR).contains(&hour) {
            0.0
        } else {
            config.number(Settings::NightChill)
        };
    base + offset - night
}

fn choose_the_sky(
    random: &mut Random,
    climate: Climate,
    temperature: f32,
    hour: f32,
    cloudy_share: f32,
) -> WeatherKind {
    let humid = humidity(climate);
    let dim = (hour - crate::data::weather::DAWN_HOUR).abs() < 1.5
        || (hour - crate::data::weather::DUSK_HOUR).abs() < 1.5;
    if dim && chance(random, humid * 0.35) {
        return WeatherKind::Fog;
    }
    if temperature <= theme::HARD_FREEZE {
        if chance(random, humid) {
            return WeatherKind::Blizzard;
        }
        return WeatherKind::Snowfall;
    }
    if temperature <= theme::FREEZING {
        if chance(random, humid * 0.7) {
            return WeatherKind::Snowfall;
        }
        if chance(random, 0.5) {
            return WeatherKind::Cloudy;
        }
        return WeatherKind::Clear;
    }
    if chance(random, humid) {
        if temperature > 20.0 && chance(random, 0.35) {
            return WeatherKind::Storm;
        }
        if chance(random, 0.5) {
            return WeatherKind::Rain;
        }
        return WeatherKind::Drizzle;
    }
    if chance(random, cloudy_share + 0.2) {
        WeatherKind::Cloudy
    } else {
        WeatherKind::Clear
    }
}

fn sky_is_open(world: &VoxelWorld, x: i32, z: i32, height: f32) -> bool {
    ground_height(world, x, z).is_none_or(|top| top as f32 <= height + 1.0)
}

fn fade(value: &mut f32, target: f32, step: f32) {
    *value += (target - *value).clamp(-step, step);
}
