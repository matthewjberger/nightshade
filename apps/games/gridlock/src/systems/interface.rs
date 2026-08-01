use crate::components::Velocity;
use crate::contract::GridlockEvent;
use crate::data::districts::DISTRICT_NAMES;
use crate::data::interface::{
    ACCENT_COLOR, HUD_HEIGHT, HUD_MARGIN, HUD_SIZE, HUD_WIDTH, MARGIN, MENU_WIDTH, OUTLINE_COLOR,
    PANEL_BACKGROUND, PANEL_BORDER, PAUSE_HEIGHT, RATE_SECONDS, RATE_WIDTH, TEXT_COLOR,
    TITLE_HEIGHT,
};
use crate::data::rooms::ROOM_NAMES;
use crate::data::settings::Settings;
use crate::queries::grid::{cell_of, district_of};
use crate::resources::{
    Flashes, GridlockResources, HudHandles, Indoors, Player, Screen, ScreenHandles,
};
use nightshade::ecs::Storage;
use nightshade::prelude::*;
use nightshade_config::contract::Config;
use nightshade_platform::prelude::Time;
use nightshade_render::data::config::RendererState;
use nightshade_scene::prelude::LocalTransform;
use nightshade_ui::commands::builder::UiBuild;
use nightshade_ui::prelude::*;
use nightshade_ui_config::prelude::{ConfigPanel, build_config_rows};
use nightshade_ui_menu::prelude::{MenuStyle, Menus, add_menu, menu_show_on};

const NOTICE_SECONDS: f32 = 3.0;
const SETTINGS_WIDTH: f32 = 340.0;
const SETTINGS_HEIGHT: f32 = 150.0;
const SETTINGS_PADDING: f32 = 12.0;
const SETTINGS_SPACING: f32 = 8.0;

pub fn menu_style() -> MenuStyle {
    MenuStyle {
        background: PANEL_BACKGROUND,
        border: PANEL_BORDER,
        row_color: TEXT_COLOR,
        padding: MARGIN,
        ..Default::default()
    }
}

pub fn build(
    mut resources: ResMut<GridlockResources>,
    config: Res<Config<Settings>>,
    style: Res<MenuStyle>,
    mut menus: ResMut<Menus<Screen>>,
    mut rows: ResMut<ConfigPanel<Settings>>,
    mut build: UiBuild,
) {
    let mut tree = UiTreeBuilder::new(build.context());
    let mut hud = HudHandles::default();
    let settings = tree.add_docked_panel_left("settings", "Settings", SETTINGS_WIDTH);
    if let Some(content) = tree.panel_content(settings) {
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
        tree.in_parent(content, |tree| {
            *rows = build_config_rows(tree, &config);
        });
    }
    tree.edit::<UiLayoutNode>(settings, |node| node.visible = false);
    hud.settings = Some(settings);
    let root = tree
        .add_node()
        .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
        .with_visible(false)
        .entity();
    tree.in_parent(root, |tree| {
        hud.rate = Some(
            tree.add_node()
                .window(
                    Rl(vec2(0.0, 0.0)) + Ab(vec2(HUD_MARGIN, HUD_MARGIN)),
                    Ab(vec2(RATE_WIDTH, HUD_HEIGHT)),
                    Anchor::TopLeft,
                )
                .with_text("", HUD_SIZE)
                .text_left()
                .with_text_outline(OUTLINE_COLOR, 1.0)
                .color_raw::<UiBase>(TEXT_COLOR)
                .without_pointer_events()
                .entity(),
        );
        hud.readout = Some(
            tree.add_node()
                .window(
                    Rl(vec2(0.0, 0.0)) + Ab(vec2(HUD_MARGIN + RATE_WIDTH, HUD_MARGIN)),
                    Ab(vec2(HUD_WIDTH, HUD_HEIGHT)),
                    Anchor::TopLeft,
                )
                .with_text("", HUD_SIZE)
                .text_left()
                .with_text_outline(OUTLINE_COLOR, 1.0)
                .color_raw::<UiBase>(TEXT_COLOR)
                .without_pointer_events()
                .entity(),
        );
        hud.notice = Some(
            tree.add_node()
                .window(
                    Rl(vec2(50.0, 0.0)) + Ab(vec2(0.0, HUD_MARGIN * 3.0)),
                    Ab(vec2(HUD_WIDTH, HUD_HEIGHT)),
                    Anchor::TopCenter,
                )
                .with_text("", HUD_SIZE)
                .text_center()
                .with_text_outline(OUTLINE_COLOR, 1.0)
                .color_raw::<UiBase>(ACCENT_COLOR)
                .without_pointer_events()
                .entity(),
        );
    });
    hud.root = Some(root);

    let title = add_menu(
        &mut tree,
        &style,
        MENU_WIDTH,
        TITLE_HEIGHT,
        "GRIDLOCK",
        &["The city never sleeps."],
        &["Drive", "Settings", "Quit"],
    );
    let pause = add_menu(
        &mut tree,
        &style,
        MENU_WIDTH,
        PAUSE_HEIGHT,
        "Paused",
        &["The city keeps moving."],
        &["Resume", "Settings", "Quit to title"],
    );
    let tuning = add_menu(
        &mut tree,
        &style,
        MENU_WIDTH,
        SETTINGS_HEIGHT,
        "Settings",
        &["A change takes effect at once and is kept."],
        &["Back"],
    );
    menu_show_on(&mut menus, Screen::Title, title.panel);
    menu_show_on(&mut menus, Screen::Paused, pause.panel);
    menu_show_on(&mut menus, Screen::Settings, tuning.panel);
    tree.finish();

    resources.hud = hud;
    resources.screens = ScreenHandles {
        play_button: title.buttons.first().copied(),
        title_settings_button: title.buttons.get(1).copied(),
        title_quit_button: title.buttons.get(2).copied(),
        resume_button: pause.buttons.first().copied(),
        pause_settings_button: pause.buttons.get(1).copied(),
        pause_title_button: pause.buttons.get(2).copied(),
        settings_back_button: tuning.buttons.first().copied(),
    };
}

