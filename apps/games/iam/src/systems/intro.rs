use crate::data::machine::{
    FROM_BLACK_SECONDS, HOLD_BOOTING, HOLD_LOGO, HOLD_STARTED, Intro, TO_BLACK_SECONDS,
};
use crate::resources::AppResources;
use nightshade::prelude::*;
use nightshade_platform::prelude::{Input, MouseState, Time};
use nightshade_ui::prelude::*;

const WAITING: &str = "click to start";

pub fn fade(resources: &AppResources) -> (f32, f32) {
    let elapsed = resources.boot.intro_elapsed;
    match resources.boot.intro {
        Intro::Waiting | Intro::Booting | Intro::Started | Intro::FromBlack | Intro::Done => {
            (0.0, 0.0)
        }
        Intro::Playing => (1.0, 1.0),
        Intro::ToBlack => (1.0 - (elapsed / TO_BLACK_SECONDS).min(1.0), 1.0),
    }
}

pub fn covered(resources: &AppResources) -> bool {
    !matches!(resources.boot.intro, Intro::Done)
}

pub fn reached_for(resources: &AppResources) -> bool {
    !matches!(resources.boot.intro, Intro::Waiting)
}

pub fn starting(resources: &AppResources) -> bool {
    !matches!(resources.boot.intro, Intro::Done)
}

pub fn advance(
    mut resources: ResMut<AppResources>,
    time: Res<Time>,
    input: Res<Input>,
    mut dirty: ResMut<RetainedUiDirty>,
    mut commands: Commands,
) {
    let resources = &mut *resources;
    if resources.boot.intro == Intro::Done {
        return;
    }
    resources.boot.intro_elapsed += time.delta_time;
    let elapsed = resources.boot.intro_elapsed;
    let asked = asked_for_the_machine(&input);

    match resources.boot.intro {
        Intro::Waiting => {
            if asked {
                move_to(resources, Intro::Playing);
            }
        }
        Intro::Playing => {
            if asked && elapsed >= HOLD_LOGO {
                move_to(resources, Intro::ToBlack);
            }
        }
        Intro::ToBlack => {
            if elapsed >= TO_BLACK_SECONDS {
                move_to(resources, Intro::Booting);
            }
        }
        Intro::Booting => {
            let done = elapsed >= crate::data::boot::duration();
            if done || (asked && elapsed >= HOLD_BOOTING) {
                move_to(resources, Intro::Started);
                resources.session.cues.push(crate::resources::Cue::Boot);
            }
        }
        Intro::Started => {
            if elapsed >= HOLD_STARTED {
                move_to(resources, Intro::FromBlack);
            }
        }
        Intro::FromBlack => {
            if elapsed >= FROM_BLACK_SECONDS {
                move_to(resources, Intro::Done);
                resources.redraw.machine = true;
            }
        }
        Intro::Done => {}
    }

    let waiting = resources.boot.intro == Intro::Waiting;
    ui_set_visible(
        &mut dirty,
        &mut commands,
        resources.interface.waiting,
        waiting,
    );
    if waiting {
        ui_set_text(&mut commands, resources.interface.waiting_text, WAITING);
    }

    let booting = matches!(
        resources.boot.intro,
        Intro::Booting | Intro::Started | Intro::FromBlack
    );
    ui_set_visible(&mut dirty, &mut commands, resources.interface.boot, booting);
    match resources.boot.intro {
        Intro::Booting => {
            let shown = crate::data::boot::shown(resources.boot.intro_elapsed);
            ui_set_text(&mut commands, resources.interface.boot_text, &shown);
        }
        Intro::Started => {
            let shown = crate::data::boot::shown(crate::data::boot::duration());
            ui_set_text(&mut commands, resources.interface.boot_text, &shown);
        }
        _ => {}
    }
}

fn move_to(resources: &mut AppResources, stage: Intro) {
    resources.boot.intro = stage;
    resources.boot.intro_elapsed = 0.0;
}

fn asked_for_the_machine(input: &Input) -> bool {
    if input.mouse.state.contains(MouseState::LEFT_JUST_PRESSED) {
        return true;
    }
    !input.keyboard.just_pressed_keys.is_empty()
}
