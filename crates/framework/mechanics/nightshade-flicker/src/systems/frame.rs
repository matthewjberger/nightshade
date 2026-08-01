use crate::components::{FlickerBase, Flickering};
use crate::resources::FlickerSettings;
use nightshade::prelude::*;
use nightshade_platform::contract::time::Time;
use nightshade_random::Random;
use nightshade_random::commands::chance;
use nightshade_render::components::Light;

pub fn flicker_the_lights(
    time: Res<Time>,
    settings: Res<FlickerSettings>,
    mut random: ResMut<Random>,
    lamps: Query<(&FlickerBase, &mut Flickering, Option<&mut Light>)>,
) {
    let delta = time.delta_time;
    lamps.for_each(|_entity, (base, flickering, light)| {
        let noise = random.0.f32();
        let dropout = if chance(&mut random, settings.dropout_chance) {
            settings.dropout_scale
        } else {
            1.0
        };
        let target = (1.0 - settings.depth + settings.depth * noise) * dropout;
        flickering.0 += (target - flickering.0) * (delta * settings.response).min(1.0);
        if let Some(light) = light {
            light.intensity = base.0 * flickering.0;
        }
    });
}
