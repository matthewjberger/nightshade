use crate::resources::Stage;
use crate::theme;
use nalgebra_glm::{Vec3, quat_angle_axis};
use nightshade::prelude::*;
use nightshade_lofi::commands::wear;
use nightshade_lofi::queries::{scale, snapping};
use nightshade_lofi::{LofiSettings, clean, handheld, playstation};
use nightshade_platform::contract::input::Input;
use nightshade_platform::contract::time::Time;
use nightshade_platform::prelude::KeyCode;
use nightshade_readout::Readout;
use nightshade_scene::contract::LocalTransform;

pub fn take_the_orders(
    input: Res<Input>,
    mut settings: ResMut<LofiSettings>,
    mut stage: ResMut<Stage>,
) {
    if input.keyboard.just_pressed(KeyCode::Digit1) {
        wear(&mut settings, clean());
        stage.wearing = "clean";
    }
    if input.keyboard.just_pressed(KeyCode::Digit2) {
        wear(&mut settings, playstation());
        stage.wearing = "playstation";
    }
    if input.keyboard.just_pressed(KeyCode::Digit3) {
        wear(&mut settings, handheld());
        stage.wearing = "handheld";
    }
}

pub fn turn_the_ball(
    time: Res<Time>,
    mut stage: ResMut<Stage>,
    mut balls: Query<(&mut LocalTransform,)>,
) {
    let Some(ball) = stage.ball else {
        return;
    };
    stage.spin += time.delta_time * theme::TURN_RATE;
    let spin = stage.spin;
    balls.for_entity(ball, |(transform,)| {
        transform.rotation = quat_angle_axis(spin, &Vec3::y_axis());
    });
}

pub fn show_the_look(settings: Res<LofiSettings>, stage: Res<Stage>, mut readout: ResMut<Readout>) {
    readout.notes.clear();
    readout.notes.push(format!("wearing {}", stage.wearing));
    readout
        .notes
        .push(format!("render scale {:.2}", scale(&settings)));
    readout.notes.push(if snapping(&settings) {
        "vertices snap".to_string()
    } else {
        "vertices are smooth".to_string()
    });
}
