use crate::components::{Cooldowns, Experience, Kills, Level};
use crate::data::save::{SAVE_PERIOD, SAVE_SLOT};
use crate::queries::save::gather;
use crate::resources::ThornmereResources;
use nightshade::prelude::*;
use nightshade_platform::prelude::Time;
use nightshade_render::data::config::RendererState;
use nightshade_save::prelude::save_write;
use nightshade_scene::prelude::LocalTransform;

type Characters<'world, 'item> = nightshade::ecs::system_param::Query<
    'world,
    (
        &'item Level,
        &'item Experience,
        &'item Kills,
        &'item LocalTransform,
    ),
    With<Cooldowns>,
>;

pub fn keep(
    time: Res<Time>,
    renderer_state: Res<RendererState>,
    mut resources: ResMut<ThornmereResources>,
    players: Characters<'_, '_>,
) {
    let resources = &mut *resources;
    resources.clock.save_timer -= time.delta_time;
    if resources.clock.save_timer > 0.0 {
        return;
    }
    resources.clock.save_timer = SAVE_PERIOD;
    let mut sheet = None;
    players.for_each(|_entity, (level, experience, kills, transform)| {
        sheet = Some((
            level.0,
            experience.0,
            kills.0,
            [
                transform.translation.x,
                transform.translation.y,
                transform.translation.z,
            ],
        ));
    });
    let Some((level, experience, kills, spot)) = sheet else {
        return;
    };
    let save = gather(
        resources,
        level,
        experience,
        kills,
        spot,
        renderer_state.day_night.hour,
    );
    let label = format!("{} the level {level}", resources.player_name);
    save_write(SAVE_SLOT, &label, &save);
}
