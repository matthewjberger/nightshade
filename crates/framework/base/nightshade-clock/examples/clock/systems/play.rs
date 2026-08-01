use crate::resources::Dial;
use crate::theme;
use nalgebra_glm::Vec3;
use nightshade::ecs::Storage;
use nightshade::ecs::system_param::{Res, ResMut};
use nightshade_clock::Clock;
use nightshade_clock::commands::{pass, set_hour};
use nightshade_clock::queries::{between, hour, share_of_day};
use nightshade_platform::contract::input::Input;
use nightshade_platform::contract::time::Time;
use nightshade_platform::prelude::KeyCode;
use nightshade_readout::Readout;
use nightshade_scene::contract::LocalTransform;

pub fn take_the_orders(input: Res<Input>, time: Res<Time>, mut clock: ResMut<Clock>) {
    let stop = if input.keyboard.just_pressed(KeyCode::Digit1) {
        Some(theme::DAWN)
    } else if input.keyboard.just_pressed(KeyCode::Digit2) {
        Some(theme::NOON)
    } else if input.keyboard.just_pressed(KeyCode::Digit3) {
        Some(theme::DUSK)
    } else if input.keyboard.just_pressed(KeyCode::Digit4) {
        Some(theme::MIDNIGHT)
    } else {
        None
    };
    if let Some(chosen) = stop {
        set_hour(&mut clock, chosen);
        return;
    }
    let step = time.delta_time * theme::HOURS_A_SECOND;
    if input.keyboard.is_key_pressed(KeyCode::ArrowRight) {
        pass(&mut clock, step);
    }
    if input.keyboard.is_key_pressed(KeyCode::ArrowLeft) {
        pass(&mut clock, -step);
    }
}

pub fn watch_the_clock(
    clock: Res<Clock>,
    dial: Res<Dial>,
    mut readout: ResMut<Readout>,
    storage: &mut Storage,
) {
    let (now, share, daylight) = {
        (
            hour(&clock),
            share_of_day(&clock),
            between(&clock, theme::DAY_FROM, theme::DAY_TO),
        )
    };

    if let Some(hand) = dial.hand
        && let Some(transform) = storage.get_mut::<LocalTransform>(hand)
    {
        let angle = -share * std::f32::consts::TAU;
        transform.rotation = nalgebra_glm::quat_angle_axis(angle, &Vec3::y());
        transform.translation.x = angle.sin() * theme::DIAL * 0.5;
        transform.translation.z = angle.cos() * theme::DIAL * 0.5;
    }

    readout.notes.clear();
    readout.notes.push(format!("hour {now:.1}"));
    readout
        .notes
        .push(if daylight { "daylight" } else { "night" }.to_string());
}
