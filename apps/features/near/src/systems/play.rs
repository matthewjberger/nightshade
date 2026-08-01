use crate::resources::Field;
use crate::theme;
use nalgebra_glm::Vec3;
use nightshade::ecs::Storage;
use nightshade::ecs::system_param::{Res, ResMut};
use nightshade_near::prelude::{NearAnswers, NearAsks, ask_what_is_near, counted_near, nearest_of};
use nightshade_platform::contract::input::Input;
use nightshade_platform::contract::time::Time;
use nightshade_platform::prelude::KeyCode;
use nightshade_readout::Readout;
use nightshade_scene::contract::LocalTransform;

pub fn walk_the_seeker(
    input: Res<Input>,
    time: Res<Time>,
    field: Res<Field>,
    storage: &mut Storage,
) {
    let Some(seeker) = field.seeker else {
        return;
    };
    let mut step = Vec3::new(0.0, 0.0, 0.0);
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
    if step.magnitude() <= 0.0 {
        return;
    }
    let walk = step.normalize() * theme::WALK * time.delta_time;
    if let Some(transform) = storage.get_mut::<LocalTransform>(seeker) {
        transform.translation += walk;
    }
}

pub fn ask_about_the_seeker(field: Res<Field>, mut asks: ResMut<NearAsks>, storage: &mut Storage) {
    let Some(seeker) = field.seeker else {
        return;
    };
    let Some(spot) = storage
        .get::<LocalTransform>(seeker)
        .map(|transform| transform.translation)
    else {
        return;
    };
    ask_what_is_near(&mut asks, theme::SEEKER_TAG, spot, theme::REACH);
}

pub fn take_the_pushes(answers: Res<NearAnswers>, time: Res<Time>, storage: &mut Storage) {
    let delta = time.delta_time;
    let moved: Vec<(nightshade::prelude::Entity, Vec3)> = answers
        .pushes
        .iter()
        .map(|(entity, push)| (*entity, *push))
        .collect();
    for (entity, push) in moved {
        if push.magnitude() <= 0.0 {
            continue;
        }
        if let Some(transform) = storage.get_mut::<LocalTransform>(entity) {
            transform.translation += push * theme::PUSH * delta;
        }
    }
}

pub fn report_the_neighbours(near_answers: Res<NearAnswers>, mut readout: ResMut<Readout>) {
    let (close, closest, pushed) = {
        let answers = &*near_answers;
        (
            counted_near(answers, theme::SEEKER_TAG),
            nearest_of(answers, theme::SEEKER_TAG),
            answers.pushes.len(),
        )
    };
    readout.notes.clear();
    readout
        .notes
        .push(format!("{close} inside {:.0}", theme::REACH));
    match closest {
        Some((_, reach)) => readout.notes.push(format!("nearest at {reach:.2}")),
        None => readout.notes.push("nothing near".to_string()),
    }
    readout.notes.push(format!("{pushed} bodies pushed apart"));
}
