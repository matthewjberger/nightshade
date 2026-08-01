use crate::queries::sky_line;
use crate::resources::Meadow;
use crate::theme;
use nightshade::ecs::Storage;
use nightshade::prelude::*;
use nightshade_cooldown::{Cooldown, Recharge};
use nightshade_daynight::{Clock, DaySettings};
use nightshade_health::{Health, PendingDamage};
use nightshade_plane::Plane;
use nightshade_plane::queries::across;
use nightshade_platform::contract::input::Input;
use nightshade_platform::prelude::KeyCode;
use nightshade_readout::Readout;
use nightshade_scene::contract::LocalTransform;
use nightshade_shake::{ShakeOffset, Trauma};
use nightshade_toast::{Toast, ToastSettings};
use nightshade_weather::Weather;

pub fn strike_the_nearest(meadow: Res<Meadow>, input: Res<Input>, storage: &mut Storage) {
    if !input.keyboard.just_pressed(KeyCode::Space) {
        return;
    }
    let Some(camera) = meadow.camera else {
        return;
    };
    let ready = storage
        .get::<Cooldown>(camera)
        .is_none_or(|cooldown| cooldown.0 <= 0.0);
    if !ready {
        return;
    }
    let Some(from) = storage
        .get::<LocalTransform>(camera)
        .map(|at| at.translation)
    else {
        return;
    };
    let mut nearest = None;
    for (entity, (transform, _health)) in storage.query_ref::<(&LocalTransform, &Health)>().iter() {
        let reach = across(Plane::Ground, from, transform.translation);
        if reach > theme::STRIKE_REACH {
            continue;
        }
        if nearest.is_none_or(|(_entity, known)| reach < known) {
            nearest = Some((entity, reach));
        }
    }
    let Some((struck, _reach)) = nearest else {
        return;
    };
    if let Some(pending) = storage.get_mut::<PendingDamage>(struck) {
        pending.0 += theme::STRIKE_DAMAGE;
    }
    if let Some(trauma) = storage.get_mut::<Trauma>(camera) {
        trauma.0 = (trauma.0 + theme::STRIKE_TRAUMA).min(1.0);
    }
    storage.set(camera, Cooldown(theme::STRIKE_WAIT));
    storage.set(camera, Recharge(theme::STRIKE_WAIT));
}

pub fn say_the_weather(
    weather: Res<Weather>,
    settings: Res<ToastSettings>,
    mut meadow: ResMut<Meadow>,
    mut toast: ResMut<Toast>,
) {
    if meadow.sky == weather.sky {
        return;
    }
    meadow.sky = weather.sky;
    nightshade_toast::commands::say(&mut toast, &settings, sky_line(weather.sky));
}

pub fn show_the_hour(
    clock: Res<Clock>,
    day: Res<DaySettings>,
    weather: Res<Weather>,
    mut readout: ResMut<Readout>,
) {
    readout.notes.clear();
    readout.notes.push(format!("{:02}:00", clock.0 as u32));
    readout.notes.push(format!(
        "night {:.0}%",
        nightshade_daynight::queries::night_share(&clock, &day) * 100.0
    ));
    readout
        .notes
        .push(format!("wet {:.0}%", weather.wetness * 100.0));
}

pub fn carry_the_shake(cameras: Query<(&mut LocalTransform, &ShakeOffset)>) {
    cameras.for_each(|_entity, (transform, offset)| {
        transform.translation += offset.0;
    });
}
