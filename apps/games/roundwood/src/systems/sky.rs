use crate::components::{Lit, Phase, Strength};
use crate::data;
use crate::queries::sky::{sun_color, sun_direction, tint};
use crate::resources::{Sky, Village, Weather};
use nalgebra_glm::Vec3;
use nightshade::prelude::*;
use nightshade_camera::queries::camera_look::camera_look_rotation;
use nightshade_graphics::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_render::components::light::Light;
use nightshade_sprite::prelude::*;

pub fn turn_the_day(
    time: Res<Time>,
    village: Res<Village>,
    weather: Res<Weather>,
    mut sky: ResMut<Sky>,
    mut settings: ResMut<RenderSettings>,
    mut suns: Query<(&mut Light, &mut LocalTransform)>,
) {
    sky.hour += time.delta_time * data::HOUR_SPEED;
    if sky.hour >= data::DAY_END {
        sky.hour = data::DAY_START;
    }

    let direction = sun_direction(sky.hour);
    let elevation = direction.y.clamp(0.0, 1.0);
    let strength = data::SUN_STRENGTH * elevation.sqrt() * (1.0 - 0.55 * weather.cloud);
    let color = sun_color(elevation);
    if let Some(sun) = village.sun {
        suns.for_entity(sun, |(light, transform)| {
            light.intensity = strength;
            light.color = color;
            transform.translation = direction * 100.0;
            transform.rotation = camera_look_rotation(direction * 100.0, Vec3::zeros(), 0.0);
        });
    }

    if let Some(fog) = settings.fog.as_mut() {
        for channel in 0..3 {
            fog.color[channel] = data::CLEAR_FOG[channel]
                + (data::RAIN_FOG[channel] - data::CLEAR_FOG[channel]) * weather.cloud;
        }
        fog.start = data::FOG_START + (data::RAIN_FOG_START - data::FOG_START) * weather.rain;
        fog.end = data::FOG_END + (data::RAIN_FOG_END - data::FOG_END) * weather.rain;
    }
    settings.color_grading.saturation = 1.0 - 0.3 * weather.cloud;
    settings.bloom_intensity = data::BLOOM_INTENSITY * (1.0 - 0.6 * weather.rain);
}

pub fn light_the_sprites(
    weather: Res<Weather>,
    mut sky: ResMut<Sky>,
    lit: Query<(&Lit, &mut Sprite)>,
) {
    let day = tint(sky.hour);
    let grey = 1.0 - 0.28 * weather.cloud;
    let wanted = [day[0] * grey, day[1] * grey, day[2] * grey];
    let settled = sky
        .tint
        .iter()
        .zip(wanted.iter())
        .all(|(held, next)| (held - next).abs() < 1.0 / 255.0);
    if settled {
        return;
    }
    sky.tint = wanted;
    lit.for_each(|_entity, (lit, sprite)| {
        sprite.color = [
            lit.0[0] * wanted[0],
            lit.0[1] * wanted[1],
            lit.0[2] * wanted[2],
            lit.0[3],
        ];
    });
}

pub fn stir_the_air(
    time: Res<Time>,
    mut sky: ResMut<Sky>,
    plants: Query<(&Phase, &Strength, &mut Sprite)>,
) {
    sky.breeze += time.delta_time * data::SWAY_RATE;
    let breeze = sky.breeze;
    plants.for_each(|_entity, (phase, strength, sprite)| {
        sprite.lean = strength.0 * (breeze + phase.0).sin();
    });
}
