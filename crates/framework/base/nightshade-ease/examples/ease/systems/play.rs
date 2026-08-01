use crate::resources::Run;
use crate::theme;
use nightshade::ecs::Storage;
use nightshade::ecs::system_param::{Res, ResMut};
use nightshade_ease::{approach, damp, lerp, remap, share, smooth_step, smoother_step};
use nightshade_platform::contract::input::Input;
use nightshade_platform::contract::time::Time;
use nightshade_platform::prelude::KeyCode;
use nightshade_readout::Readout;
use nightshade_scene::contract::LocalTransform;

pub fn take_the_orders(input: Res<Input>, time: Res<Time>, mut run: ResMut<Run>) {
    if input.keyboard.just_pressed(KeyCode::KeyR) {
        run.turn = 0.0;
        return;
    }
    if input.keyboard.just_pressed(KeyCode::KeyF) {
        run.turn = 1.0;
        return;
    }
    let step = share(theme::SCRUB, time.delta_time);
    if input.keyboard.is_key_pressed(KeyCode::ArrowRight) {
        run.turn = (run.turn + step).min(1.0);
    }
    if input.keyboard.is_key_pressed(KeyCode::ArrowLeft) {
        run.turn = (run.turn - step).max(0.0);
    }
}

pub fn race_them(
    time: Res<Time>,
    run: Res<Run>,
    mut readout: ResMut<Readout>,
    storage: &mut Storage,
) {
    let delta = time.delta_time;
    let (turn, runners) = { (run.turn, run.runners.clone()) };
    let target = lerp(theme::FROM, theme::TO, turn);

    for (lane, runner) in runners.iter().enumerate() {
        let Some(transform) = storage.get_mut::<LocalTransform>(*runner) else {
            continue;
        };
        let here = transform.translation.x;
        transform.translation.x = match lane {
            0 => target,
            1 => lerp(theme::FROM, theme::TO, smooth_step(turn)),
            2 => lerp(theme::FROM, theme::TO, smoother_step(turn)),
            3 => approach(here, target, theme::STEP * delta),
            _ => damp(here, target, theme::RATE, delta),
        };
    }

    let walked = runners
        .get(3)
        .and_then(|runner| storage.get::<LocalTransform>(*runner))
        .map(|transform| transform.translation.x)
        .unwrap_or(theme::FROM);
    let settled = runners
        .get(4)
        .and_then(|runner| storage.get::<LocalTransform>(*runner))
        .map(|transform| transform.translation.x)
        .unwrap_or(theme::FROM);

    readout.notes.clear();
    readout.notes.push(format!("turn {turn:.2}"));
    readout.notes.push(format!(
        "smooth {:.2}, smoother {:.2}",
        smooth_step(turn),
        smoother_step(turn)
    ));
    readout.notes.push(format!(
        "walked {:.2}, settled {:.2}",
        remap(walked, theme::FROM, theme::TO, 0.0, 1.0),
        remap(settled, theme::FROM, theme::TO, 0.0, 1.0)
    ));
}
