use crate::queries::hill_height;
use crate::resources::Walker;
use crate::theme;
use nalgebra_glm::{Vec3, vec2};
use nightshade::ecs::Storage;
use nightshade::ecs::system_param::{Res, ResMut};
use nightshade::prelude::Query;
use nightshade_ground::Ground;
use nightshade_ground::queries::{answers_height, height_at};
use nightshade_grounded::Grounded;
use nightshade_platform::contract::input::Input;
use nightshade_platform::contract::time::Time;
use nightshade_platform::prelude::KeyCode;
use nightshade_readout::Readout;
use nightshade_scene::contract::LocalTransform;

pub fn take_the_orders(
    input: Res<Input>,
    time: Res<Time>,
    walker: Res<Walker>,
    mut ground: ResMut<Ground>,
    mut bodies: Query<(&mut LocalTransform, &mut Grounded)>,
) {
    if input.keyboard.just_pressed(KeyCode::Space) {
        ground.height_at = match ground.height_at {
            Some(_) => None,
            None => Some(hill_height),
        };
    }
    let Some(body) = walker.body else {
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
    let lower = input.keyboard.just_pressed(KeyCode::KeyQ);
    let raise = input.keyboard.just_pressed(KeyCode::KeyE);
    if step == Vec3::zeros() && !lower && !raise {
        return;
    }
    let walk = if step == Vec3::zeros() {
        Vec3::zeros()
    } else {
        step.normalize() * theme::WALK * time.delta_time
    };
    bodies.for_entity(body, |(transform, grounded)| {
        transform.translation.x += walk.x;
        transform.translation.z += walk.z;
        if lower {
            grounded.0 = (grounded.0 - theme::STAND_STEP).max(theme::STAND_LOW);
        }
        if raise {
            grounded.0 = (grounded.0 + theme::STAND_STEP).min(theme::STAND_HIGH);
        }
    });
}

pub fn watch_the_ground(
    walker: Res<Walker>,
    ground: Res<Ground>,
    mut readout: ResMut<Readout>,
    storage: &mut Storage,
) {
    let body = walker.body;
    let answers = answers_height(&ground);
    let place = body.and_then(|body| storage.get::<LocalTransform>(body).map(|it| it.translation));
    let stand = body.and_then(|body| storage.get::<Grounded>(body).map(|it| it.0));
    let under = place.and_then(|place| height_at(&ground, storage, vec2(place.x, place.z)));

    readout.notes.clear();
    readout.notes.push(
        if answers {
            "ground answers"
        } else {
            "no ground"
        }
        .to_string(),
    );
    if let Some(place) = place {
        readout.notes.push(format!("standing at {:.2}", place.y));
    }
    if let Some(stand) = stand {
        readout.notes.push(format!("stands {stand:.2} above"));
    }
    match under {
        Some(height) => readout.notes.push(format!("ground at {height:.2}")),
        None => readout.notes.push("off the ground".to_string()),
    }
}
