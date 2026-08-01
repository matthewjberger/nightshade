use crate::data::Ask;
use crate::resources::{Hud, Readouts, Screen};
use nightshade::prelude::*;
use nightshade_input::Actions;
use nightshade_platform::prelude::*;
use nightshade_state::prelude::*;
use nightshade_ui::prelude::*;
use nightshade_ui::resources::{RetainedUiDirty, RetainedUiFrame};

pub fn watch_the_keys(
    actions: Res<Actions<Ask>>,
    state_now: Res<State<Screen>>,
    mut state_next: ResMut<NextState<Screen>>,
) {
    if !actions.just_pressed(Ask::Pause) {
        return;
    }
    match state_now.current {
        Screen::Playing => state_next.pending = Some(Screen::Paused),
        Screen::Paused => state_next.pending = Some(Screen::Playing),
        _ => {}
    }
}

pub fn drive_the_title_menu(
    hud: Res<Hud>,
    retained_ui_frame: Res<RetainedUiFrame>,
    mut window: ResMut<Window>,
    mut state_next: ResMut<NextState<Screen>>,
) {
    let hud = *hud;
    let clicked: Vec<Entity> = ui_button_clicks(&retained_ui_frame).collect();
    for entity in clicked {
        if entity == hud.start_button {
            state_next.pending = Some(Screen::Playing);
        } else if entity == hud.title_quit_button {
            window.should_exit = true;
        }
    }
}

pub fn drive_the_pause_menu(
    hud: Res<Hud>,
    retained_ui_frame: Res<RetainedUiFrame>,
    mut window: ResMut<Window>,
    mut state_next: ResMut<NextState<Screen>>,
) {
    let hud = *hud;
    let clicked: Vec<Entity> = ui_button_clicks(&retained_ui_frame).collect();
    for entity in clicked {
        if entity == hud.resume_button {
            state_next.pending = Some(Screen::Playing);
        } else if entity == hud.quit_button {
            window.should_exit = true;
        }
    }
}

pub fn hold_the_cursor(
    mut readouts: ResMut<Readouts>,
    mut window: ResMut<Window>,
    state_now: Res<State<Screen>>,
) {
    let screen = state_now.current;
    let free = screen != Screen::Playing;
    if readouts.cursor_free == free {
        return;
    }
    readouts.cursor_free = free;
    window.show_cursor = free;
    set_cursor_locked(&mut window, !free);
    set_cursor_visible(&mut window, free);
}

pub fn follow_the_screen(
    hud: Res<Hud>,
    mut readouts: ResMut<Readouts>,
    mut retained_ui_dirty: ResMut<RetainedUiDirty>,
    state_now: Res<State<Screen>>,
    mut commands: Commands,
) {
    let hud = *hud;
    if !hud.ready {
        return;
    }
    let screen = state_now.current;
    let title = screen == Screen::Title;
    let paused = screen == Screen::Paused;
    let wanted = [
        (hud.title_panel, title, readouts.title_open),
        (hud.pause_panel, paused, readouts.pause_open),
        (hud.veil, title || paused, readouts.veil_open),
        (hud.place_panel, !title, readouts.hud_open),
        (hud.pocket_panel, !title, readouts.hud_open),
    ];
    let changes: Vec<(Entity, bool)> = wanted
        .iter()
        .filter(|(_, want, shown)| want != shown)
        .map(|(entity, want, _)| (*entity, *want))
        .collect();
    if changes.is_empty() {
        return;
    }
    {
        readouts.title_open = title;
        readouts.pause_open = paused;
        readouts.veil_open = title || paused;
        readouts.hud_open = !title;
    }
    let dirty = &mut *retained_ui_dirty;
    for (entity, visible) in changes {
        ui_set_visible(dirty, &mut commands, entity, visible);
    }
}
