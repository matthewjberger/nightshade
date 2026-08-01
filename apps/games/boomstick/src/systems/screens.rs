use crate::data::Ask;
use crate::data::levels::{LEVEL_COUNT, plan};
use crate::queries::{clock_line, share};
use crate::resources::{BoomstickResources, Kit, Record, Screen, ScreenHandles};
use crate::tags::LevelPiece;
use nightshade::prelude::*;
use nightshade_camera::resources::ActiveCamera;
use nightshade_input::Actions;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::LocalTransform;
use nightshade_state::prelude::*;
use nightshade_ui::prelude::*;
use nightshade_ui::resources::{RetainedUiDirty, RetainedUiFrame};
use nightshade_ui_menu::prelude::*;

const PANEL_WIDTH: f32 = 460.0;
const TITLE_HEIGHT: f32 = 250.0;
const PAUSE_HEIGHT: f32 = 240.0;
const PAGE_HEIGHT: f32 = 290.0;
const ROW_COUNT: usize = 3;
const EMPTY_ROWS: [&str; ROW_COUNT] = ["", "", ""];

pub fn style() -> MenuStyle {
    MenuStyle {
        background: Vec4::new(0.05, 0.04, 0.05, 0.90),
        border: Vec4::new(1.0, 0.56, 0.28, 0.34),
        corner_radius: 10.0,
        border_width: 1.0,
        padding: 22.0,
        spacing: 10.0,
        visible: false,
        heading_size: 32.0,
        heading_color: Vec4::new(1.0, 0.72, 0.40, 1.0),
        centered: true,
        row_size: 17.0,
        row_color: Vec4::new(0.80, 0.82, 0.88, 1.0),
        ..Default::default()
    }
}

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
        "BOOMSTICK",
        &["Six floors of the foundry, and every one of them wants you."],
        &["Start the shift", "Quit"],
    );
    let paused = add_menu(
        tree,
        style,
        PANEL_WIDTH,
        PAUSE_HEIGHT,
        "Paused",
        &[],
        &["Back to it", "Start the floor again", "Quit to the title"],
    );
    let tally = add_menu(
        tree,
        style,
        PANEL_WIDTH,
        PAGE_HEIGHT,
        "Floor cleared",
        &EMPTY_ROWS,
        &["Down the stair"],
    );
    let ending = add_menu(
        tree,
        style,
        PANEL_WIDTH,
        PAGE_HEIGHT,
        "The foundry is quiet",
        &EMPTY_ROWS,
        &["Back to the title"],
    );
    let tally_heading = tally.heading;
    let tally_rows = rows_of(&tally);
    let ending_heading = ending.heading;
    let ending_rows = rows_of(&ending);

    menu_show_on(menus, Screen::Title, title.panel);
    menu_show_on(menus, Screen::Paused, paused.panel);
    menu_show_on(menus, Screen::Tally, tally.panel);
    menu_show_on(menus, Screen::Ending, ending.panel);

    ScreenHandles {
        start: title.buttons[0],
        quit: title.buttons[1],
        resume: paused.buttons[0],
        restart: paused.buttons[1],
        abandon: paused.buttons[2],
        carry_on: tally.buttons[0],
        stand_down: ending.buttons[0],
        tally_heading,
        tally_rows,
        ending_heading,
        ending_rows,
    }
}

fn rows_of(menu: &MenuHandles) -> [Entity; ROW_COUNT] {
    let mut rows = [Entity::default(); ROW_COUNT];
    for (row, found) in rows.iter_mut().zip(menu.rows.iter()) {
        *row = *found;
    }
    rows
}

pub fn watch_the_keys(
    actions: Res<Actions<Ask>>,
    screen: Res<State<Screen>>,
    mut next: ResMut<NextState<Screen>>,
    mut resources: ResMut<BoomstickResources>,
) {
    let accept = actions.just_pressed(Ask::Confirm);
    match screen.current {
        Screen::Level if actions.just_pressed(Ask::Pause) => next.pending = Some(Screen::Paused),
        Screen::Paused if actions.just_pressed(Ask::Resume) => next.pending = Some(Screen::Level),
        Screen::Tally if accept => next.pending = Some(Screen::Level),
        Screen::Ending if accept => next.pending = Some(Screen::Title),
        Screen::Title if accept => {
            resources.record = Record::default();
            resources.mission.level = 0;
            resources.mission.kit = Kit::default();
            resources.mission.retry = true;
            next.pending = Some(Screen::Level);
        }
        _ => {}
    }
}

