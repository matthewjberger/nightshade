use crate::components::{Lit, NightOnly, Phase, Strength};
use crate::tags::Moon;

use crate::queries::sky::sky;
use crate::resources::{Diorama, Dungeon, Home, Weather};
use crate::theme;
use nalgebra_glm::Vec3;
use nightshade::prelude::*;
use nightshade_graphics::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_render::commands::material_registry::material_registry_mutate;
use nightshade_render::components::light::{Light, LightType};
use nightshade_render::data::config::RendererState;
use nightshade_render::prelude::{golden_factor, night_factor, smoothstep};
use nightshade_sprite::prelude::Sprite;

pub fn follow_the_hour(
    time: Res<Time>,
    renderer_state: Res<RendererState>,
    weather: Res<Weather>,
    dungeon: Res<Dungeon>,
    home: Res<Home>,
    mut settings: ResMut<RenderSettings>,
    mut diorama: ResMut<Diorama>,
    mut registry: ResMut<MaterialRegistry>,
    moons: Query<(&mut Light,), WithTag<Moon>>,
    flames: Query<(&mut Phase, &Strength, &NightOnly, &mut Light)>,
) {
    let hour = renderer_state.day_night.hour;
    if hour < diorama.last_hour {
        diorama.day += 1;
    }
    diorama.last_hour = hour;

    let night = night_factor(hour, &theme::TWILIGHT);
    let golden = golden_factor(hour, &theme::TWILIGHT);
    let rain = weather.rain;
    let cloud = weather.cloud;
    let fog_thick = weather.fog;
    let storm = if weather.kind == crate::data::WeatherKind::Storm {
        rain
    } else {
        0.0
    };
    let flash = if weather.flash > 0.0 { 1.0 } else { 0.0 };
    let heavens = sky(hour, cloud, fog_thick);
    let dark = 1.0 - heavens.daylight;
    let below = dungeon.active;
    let inside = home.active;

    let grading = &mut settings.color_grading;
    grading.exposure_compensation_ev = theme::NIGHT_EXPOSURE_EV * dark
        + 0.15 * heavens.glow * (1.0 - cloud)
        + theme::CLOUD_EXPOSURE_EV * cloud
        + theme::RAIN_EXPOSURE_EV * rain
        + theme::STORM_EXPOSURE_EV * storm
        + theme::LIGHTNING_EV * flash;
    grading.saturation =
        1.0 + 0.25 * golden * (1.0 - cloud) - 0.35 * dark - 0.2 * cloud - 0.15 * rain;
    grading.vibrance = 0.3 * heavens.glow;
    grading.contrast = 1.0 + 0.08 * dark;
    grading.vignette_intensity = 0.12 + 0.3 * dark + 0.1 * rain;
    settings.bloom_intensity = theme::BLOOM_DAY + (theme::BLOOM_NIGHT - theme::BLOOM_DAY) * dark;

    let fog = heavens.horizon;
    if let Some(settings_fog) = settings.fog.as_mut() {
        if inside {
            settings_fog.color = [
                theme::INDOOR_FOG.x,
                theme::INDOOR_FOG.y,
                theme::INDOOR_FOG.z,
            ];
            settings_fog.start = theme::INDOOR_FOG_START;
            settings_fog.end = theme::INDOOR_FOG_END;
        } else if below {
            settings_fog.color = [
                theme::HOLLOW_FOG.x,
                theme::HOLLOW_FOG.y,
                theme::HOLLOW_FOG.z,
            ];
            settings_fog.start = theme::HOLLOW_FOG_START;
            settings_fog.end = theme::HOLLOW_FOG_END;
        } else {
            settings_fog.color = [fog.x, fog.y, fog.z];
            let start = theme::FOG_START + (theme::RAIN_FOG_START - theme::FOG_START) * rain;
            let end = theme::FOG_END + (theme::RAIN_FOG_END - theme::FOG_END) * rain;
            settings_fog.start = start + (theme::FOG_THICK_START - start) * fog_thick;
            settings_fog.end = end + (theme::FOG_THICK_END - end) * fog_thick;
        }
    }

    let sea = nalgebra_glm::lerp(
        &(theme::SEA_DAY * (0.12 + 0.88 * heavens.daylight)),
        &fog,
        0.55,
    );
    if (sea - diorama.sea_color).magnitude() > 0.01 {
        diorama.sea_color = sea;
        material_registry_mutate(&mut registry, theme::SEA_MATERIAL, |material| {
            material.base_color = [sea.x, sea.y, sea.z, 1.0];
        });
    }

    moons.for_each(|_entity, (light,)| {
        light.intensity = theme::MOON_STRENGTH * dark * (1.0 - 0.7 * cloud.max(rain));
    });

    let delta_time = time.delta_time;
    let lit = smoothstep(night);
    flames.for_each(|_entity, (phase, strength, night_only, light)| {
        phase.0 += delta_time;
        let wave = (phase.0 * 7.3).sin() * 0.5 + (phase.0 * 13.1 + 1.3).sin() * 0.5;
        let burn = if night_only.0 { lit } else { 1.0 };
        light.intensity = strength.0 * (0.88 + 0.12 * wave) * burn;
    });

    let glow = lit.max(0.6 * cloud.max(rain));
    if (glow - diorama.window_glow).abs() > 0.02 {
        diorama.window_glow = glow;
        material_registry_mutate(&mut registry, theme::WINDOW_MATERIAL, |material| {
            let strength = theme::WINDOW_GLOW * glow;
            material.emissive_factor = [strength.x, strength.y, strength.z];
        });
    }
}

pub fn light_the_sprites(
    renderer_state: Res<RendererState>,
    weather: Res<Weather>,
    dungeon: Res<Dungeon>,
    home: Res<Home>,
    lights: Query<(&Light, &GlobalTransform)>,
    sprites: Query<(&Lit, &GlobalTransform, &mut Sprite)>,
) {
    let heavens = sky(renderer_state.day_night.hour, weather.cloud, weather.fog);
    let ambient = if dungeon.active {
        theme::HOLLOW_AMBIENT
    } else if home.active {
        theme::INDOOR_AMBIENT
    } else {
        heavens.ambient
    };
    let mut lamps: Vec<(Vec3, Vec3, f32, f32)> = Vec::new();
    lights.for_each(|_entity, (light, transform)| {
        if light.light_type != LightType::Point || light.intensity <= 0.01 {
            return;
        }
        let position = transform.0.column(3).xyz();
        let range = light.range.max(0.5);
        lamps.push((position, light.color, light.intensity, range));
    });
    sprites.for_each(|_entity, (lit, transform, sprite)| {
        let middle = transform.0.column(3).xyz() + Vec3::new(0.0, 0.5, 0.0);
        let mut light = ambient;
        for (position, color, intensity, range) in &lamps {
            let gap = (middle - position).magnitude();
            if gap > *range {
                continue;
            }
            let window = 1.0 - gap / range;
            let reach = window * window / (1.0 + gap * gap);
            light += color * (intensity * theme::SPRITE_LIGHT_GAIN * reach);
        }
        sprite.color = [
            (lit.base[0] * light.x).min(theme::SPRITE_LIGHT_CAP),
            (lit.base[1] * light.y).min(theme::SPRITE_LIGHT_CAP),
            (lit.base[2] * light.z).min(theme::SPRITE_LIGHT_CAP),
            lit.base[3],
        ];
    });
}
