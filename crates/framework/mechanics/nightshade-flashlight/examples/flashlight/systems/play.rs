use crate::resources::Torch;
use crate::theme;
use nalgebra_glm::{Vec3, quat_angle_axis};
use nightshade::ecs::system_param::EventReader;
use nightshade::prelude::*;
use nightshade_flashlight::commands::{recharge, switch};
use nightshade_flashlight::queries::{brightness, flat, low, remaining, shining};
use nightshade_flashlight::{Battery, CellDied, CellLow, FlashlightSettings, Lit};
use nightshade_platform::contract::input::Input;
use nightshade_platform::contract::time::Time;
use nightshade_platform::prelude::KeyCode;
use nightshade_readout::Readout;
use nightshade_scene::contract::LocalTransform;

pub fn take_the_orders(
    input: Res<Input>,
    settings: Res<FlashlightSettings>,
    torch: Res<Torch>,
    mut torches: Query<(&mut Battery, &mut Lit)>,
) {
    let Some(entity) = torch.entity else {
        return;
    };
    if input.keyboard.just_pressed(KeyCode::KeyF) {
        torches.for_entity(entity, |(battery, lit)| switch(lit, battery));
    }
    if input.keyboard.just_pressed(KeyCode::KeyR) {
        torches.for_entity(entity, |(battery, _lit)| {
            recharge(battery, &settings, theme::CHARGE)
        });
    }
}

pub fn sweep_the_beam(
    time: Res<Time>,
    mut torch: ResMut<Torch>,
    mut torches: Query<(&mut LocalTransform,)>,
) {
    let Some(entity) = torch.entity else {
        return;
    };
    torch.sweep += time.delta_time * theme::SWEEP;
    let sweep = torch.sweep;
    torches.for_entity(entity, |(transform,)| {
        transform.rotation = quat_angle_axis(sweep, &Vec3::y_axis())
            * quat_angle_axis(-std::f32::consts::FRAC_PI_6, &Vec3::x_axis());
    });
}

pub fn hear_the_cell(
    went_low: EventReader<CellLow>,
    died: EventReader<CellDied>,
    mut torch: ResMut<Torch>,
) {
    for _ in went_low.iter() {
        torch.browned_out += 1;
        torch.said = "the beam browns out".to_string();
    }
    for _ in died.iter() {
        torch.deaths += 1;
        torch.said = "the light dies".to_string();
    }
}

pub fn show_the_torch(
    settings: Res<FlashlightSettings>,
    torch: Res<Torch>,
    mut readout: ResMut<Readout>,
    torches: Query<(&Battery, &Lit)>,
) {
    readout.notes.clear();
    torches.for_each(|_entity, (battery, lit)| {
        readout.notes.push(format!(
            "cell {:.0}%",
            remaining(battery, &settings) * 100.0
        ));
        readout
            .notes
            .push(format!("beam {:.2}", brightness(battery, lit, &settings)));
        readout.notes.push(if shining(lit) {
            "switched on".to_string()
        } else if flat(battery) {
            "flat".to_string()
        } else {
            "switched off".to_string()
        });
        if low(battery, &settings) {
            readout.notes.push("running low".to_string());
        }
    });
    readout.notes.push(format!(
        "{} brownouts, {} deaths",
        torch.browned_out, torch.deaths
    ));
    if !torch.said.is_empty() {
        readout.notes.push(torch.said.clone());
    }
}
