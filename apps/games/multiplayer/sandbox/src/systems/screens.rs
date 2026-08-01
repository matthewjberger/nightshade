use crate::data::Ask;
use crate::resources::{Screen, Session, UiHandles};
use nightshade::prelude::*;
use nightshade_input::Actions;
use nightshade_platform::commands::window::set_pointer_grab;
use nightshade_platform::prelude::Window;
use nightshade_ui::prelude::{UiButtonData, ui_set_visible};
use nightshade_ui::resources::RetainedUiDirty;

pub fn show(
    session: Res<Session>,
    handles: Res<UiHandles>,
    mut dirty: ResMut<RetainedUiDirty>,
    mut window: ResMut<Window>,
    mut commands: Commands,
) {
    let screen = session.screen;
    ui_set_visible(
        &mut dirty,
        &mut commands,
        handles.title_root,
        screen == Screen::Title,
    );
    ui_set_visible(
        &mut dirty,
        &mut commands,
        handles.pause_root,
        screen == Screen::Paused,
    );
    ui_set_visible(
        &mut dirty,
        &mut commands,
        handles.hud_root,
        screen != Screen::Title,
    );

    let driving = screen == Screen::Playing && !session.typing && window.is_focused;
    if window.cursor_locked != driving {
        set_pointer_grab(&mut window, driving);
    }
}

pub fn watch_the_keys(actions: Res<Actions<Ask>>, mut session: ResMut<Session>) {
    if session.typing {
        return;
    }
    if !actions.just_pressed(Ask::Pause) {
        return;
    }
    session.screen = match session.screen {
        Screen::Title => Screen::Title,
        Screen::Playing => Screen::Paused,
        Screen::Paused => Screen::Playing,
    };
}

pub fn watch_for_the_exit(
    actions: Res<Actions<Ask>>,
    session: Res<Session>,
    mut window: ResMut<Window>,
) {
    if session.typing || session.screen == Screen::Playing {
        return;
    }
    if actions.just_pressed(Ask::Quit) {
        window.should_exit = true;
    }
}

pub fn buttons(
    handles: Res<UiHandles>,
    mut session: ResMut<Session>,
    mut window: ResMut<Window>,
    mut buttons: Query<(&UiButtonData,)>,
) {
    let mut clicked = |entity: Option<Entity>| {
        let mut hit = false;
        if let Some(entity) = entity {
            buttons.for_entity(entity, |(data,)| hit = data.clicked);
        }
        hit
    };

    if clicked(handles.title_play) {
        session.screen = Screen::Playing;
    }
    if clicked(handles.pause_resume) {
        session.screen = Screen::Playing;
    }
    if clicked(handles.title_quit) || clicked(handles.pause_quit) {
        window.should_exit = true;
    }
}
