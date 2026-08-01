use crate::components::{Battery, Beam, Drain, Lit};
use crate::data::{CellDied, CellLow};
use crate::queries::brightness;
use crate::resources::FlashlightSettings;
use nightshade::ecs::system_param::EventWriter;
use nightshade::prelude::*;
use nightshade_platform::contract::time::Time;
use nightshade_render::components::Light;

pub fn drain_the_cell(
    time: Res<Time>,
    settings: Res<FlashlightSettings>,
    mut went_low: EventWriter<CellLow>,
    mut died: EventWriter<CellDied>,
    torches: Query<(&mut Battery, &Drain, &mut Lit, &Beam, &mut Light)>,
) {
    let delta = time.delta_time;
    torches.for_each(|entity, (battery, drain, lit, beam, light)| {
        if lit.0 && battery.0 > 0.0 {
            let before = battery.0;
            battery.0 = (before - drain.0 * delta).max(0.0);
            if before > settings.low && battery.0 <= settings.low && battery.0 > 0.0 {
                went_low.send(CellLow { entity });
            }
            if battery.0 <= 0.0 {
                lit.0 = false;
                died.send(CellDied { entity });
            }
        }
        light.intensity = beam.0 * brightness(battery, lit, &settings);
    });
}
