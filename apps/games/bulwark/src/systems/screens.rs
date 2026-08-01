use crate::commands::record;
use crate::data::{Ask, Settings};
use crate::queries;
use crate::resources::{Bulwark, Screen, Screens};
use crate::theme;
use nightshade::prelude::*;
use nightshade_config::contract::Config;
use nightshade_input::Actions;
use nightshade_platform::prelude::*;
use nightshade_state::prelude::*;
use nightshade_ui::prelude::*;
use nightshade_ui::resources::{RetainedUiDirty, RetainedUiFrame};
use nightshade_ui_config::prelude::{ConfigPanel, build_config_rows};
use nightshade_ui_menu::prelude::*;
use nightshade_waves::WaveProgress;

pub fn build(
    tree: &mut UiTreeBuilder,
    style: &MenuStyle,
    menus: &mut Menus<Screen>,
    config: &Config<Settings>,
    rows: &mut ConfigPanel<Settings>,
) -> Screens {
    let dim = tree
        .add_node()
        .window(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)), Anchor::TopLeft)
        .with_rect(0.0, 0.0, Vec4::zeros())
        .color_raw::<UiBase>(Vec4::from(theme::DIM_COLOR))
        .with_visible(false)
        .entity();

    let title = add_menu(
        tree,
        style,
        theme::MENU_WIDTH,
        theme::TITLE_HEIGHT,
        "Bulwark",
        &[
            "Crystals walk the road to your keep.",
            "Raise towers on the grass beside it.",
            "Hold the line for twelve waves.",
            "",
        ],
        &["March", "Board", "Settings", "Quit"],
    );

    let options = add_menu_panel(tree, style, theme::MENU_WIDTH, theme::OPTIONS_HEIGHT);
    let mut options_back = Entity::default();
    tree.in_parent(options, |tree| {
        add_menu_heading(tree, style, "Settings");
        *rows = build_config_rows(tree, config);
        options_back = tree.add_button("Back");
    });

    let pause = add_menu(
        tree,
        style,
        theme::MENU_WIDTH,
        theme::PAUSE_HEIGHT,
        "Held",
        &["The board waits."],
        &["Carry on", "Settings", "Give up"],
    );

    let lost = add_menu(
        tree,
        style,
        theme::MENU_WIDTH,
        theme::OVER_HEIGHT,
        "The keep falls",
        &[""],
        &["Again", "Title"],
    );

    let won = add_menu(
        tree,
        style,
        theme::MENU_WIDTH,
        theme::OVER_HEIGHT,
        "The road is clear",
        &[""],
        &["Again", "Title"],
    );

    menu_show_on(menus, Screen::Title, title.panel);
    menu_show_on(menus, Screen::Options, options);
    menu_show_on(menus, Screen::Paused, pause.panel);
    menu_show_on(menus, Screen::Defeat, lost.panel);
    menu_show_on(menus, Screen::Victory, won.panel);

    Screens {
        dim,
        play: title.buttons[0],
        title_board: title.buttons[1],
        title_best: title.rows[3],
        title_options: title.buttons[2],
        title_quit: title.buttons[3],
        options_back,
        resume: pause.buttons[0],
        pause_options: pause.buttons[1],
        pause_title: pause.buttons[2],
        lost_again: lost.buttons[0],
        lost_title: lost.buttons[1],
        lost_line: lost.rows[0],
        won_again: won.buttons[0],
        won_title: won.buttons[1],
        won_line: won.rows[0],
        came_from: Screen::Title,
    }
}

pub fn watch_the_keys(
    actions: Res<Actions<Ask>>,
    screen: Res<State<Screen>>,
    bulwark: Res<Bulwark>,
    mut next: ResMut<NextState<Screen>>,
) {
    if !actions.just_pressed(Ask::Pause) {
        return;
    }
    match screen.current {
        Screen::Playing => next.pending = Some(Screen::Paused),
        Screen::Paused => next.pending = Some(Screen::Playing),
        Screen::Options => next.pending = Some(bulwark.screens.came_from),
        Screen::Title | Screen::Defeat | Screen::Victory => {}
    }
}

