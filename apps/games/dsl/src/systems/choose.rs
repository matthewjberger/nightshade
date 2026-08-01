use crate::resources::{Clock, Machine, Script, Store, Studio, Switcher, Tape, Transport};
use crate::systems::setup;
use nightshade::prelude::*;
use nightshade_camera::prelude::ActiveCamera;
use nightshade_platform::prelude::Window;
use nightshade_ui::prelude::*;

pub fn listen(
    frame: Res<RetainedUiFrame>,
    mut switcher: ResMut<Switcher>,
    mut studio: ResMut<Studio>,
    mut clock: ResMut<Clock>,
    mut tape: ResMut<Tape>,
    transport: Res<Transport>,
    mut script: ResMut<Script>,
    mut store: ResMut<Store>,
    mut machine: ResMut<Machine>,
    mut active_camera: ResMut<ActiveCamera>,
    mut window: ResMut<Window>,
    mut commands: Commands,
) {
    let mut wanted = script.wanted.take();
    let mut again = false;
    for event in ui_events(&frame) {
        if let UiEvent::SliderChanged { entity, value } = event
            && transport.scrub == Some(*entity)
            && !tape.frames.is_empty()
        {
            let last = tape.frames.len() - 1;
            tape.seek = Some((value * last as f32).round().clamp(0.0, last as f32) as usize);
            clock.paused = true;
        }
        let UiEvent::ButtonClicked(entity) = event else {
            continue;
        };
        if let Some((name, _)) = switcher.buttons.iter().find(|(_, handle)| handle == entity) {
            wanted = Some(name.clone());
        }
        if transport.play == Some(*entity) {
            clock.paused = false;
            studio.stale = true;
        }
        if transport.pause == Some(*entity) {
            clock.paused = true;
            studio.stale = true;
        }
        if transport.step == Some(*entity) {
            clock.step = true;
        }
        if transport.restart == Some(*entity) {
            again = true;
        }
        if transport.back == Some(*entity) && !tape.frames.is_empty() {
            tape.seek = Some(tape.at().saturating_sub(1));
            clock.paused = true;
            studio.stale = true;
        }
        if transport.ahead == Some(*entity) && !tape.frames.is_empty() {
            let last = tape.frames.len() - 1;
            tape.seek = Some((tape.at() + 1).min(last));
            clock.paused = true;
            studio.stale = true;
        }
    }

    if script.dirty && wanted.is_none() {
        setup::raise(
            &mut store,
            &mut machine,
            &mut script,
            &mut window,
            &mut active_camera,
            &mut commands,
        );
        tape.forget();
        clock.paused = false;
        clock.step = false;
        if studio.tab != 0 {
            studio.stale = true;
        }
        return;
    }

    let wanted = match wanted {
        Some(wanted) => wanted,
        None if again => switcher.playing.clone(),
        None => return,
    };

    setup::play(
        &wanted,
        &mut store,
        &mut machine,
        &mut script,
        &mut window,
        &mut active_camera,
        &mut commands,
    );
    switcher.playing = wanted;
    tape.forget();
    clock.paused = false;
    clock.step = false;
    studio.chosen = None;
    studio.stale = true;
}
