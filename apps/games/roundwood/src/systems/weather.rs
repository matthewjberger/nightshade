use crate::commands::effects::spawn_rain;
use crate::data;
use crate::queries::cylinder::across_of;
use crate::resources::{Village, Weather, WeatherKind};
use nalgebra_glm::Vec3;
use nightshade::prelude::*;
use nightshade_particles::contract::ParticleEmitter;
use nightshade_platform::prelude::*;

pub fn plan_the_weather(time: Res<Time>, mut weather: ResMut<Weather>) {
    weather.until -= time.delta_time;
    if weather.until > 0.0 {
        return;
    }
    weather.until = data::WEATHER_SPELL;
    let roll = weather.rng.u32(..) % 100;
    weather.kind = if roll < data::RAIN_SHARE {
        WeatherKind::Rain
    } else if roll < data::RAIN_SHARE + data::CLOUD_SHARE {
        WeatherKind::Cloudy
    } else {
        WeatherKind::Clear
    };
}

pub fn rain_and_shine(
    time: Res<Time>,
    village: Res<Village>,
    mut weather: ResMut<Weather>,
    mut commands: Commands,
    mut emitters: Query<(&mut ParticleEmitter,)>,
) {
    let step = data::WEATHER_FADE * time.delta_time;
    let wanted_cloud = match weather.kind {
        WeatherKind::Clear => 0.0,
        WeatherKind::Cloudy => 0.75,
        WeatherKind::Rain => 1.0,
    };
    let wanted_rain = if weather.kind == WeatherKind::Rain {
        1.0
    } else {
        0.0
    };
    weather.cloud += (wanted_cloud - weather.cloud).clamp(-step, step);
    weather.rain += (wanted_rain - weather.rain).clamp(-step, step);

    let above = Vec3::new(
        across_of(village.feet.x),
        village.lift + data::RAIN_LIFT,
        0.0,
    );
    let emitter = match weather.emitter {
        Some(emitter) => emitter,
        None => {
            let emitter = spawn_rain(&mut commands, above);
            weather.emitter = Some(emitter);
            return;
        }
    };
    let raining = weather.rain > 0.02;
    let rate = data::RAIN_RATE * weather.rain;
    emitters.for_entity(emitter, |(emitter,)| {
        emitter.position = above;
        emitter.enabled = raining;
        emitter.spawn_rate = rate;
    });
}
