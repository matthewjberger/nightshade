use crate::data;
use crate::data::Ask;
use crate::queries;
use crate::resources::{BreakoutAdvancedResources, Screen, UiHandles};
use nightshade::prelude::*;
use nightshade_input::Actions;
use nightshade_platform::prelude::*;
use nightshade_state::prelude::*;
use nightshade_ui::prelude::*;
use nightshade_ui::resources::{RetainedUiDirty, RetainedUiFrame};
use nightshade_ui_menu::prelude::*;

pub fn build(tree: &mut UiTreeBuilder, style: &MenuStyle, menus: &mut Menus<Screen>) -> UiHandles {
    let mut handles = UiHandles::default();

    let title = add_menu(
        tree,
        style,
        data::TITLE_WIDTH,
        data::TITLE_HEIGHT,
        data::TITLE,
        &[
            "Five levels of brick to clear.",
            "Each one hits back harder.",
        ],
        &[&queries::level_label(0), "Play", "Quit"],
    );
    handles.level_button = title.buttons[0];
    handles.play_button = title.buttons[1];
    handles.title_quit_button = title.buttons[2];

    let pause = add_menu(
        tree,
        style,
        data::MENU_WIDTH,
        data::MENU_HEIGHT,
        "Paused",
        &[],
        &["Resume", "Restart", "Quit to title"],
    );
    handles.resume_button = pause.buttons[0];
    handles.restart_button = pause.buttons[1];
    handles.pause_title_button = pause.buttons[2];

    let over = add_menu_panel(tree, style, data::OVER_WIDTH, data::OVER_HEIGHT);
    tree.in_parent(over, |tree| {
        handles.over_heading = add_menu_heading(tree, style, "Game over");
        handles.over_summary = add_menu_row(tree, style, "Score 0");
        handles.again_button = tree.add_button("Play again");
        handles.over_title_button = tree.add_button("Quit to title");
    });

    menu_show_on(menus, Screen::Title, title.panel);
    menu_show_on(menus, Screen::Paused, pause.panel);
    menu_show_on(menus, Screen::Over, over);

    handles
}

pub fn watch_the_keys(
    actions: Res<Actions<Ask>>,
    screen: Res<State<Screen>>,
    mut next: ResMut<NextState<Screen>>,
    mut window: ResMut<Window>,
) {
    if !actions.just_pressed(Ask::Pause) {
        return;
    }
    match screen.current {
        Screen::Playing => next.pending = Some(Screen::Paused),
        Screen::Paused => next.pending = Some(Screen::Playing),
        Screen::Over => next.pending = Some(Screen::Title),
        Screen::Title => window.should_exit = true,
    }
}

pub fn drive_the_menus(
    mut resources: ResMut<BreakoutAdvancedResources>,
    frame: Res<RetainedUiFrame>,
    mut next: ResMut<NextState<Screen>>,
    mut window: ResMut<Window>,
    mut commands: Commands,
) {
    let clicked: Vec<Entity> = ui_button_clicks(&frame).collect();
    for entity in clicked {
        activate(
            &mut resources,
            &mut next,
            &mut window,
            &mut commands,
            entity,
        );
    }
}

pub fn activate(
    resources: &mut BreakoutAdvancedResources,
    next: &mut NextState<Screen>,
    window: &mut Window,
    commands: &mut Commands,
    entity: Entity,
) {
    let handles = resources.ui;
    if entity == handles.level_button {
        crate::commands::choose_next_level(&mut resources.round);
        ui_set_text(
            commands,
            handles.level_button,
            &queries::level_label(resources.round.chosen),
        );
    } else if entity == handles.play_button
        || entity == handles.again_button
        || entity == handles.restart_button
    {
        crate::commands::restart(&mut resources.round);
        next.pending = Some(Screen::Playing);
    } else if entity == handles.resume_button {
        next.pending = Some(Screen::Playing);
    } else if entity == handles.pause_title_button || entity == handles.over_title_button {
        next.pending = Some(Screen::Title);
    } else if entity == handles.title_quit_button {
        window.should_exit = true;
    }
}

pub fn show_hud(
    resources: Res<BreakoutAdvancedResources>,
    mut dirty: ResMut<RetainedUiDirty>,
    mut commands: Commands,
) {
    set_hud(&resources, &mut dirty, &mut commands, true);
}

pub fn hide_hud(
    resources: Res<BreakoutAdvancedResources>,
    mut dirty: ResMut<RetainedUiDirty>,
    mut commands: Commands,
) {
    set_hud(&resources, &mut dirty, &mut commands, false);
}

fn set_hud(
    resources: &BreakoutAdvancedResources,
    dirty: &mut RetainedUiDirty,
    commands: &mut Commands,
    visible: bool,
) {
    let Some(readout) = resources.ui.readout else {
        return;
    };
    ui_set_visible(dirty, commands, readout, visible);
}

pub fn menu_style() -> MenuStyle {
    MenuStyle {
        background: Vec4::from_column_slice(&data::PANEL_BACKGROUND),
        border: Vec4::from_column_slice(&data::PANEL_BORDER),
        row_color: Vec4::from_column_slice(&data::PANEL_TEXT),
        centered: true,
        corner_radius: 10.0,
        border_width: 1.0,
        padding: 18.0,
        spacing: 10.0,
        ..Default::default()
    }
}
