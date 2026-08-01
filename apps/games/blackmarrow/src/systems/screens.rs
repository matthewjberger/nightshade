use crate::commands::lock_cursor;
use crate::data::ask::Ask;
use crate::data::notes::{NOTE_ROWS, NOTES};
use crate::data::rules::MAX_HEALTH;
use crate::data::theme;
use crate::resources::{BlackmarrowResources, Screen, ScreenHandles};
use crate::tags::{Eyes, MenuCamera};
use nightshade::prelude::*;
use nightshade_camera::prelude::ActiveCamera;
use nightshade_input::Actions;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::LocalTransform;
use nightshade_ui::prelude::*;
use nightshade_ui::resources::{RetainedUiDirty, RetainedUiFrame};
use nightshade_ui_menu::prelude::*;

const PANEL_WIDTH: f32 = 440.0;
const TITLE_HEIGHT: f32 = 268.0;
const PAUSE_HEIGHT: f32 = 188.0;
const ENDING_HEIGHT: f32 = 214.0;
const NOTE_WIDTH: f32 = 660.0;
const NOTE_HEIGHT: f32 = 480.0;

pub fn build(
    tree: &mut UiTreeBuilder,
    style: &MenuStyle,
    menus: &mut Menus<Screen>,
) -> ScreenHandles {
    let title = add_menu(
        tree,
        style,
        PANEL_WIDTH,
        TITLE_HEIGHT,
        theme::TITLE,
        &[
            "The pumps stopped nine days ago and nobody came up.",
            "Your hands open every door in this place.",
            "The pack got in through the shaft. You cannot fight it.",
        ],
        &["Go down", "Quit"],
    );
    let pause = add_menu(
        tree,
        style,
        PANEL_WIDTH,
        PAUSE_HEIGHT,
        "Held",
        &["Escape to go back."],
        &["Resume", "Quit to title"],
    );
    let dead = add_menu(
        tree,
        style,
        PANEL_WIDTH,
        ENDING_HEIGHT,
        "It had you first",
        &["The station keeps what it catches."],
        &["Go down again", "Quit to title"],
    );
    let escaped = add_menu(
        tree,
        style,
        PANEL_WIDTH,
        ENDING_HEIGHT,
        "The shaft head",
        &[
            "Cold air off the ice, and a ladder that goes up.",
            "Whatever was down there is still down there.",
        ],
        &["Quit to title"],
    );

    let first = &NOTES[0];
    let rows: Vec<&str> = (0..NOTE_ROWS)
        .map(|row| first.lines.get(row).copied().unwrap_or(""))
        .collect();
    let note = add_menu(
        tree,
        style,
        NOTE_WIDTH,
        NOTE_HEIGHT,
        first.title,
        &rows,
        &["Put it down"],
    );
    let mut note_rows = [Entity::default(); NOTE_ROWS];
    for (slot, entity) in note_rows.iter_mut().zip(&note.rows) {
        *slot = *entity;
    }

    menu_show_on(menus, Screen::Title, title.panel);
    menu_show_on(menus, Screen::Reading, note.panel);
    menu_show_on(menus, Screen::Paused, pause.panel);
    menu_show_on(menus, Screen::Dead, dead.panel);
    menu_show_on(menus, Screen::Escaped, escaped.panel);

    ScreenHandles {
        play_button: title.buttons[0],
        title_quit_button: title.buttons[1],
        resume_button: pause.buttons[0],
        pause_title_button: pause.buttons[1],
        dead_again_button: dead.buttons[0],
        dead_title_button: dead.buttons[1],
        escaped_title_button: escaped.buttons[0],
        note_heading: note.heading,
        note_rows,
        note_close_button: note.buttons[0],
    }
}

pub fn watch_the_keys(actions: Res<Actions<Ask>>, mut resources: ResMut<BlackmarrowResources>) {
    let escape = actions.just_pressed(Ask::Pause);
    let reading = resources.screen == Screen::Reading;
    if !escape && !(reading && actions.just_pressed(Ask::Pocket)) {
        return;
    }
    resources.pending_screen = match resources.screen {
        Screen::Playing => Some(Screen::Paused),
        Screen::Paused | Screen::Reading => Some(Screen::Playing),
        _ => None,
    };
    if reading {
        resources.run.reading = None;
    }
}

pub fn drive_the_menus(
    frame: Res<RetainedUiFrame>,
    mut resources: ResMut<BlackmarrowResources>,
    mut window: ResMut<Window>,
) {
    let handles = resources.screens;
    for entity in ui_button_clicks(&frame) {
        if entity == handles.play_button || entity == handles.resume_button {
            resources.pending_screen = Some(Screen::Playing);
        } else if entity == handles.note_close_button {
            resources.run.reading = None;
            resources.pending_screen = Some(Screen::Playing);
        } else if entity == handles.dead_again_button {
            resources.player.health = MAX_HEALTH;
            resources.pending_screen = Some(Screen::Playing);
        } else if entity == handles.pause_title_button
            || entity == handles.dead_title_button
            || entity == handles.escaped_title_button
        {
            resources.pending_screen = Some(Screen::Title);
        } else if entity == handles.title_quit_button {
            window.should_exit = true;
        }
    }
}

pub fn settle(
    mut resources: ResMut<BlackmarrowResources>,
    mut window: ResMut<Window>,
    mut dirty: ResMut<RetainedUiDirty>,
    mut next: ResMut<nightshade_state::prelude::NextState<Screen>>,
    mut active: ResMut<ActiveCamera>,
    mut commands: Commands,
    mut eyes: Query<(&LocalTransform,), WithTag<Eyes>>,
    mut menu_cameras: Query<(&LocalTransform,), WithTag<MenuCamera>>,
) {
    let Some(screen) = resources.pending_screen.take() else {
        return;
    };
    if screen == resources.screen {
        return;
    }
    resources.screen = screen;
    next.pending = Some(screen);
    lock_cursor(&mut window, screen == Screen::Playing);

    if screen == Screen::Reading
        && let Some(index) = resources.run.reading
    {
        let note = &NOTES[index % NOTES.len()];
        let handles = resources.screens;
        ui_set_text(&mut commands, handles.note_heading, note.title);
        for (row, entity) in handles.note_rows.iter().enumerate() {
            ui_set_text(
                &mut commands,
                *entity,
                note.lines.get(row).copied().unwrap_or(""),
            );
        }
    }

    let mut seat = None;
    if screen == Screen::Playing || screen == Screen::Reading {
        if let Some(eye) = resources.player.eyes {
            eyes.for_entity(eye, |_| seat = Some(eye));
        }
    } else {
        menu_cameras.for_each_mut(|entity, _| seat = Some(entity));
    }
    if let Some(camera) = seat {
        active.0 = Some(camera);
    }

    let root = resources.hud.root;
    if root != Entity::default() {
        ui_set_visible(&mut dirty, &mut commands, root, screen == Screen::Playing);
    }
}
