use crate::resources::Warden;
use crate::theme;
use nalgebra_glm::{Vec3, quat_angle_axis, vec3};
use nightshade::prelude::*;
use nightshade_dialogue::Conversation;
use nightshade_dialogue::commands::{choose, offer, open, reveal_all, step};
use nightshade_dialogue::queries::{fully_shown, last_line, shown, talking};
use nightshade_platform::contract::input::Input;
use nightshade_platform::prelude::KeyCode;
use nightshade_readout::Readout;
use nightshade_scene::contract::LocalTransform;

pub fn answer_the_warden(input: Res<Input>, mut conversation: ResMut<Conversation>) {
    let waiting = fully_shown(&conversation) && last_line(&conversation);
    if waiting && input.keyboard.just_pressed(KeyCode::Digit1) {
        choose(&mut conversation, 0);
        return;
    }
    if waiting && input.keyboard.just_pressed(KeyCode::Digit2) {
        choose(&mut conversation, 1);
        return;
    }
    if !input.keyboard.just_pressed(KeyCode::Space) {
        return;
    }
    if conversation.chosen.is_some() {
        open(&mut conversation, theme::SPEAKER, &theme::LINES);
        offer(&mut conversation, &theme::REPLIES);
        return;
    }
    if !fully_shown(&conversation) {
        reveal_all(&mut conversation);
        return;
    }
    if last_line(&conversation) {
        return;
    }
    step(&mut conversation);
}

pub fn turn_the_warden(
    warden: Res<Warden>,
    conversation: Res<Conversation>,
    mut bodies: Query<(&mut LocalTransform,)>,
) {
    let Some(body) = warden.body else {
        return;
    };
    let lean = if talking(&conversation) && !fully_shown(&conversation) {
        theme::LEAN
    } else {
        0.0
    };
    let (turn, forward) = match conversation.chosen {
        Some(0) => (0.0, theme::STEP),
        Some(_) => (std::f32::consts::PI, 0.0),
        None => (0.0, 0.0),
    };
    bodies.for_entity(body, |(transform,)| {
        transform.translation = vec3(0.0, 1.0, forward);
        transform.rotation = quat_angle_axis(turn, &Vec3::y()) * quat_angle_axis(lean, &Vec3::x());
    });
}

pub fn show_the_reply(conversation: Res<Conversation>, mut readout: ResMut<Readout>) {
    readout.notes.clear();
    if !talking(&conversation) {
        return;
    }
    readout.notes.push(format!(
        "{} of {} letters",
        shown(&conversation).chars().count(),
        conversation.line.chars().count()
    ));
    if !(fully_shown(&conversation) && last_line(&conversation)) {
        return;
    }
    for (index, reply) in conversation.replies.iter().enumerate() {
        let mark = if conversation.chosen == Some(index) {
            "answered"
        } else {
            "open"
        };
        readout.notes.push(format!("{reply} ({mark})"));
    }
}
