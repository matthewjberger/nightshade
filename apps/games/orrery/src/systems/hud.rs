use crate::data;
use crate::queries;
use crate::resources::{HudHandles, Mode, OrreryResources, Screen};
use nightshade::prelude::*;
use nightshade_text::prelude::*;
use nightshade_ui::prelude::*;
use nightshade_ui_menu::prelude::*;

const READOUT_COLOR: Vec4 = Vec4::new(0.88, 0.93, 1.0, 1.0);
const PROMPT_COLOR: Vec4 = Vec4::new(0.35, 0.95, 0.9, 1.0);
const TITLE_COLOR: Vec4 = Vec4::new(0.96, 0.97, 1.0, 1.0);
const FLAVOR_COLOR: Vec4 = Vec4::new(0.66, 0.72, 0.82, 1.0);
const PANEL_BACKGROUND: Vec4 = Vec4::new(0.03, 0.04, 0.07, 0.88);
const PANEL_BORDER: Vec4 = Vec4::new(1.0, 1.0, 1.0, 0.12);
const OUTLINE: Vec4 = Vec4::new(0.0, 0.0, 0.0, 1.0);

const READOUT_WIDTH: f32 = 1100.0;
const READOUT_HEIGHT: f32 = 40.0;
const READOUT_SIZE: f32 = 20.0;
const PROMPT_SIZE: f32 = 24.0;
const MARGIN: f32 = 20.0;

const TITLE_WIDTH: f32 = 520.0;
const TITLE_HEIGHT: f32 = 300.0;
const PAUSE_WIDTH: f32 = 380.0;
const PAUSE_HEIGHT: f32 = 470.0;
const TITLE_SIZE: f32 = 44.0;
const HEADING_SIZE: f32 = 30.0;
const FLAVOR_SIZE: f32 = 16.0;
const LOG_SIZE: f32 = 15.0;
const ROW_PADDING: f32 = 10.0;

pub fn build(
    mut resources: ResMut<OrreryResources>,
    style: Res<MenuStyle>,
    mut menus: ResMut<Menus<Screen>>,
    mut build: nightshade_ui::commands::builder::UiBuild,
) {
    let style = &*style;
    let menus = &mut *menus;
    let mut tree = UiTreeBuilder::new(build.context());
    let root = tree
        .add_node()
        .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
        .entity();

    let mut handles = HudHandles::default();
    let mut log = Vec::new();
    tree.in_parent(root, |tree| {
        handles.readout = tree
            .add_node()
            .window(
                Rl(vec2(50.0, 0.0)) + Ab(vec2(0.0, MARGIN)),
                Ab(vec2(READOUT_WIDTH, READOUT_HEIGHT)),
                Anchor::TopCenter,
            )
            .with_text("", READOUT_SIZE)
            .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
            .with_text_outline(OUTLINE, 1.0)
            .color_raw::<UiBase>(READOUT_COLOR)
            .without_pointer_events()
            .with_visible(false)
            .entity();

        handles.prompt = tree
            .add_node()
            .window(
                Rl(vec2(50.0, 100.0)) + Ab(vec2(0.0, -MARGIN)),
                Ab(vec2(READOUT_WIDTH, READOUT_HEIGHT)),
                Anchor::BottomCenter,
            )
            .with_text("", PROMPT_SIZE)
            .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
            .with_text_outline(OUTLINE, 1.0)
            .color_raw::<UiBase>(PROMPT_COLOR)
            .without_pointer_events()
            .with_visible(false)
            .entity();

        let title = add_menu(
            tree,
            style,
            TITLE_WIDTH,
            TITLE_HEIGHT,
            "Orrery",
            &["The system keeps turning whether you watch it or not."],
            &["Fly", "Exit"],
        );
        handles.title_panel = title.panel;
        handles.fly_button = title.buttons[0];
        handles.title_exit_button = title.buttons[1];
        menu_show_on(menus, Screen::Title, title.panel);

        let paused = pause_style();
        handles.pause_panel = add_menu_panel(tree, &paused, PAUSE_WIDTH, PAUSE_HEIGHT);
        tree.in_parent(handles.pause_panel, |tree| {
            add_menu_heading(tree, &paused, "Paused");
            for _ in 0..data::BEACONS.len() {
                log.push(add_menu_row(tree, &paused, ""));
            }
            handles.resume_button = tree.add_button("Resume");
            handles.title_return_button = tree.add_button("Quit to title");
            handles.pause_exit_button = tree.add_button("Exit");
        });
        menu_show_on(menus, Screen::Paused, handles.pause_panel);
    });

    handles.root = tree.root_entity();
    tree.finish();
    resources.hud = handles;
    resources.log = log;
}

