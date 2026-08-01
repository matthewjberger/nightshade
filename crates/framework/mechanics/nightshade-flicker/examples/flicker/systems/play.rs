use crate::resources::Street;
use crate::theme;
use nightshade::prelude::*;
use nightshade_flicker::commands::{douse, steady};
use nightshade_flicker::queries::{dark, intensity, level};
use nightshade_flicker::{FlickerBase, FlickerSettings, Flickering};
use nightshade_platform::contract::input::Input;
use nightshade_platform::prelude::KeyCode;
use nightshade_readout::Readout;

pub fn take_the_orders(
    input: Res<Input>,
    mut settings: ResMut<FlickerSettings>,
    mut street: ResMut<Street>,
    mut lamps: Query<(&mut Flickering,)>,
) {
    if input.keyboard.just_pressed(KeyCode::ArrowUp) {
        settings.depth = (settings.depth + theme::STEP).min(1.0);
    }
    if input.keyboard.just_pressed(KeyCode::ArrowDown) {
        settings.depth = (settings.depth - theme::STEP).max(0.0);
    }
    if input.keyboard.just_pressed(KeyCode::Space) {
        street.doused += 1;
        for lamp in street.lamps.clone() {
            lamps.for_entity(lamp, |(flickering,)| douse(flickering));
        }
    }
    if input.keyboard.just_pressed(KeyCode::KeyR) {
        for lamp in street.lamps.clone() {
            lamps.for_entity(lamp, |(flickering,)| steady(flickering));
        }
    }
}

pub fn show_the_street(
    settings: Res<FlickerSettings>,
    street: Res<Street>,
    mut readout: ResMut<Readout>,
    lamps: Query<(&FlickerBase, &Flickering)>,
) {
    let mut out = 0;
    let mut brightest = 0.0_f32;
    let mut lowest = 1.0_f32;
    lamps.for_each(|_entity, (base, flickering)| {
        if dark(flickering, &settings) {
            out += 1;
        }
        brightest = brightest.max(intensity(base, flickering));
        lowest = lowest.min(level(flickering));
    });
    readout.notes.clear();
    readout.notes.push(format!("{out} lamps are dark"));
    readout.notes.push(format!("brightest {brightest:.0}"));
    readout.notes.push(format!("lowest level {lowest:.2}"));
    readout.notes.push(format!("depth {:.2}", settings.depth));
    readout.notes.push(format!("{} blackouts", street.doused));
}
