use crate::data::ask::Ask;
use crate::resources::{GridlockResources, Screen};
use nightshade::prelude::*;
use nightshade_input::Actions;
use nightshade_platform::prelude::Window;
use nightshade_state::prelude::*;
use nightshade_ui::prelude::*;
use nightshade_ui::resources::{RetainedUiDirty, RetainedUiFrame};

pub fn watch_the_keys(
    actions: Res<Actions<Ask>>,
    resources: Res<GridlockResources>,
    screen: Res<State<Screen>>,
    mut next: ResMut<NextState<Screen>>,
) {
    if !actions.just_pressed(Ask::Menu) {
        return;
    }
    match screen.current {
        Screen::Playing => next.pending = Some(Screen::Paused),
        Screen::Paused => next.pending = Some(Screen::Playing),
        Screen::Settings => next.pending = Some(resources.settings_from),
        Screen::Title => {}
    }
}

pub fn drive_the_menus(
    mut resources: ResMut<GridlockResources>,
    frame: Res<RetainedUiFrame>,
    screen: Res<State<Screen>>,
    mut next: ResMut<NextState<Screen>>,
    mut window: ResMut<Window>,
) {
    let handles = resources.screens;
    for entity in ui_button_clicks(&frame) {
        let clicked = |button: Option<Entity>| button == Some(entity);
        if clicked(handles.play_button) || clicked(handles.resume_button) {
            next.pending = Some(Screen::Playing);
        } else if clicked(handles.pause_title_button) {
            next.pending = Some(Screen::Title);
        } else if clicked(handles.title_settings_button) || clicked(handles.pause_settings_button) {
            resources.settings_from = screen.current;
            next.pending = Some(Screen::Settings);
        } else if clicked(handles.settings_back_button) {
            next.pending = Some(resources.settings_from);
        } else if clicked(handles.title_quit_button) {
            window.should_exit = true;
        }
    }
}

pub fn show_hud(
    resources: Res<GridlockResources>,
    mut dirty: ResMut<RetainedUiDirty>,
    mut commands: Commands,
) {
    if let Some(root) = resources.hud.root {
        ui_set_visible(&mut dirty, &mut commands, root, true);
    }
}

pub fn hide_hud(
    resources: Res<GridlockResources>,
    mut dirty: ResMut<RetainedUiDirty>,
    mut commands: Commands,
) {
    if let Some(root) = resources.hud.root {
        ui_set_visible(&mut dirty, &mut commands, root, false);
    }
}

pub fn show_settings(
    resources: Res<GridlockResources>,
    mut dirty: ResMut<RetainedUiDirty>,
    mut commands: Commands,
) {
    if let Some(panel) = resources.hud.settings {
        ui_set_visible(&mut dirty, &mut commands, panel, true);
    }
}

pub fn hide_settings(
    resources: Res<GridlockResources>,
    mut dirty: ResMut<RetainedUiDirty>,
    mut commands: Commands,
) {
    if let Some(panel) = resources.hud.settings {
        ui_set_visible(&mut dirty, &mut commands, panel, false);
    }
}
