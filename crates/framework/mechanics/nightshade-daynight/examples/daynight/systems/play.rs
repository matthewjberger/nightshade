use crate::queries::{facing, mix};
use crate::resources::Sky;
use crate::theme;
use nightshade::prelude::*;
use nightshade_daynight::commands::set_hour;
use nightshade_daynight::queries::{is_night, night_share, sun_direction};
use nightshade_daynight::{Clock, DaySettings};
use nightshade_platform::contract::input::Input;
use nightshade_platform::contract::time::Time;
use nightshade_platform::prelude::KeyCode;
use nightshade_readout::Readout;
use nightshade_render::prelude::RenderSettings;
use nightshade_scene::contract::LocalTransform;

pub fn take_the_orders(
    input: Res<Input>,
    time: Res<Time>,
    sky: Res<Sky>,
    mut clock: ResMut<Clock>,
    mut settings: ResMut<DaySettings>,
) {
    let mut turn = 0.0;
    if input.keyboard.is_key_pressed(KeyCode::ArrowRight) {
        turn += theme::SCRUB;
    }
    if input.keyboard.is_key_pressed(KeyCode::ArrowLeft) {
        turn -= theme::SCRUB;
    }
    if turn != 0.0 {
        let hour = clock.0 + turn * time.delta_time;
        set_hour(&mut clock, hour);
    }
    if input.keyboard.just_pressed(KeyCode::Digit1) {
        set_hour(&mut clock, settings.dawn);
    }
    if input.keyboard.just_pressed(KeyCode::Digit2) {
        set_hour(&mut clock, (settings.dawn + settings.dusk) * 0.5);
    }
    if input.keyboard.just_pressed(KeyCode::Digit3) {
        set_hour(&mut clock, settings.dusk);
    }
    if input.keyboard.just_pressed(KeyCode::Digit4) {
        set_hour(&mut clock, 0.0);
    }
    if input.keyboard.just_pressed(KeyCode::Space) {
        settings.hours_a_second = if settings.hours_a_second > 0.0 {
            0.0
        } else {
            sky.held
        };
    }
}

pub fn watch_the_sky(
    clock: Res<Clock>,
    settings: Res<DaySettings>,
    sky: Res<Sky>,
    mut render: ResMut<RenderSettings>,
    mut readout: ResMut<Readout>,
    mut suns: Query<(&mut LocalTransform,)>,
) {
    let share = night_share(&clock, &settings);
    let dark = is_night(&clock, &settings);
    let direction = sun_direction(&clock);

    if let Some(sun) = sky.sun {
        suns.for_entity(sun, |(transform,)| {
            transform.translation = -direction * theme::SUN_BACK;
            transform.rotation = facing(direction);
        });
    }
    render.clear_color = mix(theme::DAY_SKY, theme::NIGHT_SKY, share);

    readout.notes.clear();
    readout.notes.push(format!("hour {:.1}", clock.0));
    readout
        .notes
        .push(if dark { "night" } else { "day" }.to_string());
    readout.notes.push(format!("night share {share:.2}"));
    readout.notes.push(format!(
        "sun {:.2}, {:.2}, {:.2}",
        direction.x, direction.y, direction.z
    ));
    readout
        .notes
        .push(format!("{:.2} hours a second", settings.hours_a_second));
}