pub fn drive_the_menus(
    screen: Res<State<Screen>>,
    frame: Res<RetainedUiFrame>,
    mut bulwark: ResMut<Bulwark>,
    mut next: ResMut<NextState<Screen>>,
    mut window: ResMut<Window>,
) {
    let handles = bulwark.screens.clone();
    let hold = bulwark.hud.hold;
    for entity in ui_button_clicks(&frame) {
        if entity == hold && screen.current == Screen::Playing {
            next.pending = Some(Screen::Paused);
            continue;
        }
        if entity == handles.play
            || entity == handles.resume
            || entity == handles.lost_again
            || entity == handles.won_again
        {
            next.pending = Some(Screen::Playing);
        } else if entity == handles.title_board {
            bulwark.board = (bulwark.board + 1) % queries::boards_total();
        } else if entity == handles.title_options {
            bulwark.screens.came_from = Screen::Title;
            next.pending = Some(Screen::Options);
        } else if entity == handles.pause_options {
            bulwark.screens.came_from = Screen::Paused;
            next.pending = Some(Screen::Options);
        } else if entity == handles.options_back {
            next.pending = Some(handles.came_from);
        } else if entity == handles.pause_title
            || entity == handles.lost_title
            || entity == handles.won_title
        {
            next.pending = Some(Screen::Title);
        } else if entity == handles.title_quit {
            window.should_exit = true;
        }
    }
    if screen.current == Screen::Playing && bulwark.lives == 0 {
        next.pending = Some(Screen::Defeat);
    }
}

pub fn show_the_hud(
    bulwark: Res<Bulwark>,
    mut dirty: ResMut<RetainedUiDirty>,
    mut commands: Commands,
) {
    set_hud(&bulwark, &mut dirty, &mut commands, true);
}

pub fn hide_the_hud(
    bulwark: Res<Bulwark>,
    mut dirty: ResMut<RetainedUiDirty>,
    mut commands: Commands,
) {
    set_hud(&bulwark, &mut dirty, &mut commands, false);
}

pub fn name_the_board(bulwark: Res<Bulwark>, mut commands: Commands) {
    if bulwark.screens.title_board == Entity::default() {
        return;
    }
    let label = format!("Board   {}", queries::board_name(bulwark.board));
    ui_set_text(&mut commands, bulwark.screens.title_board, &label);
}

pub fn dim_the_board(
    screen: Res<State<Screen>>,
    mut bulwark: ResMut<Bulwark>,
    mut dirty: ResMut<RetainedUiDirty>,
    mut commands: Commands,
) {
    let dim = bulwark.screens.dim;
    if dim == Entity::default() {
        return;
    }
    let wanted = matches!(
        screen.current,
        Screen::Paused | Screen::Defeat | Screen::Victory
    );
    if wanted == bulwark.last_dim {
        return;
    }
    ui_set_visible(&mut dirty, &mut commands, dim, wanted);
    bulwark.last_dim = wanted;
}

pub fn show_the_best(bulwark: Res<Bulwark>, mut commands: Commands) {
    if bulwark.screens.title_best == Entity::default() {
        return;
    }
    let line = queries::best_line(bulwark.best);
    ui_set_text(&mut commands, bulwark.screens.title_best, &line);
}

pub fn tell_the_ending(
    mut bulwark: ResMut<Bulwark>,
    progress: Res<WaveProgress>,
    mut commands: Commands,
) {
    let beaten = record::keep_the_record(&mut bulwark, &progress);
    if bulwark.lives == 0 {
        let line = queries::defeat_line(&bulwark, &progress, beaten);
        ui_set_text(&mut commands, bulwark.screens.lost_line, &line);
        return;
    }
    let line = queries::victory_line(&bulwark, beaten);
    ui_set_text(&mut commands, bulwark.screens.won_line, &line);
}

fn set_hud(bulwark: &Bulwark, dirty: &mut RetainedUiDirty, commands: &mut Commands, visible: bool) {
    if bulwark.hud.root == Entity::default() {
        return;
    }
    ui_set_visible(dirty, commands, bulwark.hud.root, visible);
}
