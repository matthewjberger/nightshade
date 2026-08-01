use crate::commands::audio::cue;
use crate::commands::banner;
use crate::commands::chat::system_line;
use crate::components::Cooldowns;
use crate::data::chatter::DISCOVERY_LINES;
use crate::data::{
    DAY_SPEED, NIGHT_EXPOSURE_EV, PLACES, SOUND_DISCOVER, SOUND_PLAIN, SUN_DISTANCE,
};
use crate::queries::{night_amount, place_at, sun_pose};
use crate::resources::{Banners, Cues, Feel, ThornmereResources};
use nightshade::prelude::*;
use nightshade_graphics::prelude::RenderSettings;
use nightshade_platform::prelude::Time;
use nightshade_render::contract::Light;
use nightshade_render::data::config::RendererState;
use nightshade_scene::prelude::LocalTransform;

const DAY_FOG: [f32; 3] = [0.72, 0.80, 0.90];
const NIGHT_FOG: [f32; 3] = [0.08, 0.10, 0.18];
const DISCOVERY_COLOR: Vec4 = Vec4::new(0.98, 0.9, 0.6, 1.0);

pub fn turn(
    time: Res<Time>,
    mut renderer_state: ResMut<RendererState>,
    feel: Res<Feel>,
    mut settings: ResMut<RenderSettings>,
    mut resources: ResMut<ThornmereResources>,
    mut cues: ResMut<Cues>,
    mut banners: ResMut<Banners>,
    mut suns: Query<(&mut Light, &mut LocalTransform)>,
    players: Query<(&LocalTransform,), With<Cooldowns>>,
) {
    let hour = (renderer_state.day_night.hour + DAY_SPEED * time.delta_time).rem_euclid(24.0);
    renderer_state.day_night.hour = hour;
    if let Some(sun) = renderer_state.day_night.sun_entity {
        let (direction, intensity, color) = sun_pose(hour);
        suns.for_entity(sun, |(light, transform)| {
            light.intensity = intensity;
            light.color = color;
            transform.translation = direction * SUN_DISTANCE;
            let look = -direction;
            let right = look.cross(&Vec3::y()).normalize();
            let up = right.cross(&look);
            transform.rotation =
                nalgebra_glm::mat3_to_quat(&nalgebra_glm::Mat3::from_columns(&[right, up, -look]));
        });
    }
    let night = night_amount(hour);
    settings.color_grading.exposure_compensation_ev = NIGHT_EXPOSURE_EV * night;
    settings.color_grading.saturation = (1.0 - 0.3 * night) * (1.0 - 0.85 * feel.grey);
    if let Some(fog) = settings.fog.as_mut() {
        for index in 0..3 {
            fog.color[index] = DAY_FOG[index] + (NIGHT_FOG[index] - DAY_FOG[index]) * night;
        }
    }

    let mut position = None;
    players.for_each(|_entity, (transform,)| position = Some(transform.translation));
    let Some(position) = position else {
        return;
    };
    let resources = &mut *resources;
    let here = place_at(position);
    if here == resources.clock.last_place {
        return;
    }
    resources.clock.last_place = here;
    let Some(place) = here else {
        return;
    };
    if resources.clock.discovered[place] {
        return;
    }
    resources.clock.discovered[place] = true;
    system_line(
        &mut resources.chat,
        format!("Discovered: {}", PLACES[place].name),
    );
    system_line(&mut resources.chat, DISCOVERY_LINES[place].to_string());
    cue(&mut cues, SOUND_DISCOVER, SOUND_PLAIN);
    banner(
        &mut banners,
        PLACES[place].name.to_string(),
        "Discovered".to_string(),
        DISCOVERY_COLOR,
    );
}