pub fn refresh(
    mut resources: ResMut<GridlockResources>,
    player: Res<Player>,
    indoors: Res<Indoors>,
    flashes: Res<Flashes>,
    renderer_state: Res<RendererState>,
    time: Res<Time>,
    events: EventReader<GridlockEvent>,
    mut commands: Commands,
    storage: &mut Storage,
) {
    let Some(body) = player.body else {
        return;
    };
    let anchor = player.car.unwrap_or(body);
    let position = storage
        .get::<LocalTransform>(anchor)
        .map(|transform| transform.translation)
        .unwrap_or_default();
    let speed = player
        .car
        .and_then(|car| storage.get::<Velocity>(car))
        .map(|velocity| velocity.0.norm() * 3.6)
        .unwrap_or(0.0);
    let hour = renderer_state.day_night.hour.rem_euclid(24.0);
    let minutes = ((hour - hour.floor()) * 60.0).floor() as u32;
    let place = match indoors.visit {
        Some(visit) => ROOM_NAMES[visit.room as usize],
        None => DISTRICT_NAMES[district_of(cell_of(position)) as usize],
    };
    let streak = if flashes.streak > 1 {
        format!("   x{}", flashes.streak)
    } else {
        String::new()
    };
    let readout = if player.car.is_some() {
        format!(
            "{:02}:{minutes:02}   {place}   {:.0} km/h{streak}",
            hour.floor() as u32,
            speed
        )
    } else {
        format!("{:02}:{minutes:02}   {place}{streak}", hour.floor() as u32)
    };
    let resources = &mut *resources;
    ui_set_text_cached(
        &mut commands,
        resources.hud.readout,
        &mut resources.last_readout,
        &readout,
    );
    resources.rate_left -= time.delta_time;
    if resources.rate_left <= 0.0 {
        resources.rate_left = RATE_SECONDS;
        let rate = format!("{:.0} fps", time.frames_per_second);
        ui_set_text_cached(
            &mut commands,
            resources.hud.rate,
            &mut resources.last_rate,
            &rate,
        );
    }

    for event in &events {
        let notice = match event {
            GridlockEvent::Boarded { car } => format!("You took the {car}"),
            GridlockEvent::Left { car } => format!("You left the {car}"),
            GridlockEvent::Entered { place } => format!("You went into {place}"),
            GridlockEvent::Exited { place } => format!("You came out of {place}"),
        };
        resources.notice_left = NOTICE_SECONDS;
        ui_set_text_cached(
            &mut commands,
            resources.hud.notice,
            &mut resources.last_notice,
            &notice,
        );
    }
    if resources.notice_left > 0.0 {
        resources.notice_left -= time.delta_time;
        if resources.notice_left <= 0.0 {
            ui_set_text_cached(
                &mut commands,
                resources.hud.notice,
                &mut resources.last_notice,
                "",
            );
        }
    }
}
