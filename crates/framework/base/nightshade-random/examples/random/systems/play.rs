use crate::resources::Scatter;
use crate::theme;
use nightshade::ecs::Storage;
use nightshade::ecs::system_param::{Res, ResMut};
use nightshade_platform::contract::input::Input;
use nightshade_platform::prelude::KeyCode;
use nightshade_random::Random;
use nightshade_random::commands::{chance, disc, ring};
use nightshade_readout::Readout;
use nightshade_scene::contract::LocalTransform;

pub fn take_the_orders(input: Res<Input>, mut scatter: ResMut<Scatter>, mut roll: ResMut<Random>) {
    if input.keyboard.just_pressed(KeyCode::KeyR) {
        *roll = Random::seeded(theme::SEED);
        scatter.asked = true;
    }
    if input.keyboard.just_pressed(KeyCode::KeyQ) {
        scatter.spread = (scatter.spread - theme::NUDGE).max(theme::NARROWEST);
        scatter.asked = true;
    }
    if input.keyboard.just_pressed(KeyCode::KeyE) {
        scatter.spread = (scatter.spread + theme::NUDGE).min(theme::WIDEST);
        scatter.asked = true;
    }
    if input.keyboard.just_pressed(KeyCode::Space) {
        scatter.asked = true;
    }
}

pub fn scatter_the_grains(
    mut scatter: ResMut<Scatter>,
    mut random: ResMut<Random>,
    storage: &mut Storage,
) {
    let (asked, spread, discs, rings) = {
        (
            scatter.asked,
            scatter.spread,
            scatter.discs.clone(),
            scatter.rings.clone(),
        )
    };
    if !asked {
        return;
    }

    let mut heads = 0;
    {
        let roll = &mut *random;
        for grain in &discs {
            let spot = disc(roll, spread);
            if let Some(transform) = storage.get_mut::<LocalTransform>(*grain) {
                transform.translation.x = spot.x;
                transform.translation.z = spot.y;
            }
            if chance(roll, 0.5) {
                heads += 1;
            }
        }
        for grain in &rings {
            let spot = ring(roll, spread);
            if let Some(transform) = storage.get_mut::<LocalTransform>(*grain) {
                transform.translation.x = spot.x;
                transform.translation.z = spot.y;
            }
        }
    }

    scatter.asked = false;
    scatter.heads = heads;
}

pub fn watch_the_scatter(scatter: Res<Scatter>, mut readout: ResMut<Readout>) {
    let (heads, grains, spread) = { (scatter.heads, scatter.discs.len(), scatter.spread) };
    readout.notes.clear();
    readout.notes.push(format!("{heads} of {grains} heads"));
    readout.notes.push(format!("spread {spread:.1}"));
}
