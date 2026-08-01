use crate::data::Ask;
use crate::queries;
use crate::resources::{BreakoutAdvancedResources, Screen};
use crate::systems::screens::activate;
use nightshade::prelude::*;
use nightshade_input::Actions;
use nightshade_platform::prelude::*;
use nightshade_state::prelude::*;
use nightshade_ui::prelude::ui_focus;
use nightshade_ui::resources::RetainedUiInteraction;

const REPEAT_SECONDS: f32 = 0.24;
const STICK_THRESHOLD: f32 = 0.5;

pub fn navigate(
    time: Res<Time>,
    input: Res<Input>,
    actions: Res<Actions<Ask>>,
    screen: Res<State<Screen>>,
    mut resources: ResMut<BreakoutAdvancedResources>,
    mut next: ResMut<NextState<Screen>>,
    mut window: ResMut<Window>,
    mut interaction: ResMut<RetainedUiInteraction>,
    mut commands: Commands,
) {
    if input.input_mode != InputMode::Gamepad {
        return;
    }

    let buttons = queries::screen_buttons(&resources.ui, screen.current);
    let Some(last) = buttons.len().checked_sub(1) else {
        resources.pad.focus = 0;
        resources.pad.repeat = 0.0;
        return;
    };

    let step = menu_step(&actions);
    resources.pad.repeat = (resources.pad.repeat - time.delta_time).max(0.0);
    if step == 0 {
        resources.pad.repeat = 0.0;
    } else if resources.pad.repeat <= 0.0 {
        let count = buttons.len() as i32;
        let moved = resources.pad.focus as i32 + step;
        resources.pad.focus = moved.rem_euclid(count) as usize;
        resources.pad.repeat = REPEAT_SECONDS;
    }

    let focus = resources.pad.focus.min(last);
    resources.pad.focus = focus;
    ui_focus(&mut interaction, buttons[focus]);

    if actions.just_pressed(Ask::Confirm) {
        activate(
            &mut resources,
            &mut next,
            &mut window,
            &mut commands,
            buttons[focus],
        );
    } else if actions.just_pressed(Ask::Back) {
        match screen.current {
            Screen::Paused => next.pending = Some(Screen::Playing),
            Screen::Over => next.pending = Some(Screen::Title),
            Screen::Title | Screen::Playing => {}
        }
    }
}

fn menu_step(actions: &Actions<Ask>) -> i32 {
    if actions.pressed(Ask::MenuDown) {
        return 1;
    }
    if actions.pressed(Ask::MenuUp) {
        return -1;
    }
    let down = -actions.axis2(Ask::Browse).y;
    if down > STICK_THRESHOLD {
        return 1;
    }
    if down < -STICK_THRESHOLD {
        return -1;
    }
    0
}
