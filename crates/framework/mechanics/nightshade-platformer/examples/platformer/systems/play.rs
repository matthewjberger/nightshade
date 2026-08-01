use crate::resources::Jumper;
use crate::theme;
use nightshade::ecs::Storage;
use nightshade::ecs::system_param::{Res, ResMut};
use nightshade::prelude::Query;
use nightshade_platform::contract::input::Input;
use nightshade_platform::contract::time::Time;
use nightshade_platform::prelude::KeyCode;
use nightshade_platformer::commands::{ask_for_a_jump, spend_the_jump};
use nightshade_platformer::queries::may_jump;
use nightshade_platformer::{Airborne, Coyote, JumpAsked, PlatformerSettings};
use nightshade_readout::Readout;
use nightshade_scene::contract::LocalTransform;

pub fn take_the_orders(
    input: Res<Input>,
    settings: Res<PlatformerSettings>,
    mut jumper: ResMut<Jumper>,
    mut bodies: Query<(&mut JumpAsked,)>,
) {
    let mut lean = 0.0;
    if input.keyboard.is_key_pressed(KeyCode::ArrowLeft) {
        lean -= 1.0;
    }
    if input.keyboard.is_key_pressed(KeyCode::ArrowRight) {
        lean += 1.0;
    }
    jumper.lean = lean;
    if !input.keyboard.just_pressed(KeyCode::Space) {
        return;
    }
    bodies.for_each_mut(|_entity, (asked,)| {
        ask_for_a_jump(asked, &settings);
    });
}

pub fn run_and_jump(
    time: Res<Time>,
    mut jumper: ResMut<Jumper>,
    mut readout: ResMut<Readout>,
    storage: &mut Storage,
) {
    let delta = time.delta_time;
    let Some(walker) = jumper.walker else {
        return;
    };

    let mut asked = storage
        .get::<JumpAsked>(walker)
        .copied()
        .unwrap_or_default();
    let mut coyote = storage.get::<Coyote>(walker).copied().unwrap_or_default();
    let Some(here) = storage
        .get::<LocalTransform>(walker)
        .map(|transform| transform.translation)
    else {
        return;
    };

    let lean = jumper.lean;
    let along = (here.x + lean * theme::RUN * delta).clamp(-theme::REACH + 1.0, theme::REACH - 1.0);
    let over_ledge = (along - theme::LEDGE).abs() <= 3.0;
    let ground = if over_ledge {
        theme::LEDGE_TOP + 1.0
    } else {
        1.0
    };

    let mut leapt = false;
    {
        let jump = &mut *jumper;
        jump.rise -= delta * theme::GRAVITY;
        if may_jump(&coyote, &asked) {
            jump.rise = theme::JUMP;
            spend_the_jump(&mut asked, &mut coyote);
            leapt = true;
        }
    }
    let rise = jumper.rise;

    let mut airborne = true;
    if let Some(transform) = storage.get_mut::<LocalTransform>(walker) {
        transform.translation.x = along;
        transform.translation.y = (here.y + rise * delta).max(ground);
        airborne = transform.translation.y > ground + 0.01;
    }
    if !airborne {
        jumper.rise = 0.0;
    }
    if leapt {
        jumper.leaps += 1;
    }
    storage.set(walker, asked);
    storage.set(walker, coyote);
    storage.set(walker, Airborne(airborne));

    let leaps = jumper.leaps;
    readout.notes.clear();
    readout.notes.push(
        if airborne {
            "in the air"
        } else {
            "on the ground"
        }
        .to_string(),
    );
    readout.notes.push(format!("coyote {:.2}", coyote.0));
    readout.notes.push(format!("buffered {:.2}", asked.0));
    readout.notes.push(format!("{leaps} jumps"));
}