pub fn drive_the_menus(
    frame: Res<RetainedUiFrame>,
    mut resources: ResMut<BoomstickResources>,
    mut next: ResMut<NextState<Screen>>,
    mut window: ResMut<Window>,
) {
    let handles = resources.screens;
    for entity in ui_button_clicks(&frame) {
        if entity == handles.start {
            resources.record = Record::default();
            resources.mission.level = 0;
            resources.mission.kit = Kit::default();
            resources.mission.retry = true;
            next.pending = Some(Screen::Level);
        } else if entity == handles.resume {
            next.pending = Some(Screen::Level);
        } else if entity == handles.restart {
            resources.mission.retry = true;
            resources.mission.wanted = true;
            next.pending = Some(Screen::Level);
        } else if entity == handles.abandon || entity == handles.stand_down {
            next.pending = Some(Screen::Title);
        } else if entity == handles.carry_on {
            next.pending = Some(Screen::Level);
        } else if entity == handles.quit {
            window.should_exit = true;
        }
    }
}

pub fn step_the_floor(mut resources: ResMut<BoomstickResources>) {
    resources.mission.level = (resources.mission.level + 1).min(LEVEL_COUNT - 1);
    resources.mission.built = false;
    resources.mission.retry = false;
}

pub fn open_title(
    mut resources: ResMut<BoomstickResources>,
    mut active_camera: ResMut<ActiveCamera>,
    mut commands: Commands,
    pieces: Query<(&LocalTransform,), WithTag<LevelPiece>>,
) {
    pieces.for_each(|entity, _| commands.despawn_recursive(entity));
    active_camera.0 = resources.player.eyes;
    resources.mission.built = false;
    resources.mission.populated = false;
    resources.mission.armed = false;
    resources.mission.level = 0;
}

pub fn show_hud(
    resources: Res<BoomstickResources>,
    mut dirty: ResMut<RetainedUiDirty>,
    mut commands: Commands,
) {
    reveal(&mut dirty, &mut commands, resources.hud.root, true);
}

pub fn hide_hud(
    resources: Res<BoomstickResources>,
    mut dirty: ResMut<RetainedUiDirty>,
    mut commands: Commands,
) {
    reveal(&mut dirty, &mut commands, resources.hud.root, false);
}

pub fn write_the_pages(
    screen: Res<State<Screen>>,
    mut resources: ResMut<BoomstickResources>,
    mut commands: Commands,
) {
    let handles = resources.screens;
    if handles.carry_on == Entity::default() {
        return;
    }
    let key = match screen.current {
        Screen::Tally => format!(
            "tally {} {} {}",
            resources.mission.level, resources.mission.kills, resources.mission.secrets
        ),
        Screen::Ending => format!(
            "ending {} {}",
            resources.record.kills, resources.record.secrets
        ),
        _ => String::new(),
    };
    if key == resources.shown.board {
        return;
    }
    resources.shown.board = key;

    match screen.current {
        Screen::Tally => {
            let mission = &resources.mission;
            let plan = plan(mission.level);
            ui_set_text(
                &mut commands,
                handles.tally_heading,
                &format!("{} cleared", plan.name),
            );
            let lines = [
                format!(
                    "kills   {} of {}   {}%",
                    mission.kills,
                    mission.standing,
                    share(mission.kills, mission.standing)
                ),
                format!(
                    "secrets {} of {}   {}%",
                    mission.secrets,
                    mission.hidden,
                    share(mission.secrets, mission.hidden)
                ),
                format!("time    {}", clock_line(mission.clock)),
            ];
            for (row, line) in handles.tally_rows.iter().zip(lines.iter()) {
                ui_set_text(&mut commands, *row, line);
            }
        }
        Screen::Ending => {
            let record = &resources.record;
            ui_set_text(
                &mut commands,
                handles.ending_heading,
                "The foundry is quiet",
            );
            let lines = [
                format!("kills   {}", record.kills),
                format!("secrets {}", record.secrets),
                format!("time    {}", clock_line(record.time)),
            ];
            for (row, line) in handles.ending_rows.iter().zip(lines.iter()) {
                ui_set_text(&mut commands, *row, line);
            }
        }
        _ => {}
    }
}

fn reveal(dirty: &mut RetainedUiDirty, commands: &mut Commands, node: Entity, visible: bool) {
    if node == Entity::default() {
        return;
    }
    ui_set_visible(dirty, commands, node, visible);
}
