use crate::data::Ask;
use crate::resources::{OrreryResources, Screen};
use nightshade::prelude::*;
use nightshade_graphics::prelude::*;
use nightshade_input::Actions;
use nightshade_platform::prelude::*;
use nightshade_state::prelude::*;
use nightshade_ui::prelude::*;
use nightshade_ui::resources::{RetainedUiDirty, RetainedUiFrame};

pub fn read_state(mut orrery_resources: ResMut<OrreryResources>, state_now: Res<State<Screen>>) {
    let now = state_now.current;
    orrery_resources.screen = now;
}

pub fn apply_request(
    mut orrery_resources: ResMut<OrreryResources>,
    mut state_next: ResMut<NextState<Screen>>,
) {
    let wanted = orrery_resources.wanted.take();
    if let Some(wanted) = wanted {
        state_next.pending = Some(wanted);
    }
}

pub fn drive(
    actions: Res<Actions<Ask>>,
    frame: Res<RetainedUiFrame>,
    mut resources: ResMut<OrreryResources>,
    mut window: ResMut<Window>,
) {
    let handles = resources.hud;
    let screen = resources.screen;
    let clicks: Vec<Entity> = ui_button_clicks(&frame).collect();
    let pressed = |button: Entity| button != Entity::default() && clicks.contains(&button);

    match screen {
        Screen::Title => {
            if pressed(handles.fly_button) {
                resources.wanted = Some(Screen::Playing);
            } else if pressed(handles.title_exit_button) {
                window.should_exit = true;
            }
        }
        Screen::Playing => {
            if actions.just_pressed(Ask::Pause) {
                resources.wanted = Some(Screen::Paused);
            }
        }
        Screen::Paused => {
            if pressed(handles.resume_button) || actions.just_pressed(Ask::Pause) {
                resources.wanted = Some(Screen::Playing);
            } else if pressed(handles.title_return_button) {
                resources.wanted = Some(Screen::Title);
            } else if pressed(handles.pause_exit_button) {
                window.should_exit = true;
            }
        }
    }
}

pub fn show_screen(
    mut resources: ResMut<OrreryResources>,
    mut dirty: ResMut<RetainedUiDirty>,
    mut commands: Commands,
) {
    if resources.shown == Some(resources.screen) {
        return;
    }
    let screen = resources.screen;
    resources.shown = Some(screen);
    if screen == Screen::Paused {
        crate::systems::hud::show_log(&mut resources, &mut commands);
    }
    let handles = resources.hud;
    ui_set_visible(
        &mut dirty,
        &mut commands,
        handles.title_panel,
        screen == Screen::Title,
    );
    ui_set_visible(
        &mut dirty,
        &mut commands,
        handles.pause_panel,
        screen == Screen::Paused,
    );
    ui_set_visible(
        &mut dirty,
        &mut commands,
        handles.readout,
        screen == Screen::Playing,
    );
    ui_set_visible(
        &mut dirty,
        &mut commands,
        handles.prompt,
        screen == Screen::Playing,
    );
}

pub fn hold_cursor(mut resources: ResMut<OrreryResources>, mut window: ResMut<Window>) {
    if window.handle.is_none() {
        return;
    }
    let wanted = resources.screen == Screen::Playing;
    if resources.cursor_applied == Some(wanted) {
        return;
    }
    set_cursor_locked(&mut window, wanted);
    set_cursor_visible(&mut window, !wanted);
    resources.cursor_applied = Some(wanted);
}