pub fn show_log(resources: &mut OrreryResources, commands: &mut Commands) {
    for (row, node) in resources.log.clone().into_iter().enumerate() {
        let line = match resources.logged.get(row) {
            Some(&body) => format!("{}   heard", queries::body_name(body)),
            None => String::new(),
        };
        ui_set_text(commands, node, &line);
    }
}

pub fn show(mut resources: ResMut<OrreryResources>, mut commands: Commands) {
    if resources.hud.readout == Entity::default() {
        return;
    }

    let readout = compose_readout(&resources);
    if readout != resources.readout {
        ui_set_text(&mut commands, resources.hud.readout, &readout);
        resources.readout = readout;
    }

    let prompt = compose_prompt(&resources);
    if prompt != resources.prompt {
        ui_set_text(&mut commands, resources.hud.prompt, &prompt);
        resources.prompt = prompt;
    }
}

fn compose_readout(resources: &OrreryResources) -> String {
    let pose = resources.pose;
    let found = resources.flight.found;
    let total = resources.flight.total;
    match resources.flight.mode {
        Mode::Flying => {
            let nearest = queries::body_name(pose.nearest);
            let target = queries::body_name(pose.target);
            let state = match (pose.landed.is_some(), pose.autopilot) {
                (true, _) => "landed".to_string(),
                (false, true) => format!("autopilot to {target}"),
                (false, false) => format!("target {target} {:.0}", pose.target_distance.max(0.0)),
            };
            format!(
                "{nearest}   height {:.0}   speed {:.1}   {state}   {}   signals {found}/{total}{}",
                pose.height.max(0.0),
                pose.relative_speed,
                sweep_line(resources),
                knock_line(resources),
            )
        }
        Mode::OnFoot => {
            let ground = queries::body_name(pose.nearest);
            format!(
                "on {ground}   {}   signals {found}/{total}",
                sweep_line(resources)
            )
        }
    }
}

fn sweep_line(resources: &OrreryResources) -> String {
    match resources.sweep {
        Some(signal) => format!(
            "signal {} {:.0}",
            queries::body_name(signal.body),
            signal.distance
        ),
        None => "every signal heard".to_string(),
    }
}

fn knock_line(resources: &OrreryResources) -> String {
    match resources.flight.knocks {
        0 => String::new(),
        knocks => format!("   knocks {knocks}"),
    }
}

fn compose_prompt(resources: &OrreryResources) -> String {
    let pose = resources.pose;
    match resources.flight.mode {
        Mode::Flying => {
            if pose.landed.is_some() {
                "Step outside".to_string()
            } else {
                String::new()
            }
        }
        Mode::OnFoot => {
            if resources.flight.found < resources.flight.total {
                "Find the signal".to_string()
            } else if (pose.walker_position - pose.ship_position).magnitude() < data::BOARD_REACH {
                "Board the ship".to_string()
            } else {
                String::new()
            }
        }
    }
}

pub fn menu_style() -> MenuStyle {
    MenuStyle {
        background: PANEL_BACKGROUND,
        border: PANEL_BORDER,
        corner_radius: 8.0,
        padding: 16.0,
        spacing: ROW_PADDING,
        centered: true,
        heading_size: TITLE_SIZE,
        heading_color: TITLE_COLOR,
        row_size: FLAVOR_SIZE,
        row_color: FLAVOR_COLOR,
        ..Default::default()
    }
}

fn pause_style() -> MenuStyle {
    MenuStyle {
        heading_size: HEADING_SIZE,
        row_size: LOG_SIZE,
        ..menu_style()
    }
}
