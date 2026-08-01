use crate::components::{Drift, Orbit, Phase, Spin};
use crate::resources::{Player, Weather};
use crate::theme;
use nalgebra_glm::Vec3;
use nightshade::prelude::*;
use nightshade_platform::prelude::Time;
use nightshade_render::data::config::{Fog, FogMode, RenderSettings, RendererState};
use nightshade_scene::prelude::LocalTransform;

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

pub fn drift(
    time: Res<Time>,
    player: Res<Player>,
    mut clouds: Query<(&Drift, &mut LocalTransform), Without<Orbit>>,
) {
    let delta_time = time.delta_time;
    let reach = theme::CLOUD_REACH;
    clouds.for_each_mut(|_entity, (drift, transform)| {
        transform.translation += drift.0 * delta_time;
        if transform.translation.x > player.position.x + reach {
            transform.translation.x -= reach * 2.0;
        }
        if transform.translation.z > player.position.z + reach {
            transform.translation.z -= reach * 2.0;
        }
        if transform.translation.x < player.position.x - reach {
            transform.translation.x += reach * 2.0;
        }
        if transform.translation.z < player.position.z - reach {
            transform.translation.z += reach * 2.0;
        }
    });
}

pub fn wheel(
    time: Res<Time>,
    mut birds: Query<(&Orbit, &Spin, &Drift, &mut Phase, &mut LocalTransform)>,
) {
    let delta_time = time.delta_time;
    birds.for_each_mut(|_entity, (orbit, spin, drift, phase, transform)| {
        phase.0 += spin.0 * delta_time;
        let angle = phase.0;
        let radius_x = drift.0.x;
        let radius_z = drift.0.z;
        let position = Vec3::new(
            orbit.0.x + angle.cos() * radius_x,
            orbit.0.y + (angle * 3.0).sin() * 1.6,
            orbit.0.z + angle.sin() * radius_z,
        );
        let forward = Vec3::new(-angle.sin() * radius_x, 0.0, angle.cos() * radius_z) * spin.0;
        let yaw = forward.x.atan2(forward.z);
        let flap = (angle * 14.0).sin() * 0.35;
        transform.translation = position;
        transform.rotation = nalgebra_glm::quat_angle_axis(yaw, &Vec3::y())
            * nalgebra_glm::quat_angle_axis(flap, &Vec3::z());
    });
}
