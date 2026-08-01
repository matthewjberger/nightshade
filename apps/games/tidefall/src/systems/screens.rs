use crate::data::{Ask, Settings};
use crate::resources::{Screen, ScreenHandles, TidefallResources};
use nightshade::prelude::*;
use nightshade_config::prelude::Config;
use nightshade_input::Actions;
use nightshade_platform::prelude::*;
use nightshade_state::prelude::*;
use nightshade_ui::components::UiLayoutNode;
use nightshade_ui::components::layout_types::{FlowAlignment, FlowDirection, FlowLayout};
use nightshade_ui::prelude::*;
use nightshade_ui::resources::{RetainedUiDirty, RetainedUiFrame};
use nightshade_ui_config::prelude::{ConfigPanel, build_config_rows};
use nightshade_ui_menu::prelude::*;

const PANEL_BACKGROUND: Vec4 = Vec4::new(0.03, 0.04, 0.07, 0.88);
const PANEL_BORDER: Vec4 = Vec4::new(0.55, 0.80, 1.0, 0.25);
const TEXT: Vec4 = Vec4::new(0.92, 0.95, 1.0, 1.0);
const TITLE_WIDTH: f32 = 480.0;
const TITLE_HEIGHT: f32 = 230.0;
const MENU_WIDTH: f32 = 330.0;
const MENU_HEIGHT: f32 = 240.0;
const SETTINGS_WIDTH: f32 = 360.0;
const SETTINGS_PADDING: f32 = 12.0;
const SETTINGS_SPACING: f32 = 8.0;
const TITLE_SIZE: f32 = 40.0;
const HEADING_SIZE: f32 = 30.0;
const ROW_SIZE: f32 = 18.0;

pub fn menu_style() -> MenuStyle {
    MenuStyle {
        background: PANEL_BACKGROUND,
        border: PANEL_BORDER,
        corner_radius: 10.0,
        border_width: 1.0,
        padding: 14.0,
        spacing: 8.0,
        visible: true,
        centered: true,
        heading_size: HEADING_SIZE,
        heading_color: TEXT,
        row_size: ROW_SIZE,
        row_color: TEXT,
        ..Default::default()
    }
}

pub fn build(
    tree: &mut UiTreeBuilder,
    style: &MenuStyle,
    menus: &mut Menus<Screen>,
    config: &Config<Settings>,
) -> (ScreenHandles, ConfigPanel<Settings>) {
    let title_style = MenuStyle {
        heading_size: TITLE_SIZE,
        ..*style
    };
    let title = add_menu(
        tree,
        &title_style,
        TITLE_WIDTH,
        TITLE_HEIGHT,
        "Tidefall",
        &["A small solar system, and a ship to cross it."],
        &["Play", "Quit"],
    );
    let pause = add_menu(
        tree,
        style,
        MENU_WIDTH,
        MENU_HEIGHT,
        "Paused",
        &[],
        &["Resume", "Settings", "Quit to title"],
    );
    menu_show_on(menus, Screen::Title, title.panel);
    menu_show_on(menus, Screen::Paused, pause.panel);

    let settings_panel = tree.add_docked_panel_left("settings", "Settings", SETTINGS_WIDTH);
    let content = tree.panel_content(settings_panel).unwrap_or(settings_panel);
    tree.edit::<UiLayoutNode>(content, |node| {
        node.flow_layout = Some(FlowLayout {
            direction: FlowDirection::Vertical,
            padding: SETTINGS_PADDING,
            spacing: SETTINGS_SPACING,
            alignment: FlowAlignment::Start,
            cross_alignment: FlowAlignment::Start,
            wrap: false,
        });
    });
    let mut rows = ConfigPanel::<Settings>::default();
    tree.in_parent(content, |tree| {
        rows = build_config_rows(tree, config);
    });
    tree.edit::<UiLayoutNode>(settings_panel, |node| node.visible = false);

    (
        ScreenHandles {
            play_button: title.buttons[0],
            title_quit_button: title.buttons[1],
            resume_button: pause.buttons[0],
            settings_button: pause.buttons[1],
            pause_title_button: pause.buttons[2],
            settings_panel,
        },
        rows,
    )
}

pub fn watch_the_keys(
    actions: Res<Actions<Ask>>,
    screen: Res<State<Screen>>,
    mut next: ResMut<NextState<Screen>>,
) {
    if !actions.just_pressed(Ask::Pause) {
        return;
    }
    match screen.current {
        Screen::Playing => next.pending = Some(Screen::Paused),
        Screen::Paused => next.pending = Some(Screen::Playing),
        Screen::Title => {}
    }
}

pub fn drive_the_menus(
    mut resources: ResMut<TidefallResources>,
    frame: Res<RetainedUiFrame>,
    mut next: ResMut<NextState<Screen>>,
    mut window: ResMut<Window>,
    mut dirty: ResMut<RetainedUiDirty>,
    mut commands: Commands,
) {
    let handles = resources.screens;
    for entity in ui_button_clicks(&frame) {
        if entity == handles.play_button || entity == handles.resume_button {
            next.pending = Some(Screen::Playing);
        } else if entity == handles.settings_button {
            resources.shown.settings_open = !resources.shown.settings_open;
            ui_set_visible(
                &mut dirty,
                &mut commands,
                handles.settings_panel,
                resources.shown.settings_open,
            );
        } else if entity == handles.pause_title_button {
            next.pending = Some(Screen::Title);
        } else if entity == handles.title_quit_button {
            window.should_exit = true;
        }
    }
}

pub fn enter_the_world(
    mut resources: ResMut<TidefallResources>,
    mut window: ResMut<Window>,
    mut dirty: ResMut<RetainedUiDirty>,
    mut commands: Commands,
) {
    if resources.shown.settings_open {
        resources.shown.settings_open = false;
        ui_set_visible(
            &mut dirty,
            &mut commands,
            resources.screens.settings_panel,
            false,
        );
    }
    ui_set_visible(&mut dirty, &mut commands, resources.hud.root, true);
    set_cursor_locked(&mut window, true);
    set_cursor_visible(&mut window, false);
}

pub fn leave_the_world(
    resources: Res<TidefallResources>,
    mut window: ResMut<Window>,
    mut dirty: ResMut<RetainedUiDirty>,
    mut commands: Commands,
) {
    ui_set_visible(&mut dirty, &mut commands, resources.hud.root, false);
    set_cursor_locked(&mut window, false);
    set_cursor_visible(&mut window, true);
}
