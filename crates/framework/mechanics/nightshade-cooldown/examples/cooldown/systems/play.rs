use crate::resources::Rack;
use crate::theme;
use nightshade::ecs::Storage;
use nightshade::ecs::system_param::{Res, ResMut};
use nightshade_cooldown::commands::{clear, start, start_for};
use nightshade_cooldown::queries::{ready, share};
use nightshade_cooldown::{Cooldown, Recharge};
use nightshade_platform::contract::input::Input;
use nightshade_platform::prelude::KeyCode;
use nightshade_readout::Readout;
use nightshade_scene::contract::LocalTransform;

const PICKS: [KeyCode; 5] = [
    KeyCode::Digit1,
    KeyCode::Digit2,
    KeyCode::Digit3,
    KeyCode::Digit4,
    KeyCode::Digit5,
];

fn fire(storage: &mut Storage, gun: nightshade::prelude::Entity) -> bool {
    let (Some(mut cooldown), Some(recharge)) = (
        storage.get::<Cooldown>(gun).copied(),
        storage.get::<Recharge>(gun).copied(),
    ) else {
        return false;
    };
    if !ready(&cooldown) {
        return false;
    }
    start(&mut cooldown, &recharge);
    storage.set(gun, cooldown);
    true
}

pub fn take_the_orders(input: Res<Input>, rack: Res<Rack>, storage: &mut Storage) {
    let guns = rack.guns.clone();
    for (index, gun) in guns.iter().enumerate() {
        if let Some(key) = PICKS.get(index)
            && input.keyboard.just_pressed(*key)
        {
            fire(storage, *gun);
        }
    }

    if input.keyboard.just_pressed(KeyCode::Space) {
        for gun in guns.iter() {
            fire(storage, *gun);
        }
    }

    if input.keyboard.just_pressed(KeyCode::KeyR) {
        for gun in guns.iter() {
            if let Some(mut cooldown) = storage.get::<Cooldown>(*gun).copied() {
                clear(&mut cooldown);
                storage.set(*gun, cooldown);
            }
        }
    }

    if input.keyboard.just_pressed(KeyCode::KeyF) {
        for gun in guns.iter() {
            if let Some(mut cooldown) = storage.get::<Cooldown>(*gun).copied() {
                start_for(&mut cooldown, theme::SNAP);
                storage.set(*gun, cooldown);
            }
        }
    }
}

pub fn show_the_rack(rack: Res<Rack>, mut readout: ResMut<Readout>, storage: &mut Storage) {
    let mut waiting = 0;
    let mut longest = 0.0f32;
    for gun in rack.guns.iter() {
        let (Some(cooldown), Some(recharge)) = (
            storage.get::<Cooldown>(*gun).copied(),
            storage.get::<Recharge>(*gun).copied(),
        ) else {
            continue;
        };
        if !ready(&cooldown) {
            waiting += 1;
            longest = longest.max(cooldown.0);
        }
        let rest = share(&cooldown, &recharge);
        if let Some(transform) = storage.get_mut::<LocalTransform>(*gun) {
            transform.translation.y = 0.6 + rest * theme::LIFT;
        }
    }

    readout.notes.clear();
    readout
        .notes
        .push(format!("{} of {} waiting", waiting, rack.guns.len()));
    readout.notes.push(format!("longest wait {longest:.2}"));
}
