use crate::resources::Line;
use crate::theme;
use nalgebra_glm::Vec3;
use nightshade::prelude::*;
use nightshade_follow::{FollowOffset, FollowSettings, Following};
use nightshade_platform::contract::input::Input;
use nightshade_platform::contract::time::Time;
use nightshade_platform::prelude::KeyCode;
use nightshade_readout::Readout;
use nightshade_scene::contract::LocalTransform;

pub fn take_the_orders(
    input: Res<Input>,
    time: Res<Time>,
    line: Res<Line>,
    mut settings: ResMut<FollowSettings>,
    mut bodies: Query<(&mut LocalTransform,)>,
) {
    if input.keyboard.just_pressed(KeyCode::Space) {
        settings.aim = !settings.aim;
    }
    if input.keyboard.just_pressed(KeyCode::KeyQ) {
        settings.rate = (settings.rate - theme::RATE_STEP).max(theme::SLOWEST);
    }
    if input.keyboard.just_pressed(KeyCode::KeyE) {
        settings.rate = (settings.rate + theme::RATE_STEP).min(theme::FASTEST);
    }
    if input.keyboard.just_pressed(KeyCode::KeyZ) {
        settings.snap = match settings.snap {
            Some(_) => None,
            None => Some(theme::SNAP),
        };
    }

    let Some(lead) = line.lead else {
        return;
    };
    let mut step = Vec3::zeros();
    if input.keyboard.is_key_pressed(KeyCode::ArrowUp) {
        step.z -= 1.0;
    }
    if input.keyboard.is_key_pressed(KeyCode::ArrowDown) {
        step.z += 1.0;
    }
    if input.keyboard.is_key_pressed(KeyCode::ArrowLeft) {
        step.x -= 1.0;
    }
    if input.keyboard.is_key_pressed(KeyCode::ArrowRight) {
        step.x += 1.0;
    }
    if step == Vec3::zeros() {
        return;
    }
    let walk = step.normalize() * theme::WALK * time.delta_time;
    bodies.for_entity(lead, |(transform,)| {
        transform.translation += walk;
    });
}

pub fn watch_the_line(
    line: Res<Line>,
    settings: Res<FollowSettings>,
    mut readout: ResMut<Readout>,
    mut leads: Query<(&LocalTransform,)>,
    followers: Query<(&LocalTransform, &Following, &FollowOffset)>,
) {
    let mut tail = Vec::new();
    followers.for_each(|_entity, (transform, following, offset)| {
        tail.push((transform.translation, following.0, offset.0));
    });

    let mut lag = None;
    if let Some(lead) = line.lead {
        let mut head = None;
        leads.for_entity(lead, |(transform,)| {
            head = Some(transform.translation);
        });
        if let Some(head) = head
            && let Some((place, _, offset)) = tail.iter().find(|(_, target, _)| *target == lead)
        {
            lag = Some((place - (head + offset)).norm());
        }
    }

    readout.notes.clear();
    readout.notes.push(format!("{} in the line", tail.len()));
    readout.notes.push(format!("rate {:.1}", settings.rate));
    readout
        .notes
        .push(format!("aim {}", if settings.aim { "on" } else { "off" }));
    readout.notes.push(match settings.snap {
        Some(step) => format!("snap {step:.2}"),
        None => "no snap".to_string(),
    });
    if let Some(lag) = lag {
        readout.notes.push(format!("first lag {lag:.2}"));
    }
}
