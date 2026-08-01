use crate::commands::interface::{set_bond_row, set_fill, set_text};
use crate::commands::interface::{set_menu_height, show_menu_rows};
use crate::commands::save::read_the_run;
use crate::data::{
    ACTIVITIES, Bond, CHAPTERS, COMPANIONS, MENU_LABELS, PROLOGUE, Team, growth_of, level_of,
    level_share, morale_name, rapport_growth,
};
use crate::queries::Structures;
use crate::queries::camp::menu_rows;
use crate::queries::menu::{menu_items, pause_items};
use crate::queries::terrain::terrain_name;
use crate::queries::{
    affected, area_tiles, effective_move, fighter_index, hp_share, is_stationary, mp_share,
    mustered, predict, roster, roster_index, site_at, sites, squad, status_text, turn_order,
    unit_at,
};
use crate::resources::{
    Battle, Camp, CampPhase, Campaign, Cursor, Pause, Phase, Prompt, Screen, Terrain, UiHandles,
};
use crate::theme;
use nalgebra_glm::{Vec4, vec2};
use nightshade::prelude::*;
use nightshade_state::prelude::*;
use nightshade_ui::prelude::*;
use nightshade_ui_menu::prelude::*;

pub fn build(mut handles: ResMut<UiHandles>, mut build: UiBuild) {
    let mut tree = UiTreeBuilder::new(build.context());
    let root = tree
        .add_node()
        .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
        .entity();
    let built = tree.in_parent(root, |tree| {
        let mut built = UiHandles::default();
        build_unit_card(tree, &mut built);
        build_tile_card(tree, &mut built);
        build_camp(tree, &mut built);
        build_speech(tree, &mut built);
        build_fighter_card(tree, &mut built);
        build_camp_menu(tree, &mut built);
        build_order(tree, &mut built);
        build_menu(tree, &mut built);
        build_ability_menu(tree, &mut built);
        build_log(tree, &mut built);
        build_predict(tree, &mut built);
        build_title(tree, &mut built);
        build_pause(tree, &mut built);
        build_prologue(tree, &mut built);
        build_over(tree, &mut built);
        built
    });
    let mut built = built;
    built.layout_root = Some(tree.finish());
    *handles = built;
}

fn bar(tree: &mut UiTreeBuilder<'_, '_>, color: Vec4) -> Entity {
    let track = tree
        .add_node()
        .flow_child(Ab(vec2(theme::BAR_WIDTH, theme::BAR_HEIGHT)))
        .with_rect(4.0, 1.0, theme::PANEL_BORDER)
        .color_raw::<UiBase>(theme::HP_TRACK)
        .entity();
    let mut fill = Entity::default();
    tree.in_parent(track, |tree| {
        fill = tree
            .add_node()
            .window(
                Ab(vec2(0.0, 0.0)),
                Ab(vec2(theme::BAR_WIDTH, theme::BAR_HEIGHT)),
                Anchor::TopLeft,
            )
            .with_rect(4.0, 0.0, Vec4::new(0.0, 0.0, 0.0, 0.0))
            .color_raw::<UiBase>(color)
            .entity();
    });
    fill
}

fn build_unit_card(tree: &mut UiTreeBuilder<'_, '_>, built: &mut UiHandles) {
    let panel = tree
        .card_node(10.0, 2.0, theme::PANEL_BORDER)
        .color_raw::<UiBase>(theme::PANEL_COLOR)
        .window(
            Ab(vec2(20.0, 20.0)),
            Ab(vec2(theme::CARD_WIDTH, theme::CARD_HEIGHT)),
            Anchor::TopLeft,
        )
        .flow(FlowDirection::Vertical, 12.0, 4.0)
        .with_visible(false)
        .entity();
    tree.in_parent(panel, |tree| {
        built.unit_name = Some(add_menu_text(
            tree,
            "",
            theme::BODY_SIZE,
            theme::ACCENT_COLOR,
        ));
        built.unit_class = Some(add_menu_text(
            tree,
            "",
            theme::SMALL_SIZE,
            theme::DIM_TEXT_COLOR,
        ));
        built.unit_hp = Some(add_menu_text(
            tree,
            "",
            theme::SMALL_SIZE,
            theme::TEXT_COLOR,
        ));
        built.unit_hp_fill = Some(bar(tree, theme::HP_FILL));
        built.unit_mp = Some(add_menu_text(
            tree,
            "",
            theme::SMALL_SIZE,
            theme::TEXT_COLOR,
        ));
        built.unit_mp_fill = Some(bar(tree, theme::MP_FILL));
        built.unit_stats = Some(add_menu_text(
            tree,
            "",
            theme::SMALL_SIZE,
            theme::DIM_TEXT_COLOR,
        ));
        built.unit_movement = Some(add_menu_text(
            tree,
            "",
            theme::SMALL_SIZE,
            theme::DIM_TEXT_COLOR,
        ));
        built.unit_status = Some(add_menu_text(
            tree,
            "",
            theme::SMALL_SIZE,
            theme::ENEMY_COLOR,
        ));
    });
    built.unit_card = Some(panel);
}

fn build_tile_card(tree: &mut UiTreeBuilder<'_, '_>, built: &mut UiHandles) {
    let panel = tree
        .card_node(10.0, 2.0, theme::PANEL_BORDER)
        .color_raw::<UiBase>(theme::PANEL_COLOR)
        .window(
            Rl(vec2(0.0, 100.0)) + Ab(vec2(20.0, -32.0 - theme::LOG_HEIGHT)),
            Ab(vec2(theme::CARD_WIDTH, theme::TILE_CARD_HEIGHT)),
            Anchor::BottomLeft,
        )
        .flow(FlowDirection::Vertical, 12.0, 4.0)
        .with_visible(false)
        .entity();
    tree.in_parent(panel, |tree| {
        built.tile_label = Some(add_menu_text(
            tree,
            "",
            theme::SMALL_SIZE,
            theme::DIM_TEXT_COLOR,
        ));
        built.tile_site = Some(add_menu_text(
            tree,
            "",
            theme::SMALL_SIZE,
            theme::ACCENT_COLOR,
        ));
        built.tile_unit = Some(add_menu_text(tree, "", theme::BODY_SIZE, theme::TEXT_COLOR));
        built.tile_hp = Some(add_menu_text(
            tree,
            "",
            theme::SMALL_SIZE,
            theme::TEXT_COLOR,
        ));
    });
    built.tile_card = Some(panel);
}

fn build_camp(tree: &mut UiTreeBuilder<'_, '_>, built: &mut UiHandles) {
    let panel = tree
        .card_node(10.0, 2.0, theme::PANEL_BORDER)
        .color_raw::<UiBase>(theme::PANEL_COLOR)
        .window(
            Ab(vec2(20.0, 20.0)),
            Ab(vec2(theme::CARD_WIDTH, theme::CAMP_PANEL_HEIGHT)),
            Anchor::TopLeft,
        )
        .flow(FlowDirection::Vertical, 12.0, 4.0)
        .with_visible(false)
        .entity();
    tree.in_parent(panel, |tree| {
        built.camp_chapter = Some(add_menu_text(
            tree,
            "",
            theme::BODY_SIZE,
            theme::ACCENT_COLOR,
        ));
        built.camp_day = Some(add_menu_text(
            tree,
            "",
            theme::SMALL_SIZE,
            theme::TEXT_COLOR,
        ));
        built.camp_supplies = Some(add_menu_text(
            tree,
            "",
            theme::SMALL_SIZE,
            theme::TEXT_COLOR,
        ));
        built.camp_objective = Some(add_menu_text(
            tree,
            "",
            theme::SMALL_SIZE,
            theme::ACCENT_COLOR,
        ));
        built.camp_brief = Some(add_menu_text(
            tree,
            "",
            theme::SMALL_SIZE,
            theme::DIM_TEXT_COLOR,
        ));
        built.camp_prompt = Some(add_menu_text(
            tree,
            "",
            theme::SMALL_SIZE,
            theme::PLAYER_COLOR,
        ));
        built.march_button = Some(tree.add_button("March out"));
    });
    built.camp_panel = Some(panel);
}

fn build_speech(tree: &mut UiTreeBuilder<'_, '_>, built: &mut UiHandles) {
    let panel = tree
        .card_node(10.0, 2.0, theme::PANEL_BORDER)
        .color_raw::<UiBase>(theme::PANEL_COLOR)
        .window(
            Rl(vec2(50.0, 100.0)) + Ab(vec2(0.0, -theme::SPEECH_LIFT)),
            Ab(vec2(theme::SPEECH_WIDTH, theme::SPEECH_HEIGHT)),
            Anchor::BottomCenter,
        )
        .flow(FlowDirection::Vertical, 20.0, 8.0)
        .with_visible(false)
        .entity();
    tree.in_parent(panel, |tree| {
        built.speech_name = Some(add_menu_text(
            tree,
            "",
            theme::BODY_SIZE,
            theme::ACCENT_COLOR,
        ));
        built.speech_line = Some(add_menu_text(
            tree,
            "",
            theme::SMALL_SIZE,
            theme::TEXT_COLOR,
        ));
        built.speech_hint = Some(add_menu_text(
            tree,
            "",
            theme::SMALL_SIZE,
            theme::DIM_TEXT_COLOR,
        ));
    });
    built.speech_panel = Some(panel);
}

fn build_fighter_card(tree: &mut UiTreeBuilder<'_, '_>, built: &mut UiHandles) {
    let panel = tree
        .card_node(10.0, 2.0, theme::PANEL_BORDER)
        .color_raw::<UiBase>(theme::PANEL_COLOR)
        .window(
            Ab(vec2(20.0, theme::CAMP_PANEL_HEIGHT + 32.0)),
            Ab(vec2(theme::CARD_WIDTH, theme::FIGHTER_CARD_HEIGHT)),
            Anchor::TopLeft,
        )
        .flow(FlowDirection::Vertical, 12.0, 4.0)
        .with_visible(false)
        .entity();
    tree.in_parent(panel, |tree| {
        built.fighter_name = Some(add_menu_text(
            tree,
            "",
            theme::BODY_SIZE,
            theme::ACCENT_COLOR,
        ));
        built.fighter_class = Some(add_menu_text(
            tree,
            "",
            theme::SMALL_SIZE,
            theme::DIM_TEXT_COLOR,
        ));
        built.fighter_level = Some(add_menu_text(
            tree,
            "",
            theme::SMALL_SIZE,
            theme::TEXT_COLOR,
        ));
        built.fighter_level_fill = Some(bar(tree, theme::MP_FILL));
        built.fighter_rapport = Some(add_menu_text(
            tree,
            "",
            theme::SMALL_SIZE,
            theme::TEXT_COLOR,
        ));
        built.fighter_rapport_fill = Some(bar(tree, theme::ACCENT_COLOR));
        built.fighter_stats = Some(add_menu_text(
            tree,
            "",
            theme::SMALL_SIZE,
            theme::DIM_TEXT_COLOR,
        ));
        built.fighter_state = Some(add_menu_text(
            tree,
            "",
            theme::SMALL_SIZE,
            theme::PLAYER_COLOR,
        ));
        built.fighter_bond = Some(add_menu_text(
            tree,
            "",
            theme::SMALL_SIZE,
            theme::DIM_TEXT_COLOR,
        ));
        built.fighter_arc = Some(add_menu_text(
            tree,
            "",
            theme::SMALL_SIZE,
            theme::ACCENT_COLOR,
        ));
        built.fighter_arc_line = Some(add_menu_text(
            tree,
            "",
            theme::SMALL_SIZE,
            theme::DIM_TEXT_COLOR,
        ));
    });
    built.fighter_panel = Some(panel);
}

fn build_camp_menu(tree: &mut UiTreeBuilder<'_, '_>, built: &mut UiHandles) {
    let panel = tree
        .card_node(10.0, 2.0, theme::PANEL_BORDER)
        .color_raw::<UiBase>(theme::PANEL_COLOR)
        .window(
            Rl(vec2(50.0, 100.0)) + Ab(vec2(0.0, -theme::SPEECH_LIFT)),
            Ab(vec2(theme::CAMP_MENU_WIDTH, theme::CAMP_MENU_HEIGHT)),
            Anchor::BottomCenter,
        )
        .flow(FlowDirection::Vertical, 14.0, 6.0)
        .with_visible(false)
        .entity();
    tree.in_parent(panel, |tree| {
        built.camp_menu_title = Some(add_menu_text(
            tree,
            "",
            theme::SMALL_SIZE,
            theme::ACCENT_COLOR,
        ));
        for _ in 0..COMPANIONS.len() {
            let button = tree.add_button("");
            built.camp_menu_buttons.push(button);
        }
    });
    built.camp_menu_panel = Some(panel);
}

fn build_order(tree: &mut UiTreeBuilder<'_, '_>, built: &mut UiHandles) {
    let panel = tree
        .card_node(10.0, 2.0, theme::PANEL_BORDER)
        .color_raw::<UiBase>(theme::PANEL_COLOR)
        .window(
            Rl(vec2(100.0, 0.0)) + Ab(vec2(-20.0, 20.0)),
            Ab(vec2(theme::ORDER_WIDTH, theme::ORDER_HEIGHT)),
            Anchor::TopRight,
        )
        .flow(FlowDirection::Vertical, 12.0, 2.0)
        .with_visible(false)
        .entity();
    tree.in_parent(panel, |tree| {
        built.order_title = Some(add_menu_text(
            tree,
            "Turn order",
            theme::SMALL_SIZE,
            theme::DIM_TEXT_COLOR,
        ));
        for _ in 0..theme::ORDER_LENGTH {
            let row = add_menu_text(tree, "", theme::SMALL_SIZE, theme::TEXT_COLOR);
            built.order_rows.push(row);
        }
    });
    built.order_panel = Some(panel);
}
fn build_menu(tree: &mut UiTreeBuilder<'_, '_>, built: &mut UiHandles) {
    let panel = tree
        .card_node(10.0, 2.0, theme::PANEL_BORDER)
        .color_raw::<UiBase>(theme::PANEL_COLOR)
        .window(
            Rl(vec2(100.0, 0.0)) + Ab(vec2(-20.0, theme::ORDER_HEIGHT + 32.0)),
            Ab(vec2(theme::MENU_WIDTH, theme::MENU_HEIGHT)),
            Anchor::TopRight,
        )
        .flow(FlowDirection::Vertical, 14.0, 6.0)
        .with_visible(false)
        .entity();
    tree.in_parent(panel, |tree| {
        built.menu_title = Some(add_menu_text(
            tree,
            "",
            theme::SMALL_SIZE,
            theme::ACCENT_COLOR,
        ));
        for label in MENU_LABELS {
            let button = tree.add_button(label);
            built.menu_buttons.push(button);
        }
    });
    built.menu_panel = Some(panel);
}

fn build_ability_menu(tree: &mut UiTreeBuilder<'_, '_>, built: &mut UiHandles) {
    let panel = tree
        .card_node(10.0, 2.0, theme::PANEL_BORDER)
        .color_raw::<UiBase>(theme::PANEL_COLOR)
        .window(
            Rl(vec2(100.0, 0.0)) + Ab(vec2(-20.0, theme::ORDER_HEIGHT + 32.0)),
            Ab(vec2(theme::MENU_WIDTH, theme::ABILITY_HEIGHT)),
            Anchor::TopRight,
        )
        .flow(FlowDirection::Vertical, 14.0, 6.0)
        .with_visible(false)
        .entity();
    tree.in_parent(panel, |tree| {
        built.ability_title = Some(add_menu_text(
            tree,
            "",
            theme::SMALL_SIZE,
            theme::ACCENT_COLOR,
        ));
        for _ in 0..2 {
            let button = tree.add_button("");
            built.ability_buttons.push(button);
        }
        add_menu_text(tree, "X back", theme::SMALL_SIZE, theme::DIM_TEXT_COLOR);
    });
    built.ability_panel = Some(panel);
}

fn build_log(tree: &mut UiTreeBuilder<'_, '_>, built: &mut UiHandles) {
    let panel = tree
        .card_node(10.0, 2.0, theme::PANEL_BORDER)
        .color_raw::<UiBase>(theme::PANEL_COLOR)
        .window(
            Rl(vec2(0.0, 100.0)) + Ab(vec2(20.0, -20.0)),
            Ab(vec2(theme::LOG_WIDTH, theme::LOG_HEIGHT)),
            Anchor::BottomLeft,
        )
        .flow(FlowDirection::Vertical, 10.0, 1.0)
        .with_visible(false)
        .entity();
    tree.in_parent(panel, |tree| {
        for _ in 0..theme::LOG_LINES {
            let row = add_menu_text(tree, "", theme::SMALL_SIZE, theme::TEXT_COLOR);
            built.log_rows.push(row);
        }
    });
    built.log_panel = Some(panel);
}
fn build_predict(tree: &mut UiTreeBuilder<'_, '_>, built: &mut UiHandles) {
    let panel = tree
        .card_node(10.0, 2.0, theme::PANEL_BORDER)
        .color_raw::<UiBase>(theme::PANEL_COLOR)
        .window(
            Rl(vec2(100.0, 100.0)) + Ab(vec2(-20.0, -20.0)),
            Ab(vec2(theme::PREDICT_WIDTH, theme::PREDICT_HEIGHT)),
            Anchor::BottomRight,
        )
        .flow(FlowDirection::Vertical, 12.0, 4.0)
        .with_visible(false)
        .entity();
    tree.in_parent(panel, |tree| {
        built.predict_label = Some(add_menu_text(
            tree,
            "",
            theme::BODY_SIZE,
            theme::ACCENT_COLOR,
        ));
        built.predict_detail = Some(add_menu_text(
            tree,
            "",
            theme::SMALL_SIZE,
            theme::TEXT_COLOR,
        ));
    });
    built.predict_panel = Some(panel);
}

fn build_title(tree: &mut UiTreeBuilder<'_, '_>, built: &mut UiHandles) {
    let panel = tree
        .card_node(10.0, 2.0, theme::PANEL_BORDER)
        .color_raw::<UiBase>(theme::PANEL_COLOR)
        .window(
            Rl(vec2(50.0, 50.0)),
            Ab(vec2(theme::TITLE_WIDTH, theme::TITLE_HEIGHT)),
            Anchor::Center,
        )
        .flow(FlowDirection::Vertical, 24.0, 10.0)
        .with_visible(false)
        .entity();
    tree.in_parent(panel, |tree| {
        tree.add_node()
            .fill_width()
            .auto_height()
            .with_text(theme::TITLE, theme::TITLE_SIZE)
            .text_center()
            .color_raw::<UiBase>(theme::ACCENT_COLOR)
            .entity();
        tree.add_node()
            .fill_width()
            .auto_height()
            .with_text("The valley the war left behind", theme::BODY_SIZE)
            .text_center()
            .color_raw::<UiBase>(theme::TEXT_COLOR)
            .entity();
        add_menu_text(
            tree,
            "Six chapters. The squad you keep is the squad you bring home.",
            theme::SMALL_SIZE,
            theme::DIM_TEXT_COLOR,
        );
        built.continue_button = Some(tree.add_button("Continue"));
        built.start_button = Some(tree.add_button("New run"));
        built.title_quit_button = Some(tree.add_button("Quit"));
    });
    built.title_panel = Some(panel);
}

fn build_pause(tree: &mut UiTreeBuilder<'_, '_>, built: &mut UiHandles) {
    let panel = tree
        .card_node(10.0, 2.0, theme::PANEL_BORDER)
        .color_raw::<UiBase>(theme::PANEL_COLOR)
        .window(Rl(vec2(50.0, 50.0)), Ab(vec2(360.0, 272.0)), Anchor::Center)
        .flow(FlowDirection::Vertical, 20.0, 12.0)
        .with_visible(false)
        .entity();
    tree.in_parent(panel, |tree| {
        add_menu_text(tree, "Paused", theme::HEADING_SIZE, theme::ACCENT_COLOR);
        add_menu_text(
            tree,
            "The hill waits.",
            theme::SMALL_SIZE,
            theme::DIM_TEXT_COLOR,
        );
        built.pause_buttons = [
            Some(tree.add_button("Resume")),
            Some(tree.add_button("Fall back to camp")),
            Some(tree.add_button("Main menu")),
        ];
    });
    built.pause_panel = Some(panel);
}

pub fn refresh_the_pause(
    screen_state: Res<State<Screen>>,
    handles_state: Res<UiHandles>,
    pause_state: Res<Pause>,
    mut interaction: ResMut<RetainedUiInteraction>,
    mut commands: Commands,
) {
    let commands = &mut commands;
    if screen_state.current != Screen::Paused {
        return;
    }
    let handles = handles_state.clone();
    let pause = *pause_state;
    let enabled = pause_items(pause.from);
    for (item, button) in handles.pause_buttons.iter().enumerate() {
        let Some(button) = button else {
            continue;
        };
        let ready = enabled.get(item).copied().unwrap_or(false);
        let color = if !ready {
            theme::PANEL_COLOR
        } else if item == pause.index {
            theme::MENU_SELECTED
        } else {
            theme::MENU_BUTTON
        };
        ui_set_color::<UiBase>(commands, *button, color);
        ui_set_disabled(commands, *button, !ready);
        if ready && item == pause.index {
            ui_focus(&mut interaction, *button);
        }
    }
}

fn build_prologue(tree: &mut UiTreeBuilder<'_, '_>, built: &mut UiHandles) {
    let panel = tree
        .card_node(10.0, 2.0, theme::PANEL_BORDER)
        .color_raw::<UiBase>(theme::PANEL_COLOR)
        .window(
            Rl(vec2(50.0, 50.0)),
            Ab(vec2(theme::PROLOGUE_WIDTH, theme::PROLOGUE_HEIGHT)),
            Anchor::Center,
        )
        .flow(FlowDirection::Vertical, 26.0, 10.0)
        .with_visible(false)
        .entity();
    tree.in_parent(panel, |tree| {
        built.prologue_heading = Some(
            tree.add_node()
                .fill_width()
                .auto_height()
                .with_text("", theme::HEADING_SIZE)
                .text_center()
                .color_raw::<UiBase>(theme::ACCENT_COLOR)
                .entity(),
        );
        for _ in 0..theme::PROLOGUE_LINES {
            let row = tree
                .add_node()
                .fill_width()
                .auto_height()
                .with_text("", theme::SMALL_SIZE)
                .text_center()
                .color_raw::<UiBase>(theme::TEXT_COLOR)
                .entity();
            built.prologue_rows.push(row);
        }
        built.prologue_next = Some(tree.add_button("Continue"));
        built.prologue_skip = Some(tree.add_button("Skip the briefing"));
    });
    built.prologue_panel = Some(panel);
}

fn build_over(tree: &mut UiTreeBuilder<'_, '_>, built: &mut UiHandles) {
    let panel = tree
        .card_node(10.0, 2.0, theme::PANEL_BORDER)
        .color_raw::<UiBase>(theme::PANEL_COLOR)
        .window(
            Rl(vec2(50.0, 50.0)),
            Ab(vec2(theme::OVER_WIDTH, theme::OVER_HEIGHT)),
            Anchor::Center,
        )
        .flow(FlowDirection::Vertical, 24.0, 10.0)
        .with_visible(false)
        .entity();
    tree.in_parent(panel, |tree| {
        built.over_label = Some(
            tree.add_node()
                .fill_width()
                .auto_height()
                .with_text("", theme::HEADING_SIZE)
                .text_center()
                .color_raw::<UiBase>(theme::ACCENT_COLOR)
                .entity(),
        );
        built.over_detail = Some(
            tree.add_node()
                .fill_width()
                .auto_height()
                .with_text("", theme::SMALL_SIZE)
                .text_center()
                .color_raw::<UiBase>(theme::TEXT_COLOR)
                .entity(),
        );
        for _ in 0..theme::REPORT_LINES {
            let row = tree
                .add_node()
                .fill_width()
                .auto_height()
                .with_text("", theme::SMALL_SIZE)
                .text_center()
                .color_raw::<UiBase>(theme::DIM_TEXT_COLOR)
                .entity();
            built.over_rows.push(row);
        }
        built.retry_button = Some(tree.add_button("Return to camp"));
        built.over_quit_button = Some(tree.add_button("Main menu"));
    });
    built.over_panel = Some(panel);
}

pub fn show_the_screen(
    screen_state: Res<State<Screen>>,
    mut handles_state: ResMut<UiHandles>,
    mut dirty_state: ResMut<RetainedUiDirty>,
    mut commands: Commands,
) {
    let commands = &mut commands;
    let screen = screen_state.current;
    let handles = {
        let handles = &mut *handles_state;
        if handles.shown == Some(screen) {
            return;
        }
        handles.shown = Some(screen);
        handles.shown_phase = None;
        handles.shown_camp_phase = None;
        handles.shown_fighter = None;
        handles.predict_visible = false;
        handles.clone()
    };
    let dirty = &mut *dirty_state;
    let camp = screen == Screen::Camp;
    let in_battle = matches!(screen, Screen::Battle | Screen::Paused | Screen::Over);
    let panels = [
        (handles.title_panel, screen == Screen::Title),
        (handles.prologue_panel, screen == Screen::Prologue),
        (handles.pause_panel, screen == Screen::Paused),
        (handles.over_panel, screen == Screen::Over),
        (handles.unit_card, in_battle),
        (handles.tile_card, in_battle),
        (handles.camp_panel, camp),
        (handles.camp_menu_panel, false),
        (handles.speech_panel, false),
        (handles.order_panel, in_battle || camp),
        (handles.log_panel, in_battle || camp),
        (handles.menu_panel, false),
        (handles.ability_panel, false),
        (handles.predict_panel, false),
    ];
    for (panel, visible) in panels {
        if let Some(panel) = panel {
            ui_set_visible(dirty, commands, panel, visible);
        }
    }
    if screen == Screen::Title
        && let Some(button) = handles.continue_button
    {
        let kept = read_the_run().is_some();
        ui_set_disabled(commands, button, !kept);
        ui_set_color::<UiBase>(
            commands,
            button,
            if kept {
                theme::MENU_BUTTON
            } else {
                theme::PANEL_COLOR
            },
        );
    }
}

pub fn refresh_the_prologue(
    screen_state: Res<State<Screen>>,
    campaign: Res<Campaign>,
    mut handles_state: ResMut<UiHandles>,
    mut commands: Commands,
) {
    let commands = &mut commands;
    if screen_state.current != Screen::Prologue {
        return;
    }
    let page = campaign.page.min(PROLOGUE.len() - 1);
    if handles_state.shown_page == Some(page) {
        return;
    }
    handles_state.shown_page = Some(page);
    let handles = handles_state.clone();
    let leaf = &PROLOGUE[page];
    set_text(commands, handles.prologue_heading, leaf.heading);
    for (slot, row) in handles.prologue_rows.iter().enumerate() {
        ui_set_text(commands, *row, leaf.body.get(slot).copied().unwrap_or(""));
    }
    if let Some(button) = handles.prologue_next {
        ui_button_set_text(
            commands,
            button,
            if page + 1 >= PROLOGUE.len() {
                "Take command"
            } else {
                "Continue"
            },
        );
    }
}

pub fn refresh_the_camp(
    screen_state: Res<State<Screen>>,
    campaign: Res<Campaign>,
    camp: Res<Camp>,
    mut handles_state: ResMut<UiHandles>,
    mut dirty_state: ResMut<RetainedUiDirty>,
    mut interaction: ResMut<RetainedUiInteraction>,
    mut commands: Commands,
    mut fighters: crate::queries::Fighters,
) {
    let commands = &mut commands;
    if screen_state.current != Screen::Camp {
        return;
    }
    let squad = squad(&mut fighters);
    let handles = handles_state.clone();
    let (chapter_index, day, actions, supplies, phase, selected, menu_index, story) = {
        let campaign = &*campaign;
        (
            campaign.chapter,
            campaign.day,
            campaign.actions,
            campaign.supplies,
            campaign.phase,
            campaign.selected,
            campaign.menu_index,
            campaign.story.clone(),
        )
    };
    let chapter = &CHAPTERS[chapter_index.min(CHAPTERS.len() - 1)];
    let prompt = camp.prompt.clone();

    set_text(
        commands,
        handles.camp_chapter,
        &format!("Chapter {}   {}", chapter_index + 1, chapter.name),
    );
    set_text(
        commands,
        handles.camp_day,
        &format!(
            "Day {} of {}   Actions {}",
            day.min(chapter.days),
            chapter.days,
            actions
        ),
    );
    set_text(
        commands,
        handles.camp_supplies,
        &format!(
            "Supplies {}   Mustered {} of {}",
            supplies,
            mustered(&squad),
            chapter.deploy
        ),
    );
    set_text(commands, handles.camp_objective, chapter.objective.name());
    let brief = match phase {
        CampPhase::Roam => chapter.intro.to_string(),
        CampPhase::Menu => ACTIVITIES
            .get(menu_index)
            .map_or(String::new(), |activity| activity.hint().to_string()),
        CampPhase::Partner => "Who spars with them?".to_string(),
        CampPhase::Muster => "The roster for the next battle.".to_string(),
        CampPhase::Advance => "A mark of training becomes a lasting gain.".to_string(),
        CampPhase::Speaking => String::new(),
    };
    set_text(commands, handles.camp_brief, &brief);
    let call = match &prompt {
        Prompt::Nothing => String::new(),
        Prompt::Talk(_, name) => format!("Enter   Talk with {name}"),
        Prompt::Enter(kind) => format!("Enter   {}  ({})", kind.call(), kind.name()),
    };
    set_text(commands, handles.camp_prompt, &call);

    let near = match &prompt {
        Prompt::Talk(entity, _) => Some(*entity),
        _ => None,
    };
    let shown = selected
        .or(near)
        .and_then(|entity| fighter_index(&squad, entity));
    if let Some(index) = shown {
        let fighter = &squad[index].1;
        let sheet = fighter.class.sheet();
        let level = level_of(fighter.experience);
        let bond = Bond::of(fighter.rapport);
        set_text(commands, handles.fighter_name, fighter.name);
        set_text(
            commands,
            handles.fighter_class,
            &format!(
                "{}  {}   likes {}",
                sheet.name,
                fighter.title,
                fighter.taste.name()
            ),
        );
        set_text(
            commands,
            handles.fighter_level,
            &format!("Level {}   {} experience", level, fighter.experience),
        );
        set_fill(
            &mut dirty_state,
            commands,
            handles.fighter_level_fill,
            level_share(fighter.experience),
            theme::MP_FILL,
        );
        set_text(
            commands,
            handles.fighter_rapport,
            &format!("Bond with you  {} ({})", bond.name(), bond.grade()),
        );
        set_fill(
            &mut dirty_state,
            commands,
            handles.fighter_rapport_fill,
            bond.share(fighter.rapport),
            theme::ACCENT_COLOR,
        );
        let growth = growth_of(level);
        let earned = rapport_growth(bond) + fighter.boon;
        set_text(
            commands,
            handles.fighter_stats,
            &format!(
                "HP +{}   ATK +{}   DEF +{}   SPD +{}",
                growth.hp + earned.hp,
                growth.attack + earned.attack,
                growth.defense + earned.defense,
                growth.speed + earned.speed
            ),
        );
        let companion = &COMPANIONS[fighter.slot.min(COMPANIONS.len() - 1)];
        set_text(
            commands,
            handles.fighter_arc,
            &format!("{}  {} of 4", companion.arc, fighter.arc_stage),
        );
        let told = fighter.arc_stage.min(4);
        let arc_line = if told == 0 {
            "Their story has not opened yet."
        } else {
            companion.stages[(told - 1) as usize].title
        };
        set_text(commands, handles.fighter_arc_line, arc_line);
        let state = if fighter.wounded > 0 {
            format!("Wounded, {} days of rest", fighter.wounded)
        } else if fighter.busy {
            format!("Busy today   {}", morale_name(fighter.morale))
        } else if fighter.deployed {
            format!(
                "Mustered   {}   {} battles",
                morale_name(fighter.morale),
                fighter.battles
            )
        } else {
            format!("In camp   {}", morale_name(fighter.morale))
        };
        set_text(commands, handles.fighter_state, &state);
        if let Some(label) = handles.fighter_state {
            let color = if fighter.wounded > 0 {
                theme::ENEMY_COLOR
            } else if fighter.deployed {
                theme::PLAYER_COLOR
            } else {
                theme::DIM_TEXT_COLOR
            };
            ui_set_color::<UiBase>(commands, label, color);
        }
        set_bond_row(commands, handles.fighter_bond, &squad, index);
    }
    if handles_state.shown_fighter != Some(near.is_some()) {
        handles_state.shown_fighter = Some(near.is_some());
        let dirty = &mut *dirty_state;
        if let Some(panel) = handles.fighter_panel {
            ui_set_visible(dirty, commands, panel, near.is_some());
        }
    }

    set_text(commands, handles.order_title, "The squad");
    for (slot, row) in handles.order_rows.iter().enumerate() {
        match squad.get(slot) {
            Some((_, fighter)) => {
                let mark = if fighter.wounded > 0 {
                    "x"
                } else if fighter.deployed {
                    "*"
                } else {
                    " "
                };
                ui_set_text(
                    commands,
                    *row,
                    &format!(
                        "{} {}  Lv{}  {}",
                        mark,
                        fighter.name,
                        level_of(fighter.experience),
                        Bond::of(fighter.rapport).grade()
                    ),
                );
                let color = if fighter.wounded > 0 {
                    theme::ENEMY_COLOR
                } else if fighter.deployed {
                    theme::TEXT_COLOR
                } else {
                    theme::DIM_TEXT_COLOR
                };
                ui_set_color::<UiBase>(commands, *row, color);
            }
            None => ui_set_text(commands, *row, ""),
        }
    }

    for (slot, row) in handles.log_rows.iter().enumerate() {
        let start = story.len().saturating_sub(theme::LOG_LINES);
        let text = story.get(start + slot).map_or("", String::as_str);
        ui_set_text(commands, *row, text);
    }
    let holder = selected.and_then(|entity| fighter_index(&squad, entity));
    let title = match phase {
        CampPhase::Muster => "Muster".to_string(),
        CampPhase::Partner => "Who spars?".to_string(),
        CampPhase::Advance => holder.map_or(String::new(), |index| {
            format!("{} advances", squad[index].1.name)
        }),
        _ => holder.map_or(String::new(), |index| {
            format!("{}   {}", squad[index].1.name, squad[index].1.title)
        }),
    };
    set_text(commands, handles.camp_menu_title, &title);
    let rows = menu_rows(&campaign, &squad, holder, chapter.deploy);
    let mut focused = None;
    for (item, button) in handles.camp_menu_buttons.iter().enumerate() {
        let (label, ready) = rows.get(item).cloned().unwrap_or_default();
        ui_button_set_text(commands, *button, &label);
        let color = if !ready {
            theme::PANEL_COLOR
        } else if item == menu_index {
            theme::MENU_SELECTED
        } else {
            theme::MENU_BUTTON
        };
        ui_set_color::<UiBase>(commands, *button, color);
        ui_set_disabled(commands, *button, !ready);
        if ready && item == menu_index {
            focused = Some(*button);
        }
    }
    if handles_state.shown_camp_rows != rows.len() {
        handles_state.shown_camp_rows = rows.len();
        let dirty = &mut *dirty_state;
        set_menu_height(dirty, commands, handles.camp_menu_panel, rows.len());
        show_menu_rows(dirty, commands, &handles.camp_menu_buttons, rows.len());
    }

    if let Some(button) = focused {
        ui_focus(&mut interaction, button);
    }

    let spoken = {
        let campaign = &*campaign;
        campaign.speech.get(campaign.said).cloned()
    };
    let (speaker, line) = spoken.unwrap_or_default();
    set_text(commands, handles.speech_name, &speaker);
    set_text(commands, handles.speech_line, &line);
    set_text(commands, handles.speech_hint, &speech_step(&campaign));

    let shown_phase = handles_state.shown_camp_phase;
    if shown_phase != Some(phase) {
        handles_state.shown_camp_phase = Some(phase);
        let dirty = &mut *dirty_state;
        if let Some(panel) = handles.camp_menu_panel {
            ui_set_visible(
                dirty,
                commands,
                panel,
                !matches!(phase, CampPhase::Roam | CampPhase::Speaking),
            );
        }
        if let Some(panel) = handles.speech_panel {
            ui_set_visible(dirty, commands, panel, phase == CampPhase::Speaking);
        }
    }
}

pub fn refresh(
    screen_state: Res<State<Screen>>,
    battle_state: Res<Battle>,
    campaign: Res<Campaign>,
    cursor: Res<Cursor>,
    terrain: Res<Terrain>,
    mut handles_state: ResMut<UiHandles>,
    mut dirty_state: ResMut<RetainedUiDirty>,
    mut interaction: ResMut<RetainedUiInteraction>,
    mut commands: Commands,
    mut units: crate::queries::Units,
    mut structures: Structures<'_, '_>,
) {
    let commands = &mut commands;
    let screen = screen_state.current;
    if screen == Screen::Title || screen == Screen::Camp {
        return;
    }
    let roster = roster(&mut units);
    let sites = sites(&mut structures);
    let handles = handles_state.clone();
    let mut focused = None;
    let (cursor_column, cursor_row) = (cursor.column, cursor.row);
    let (phase, active, menu_index, ability_index, ability, targets, log, outcome) = {
        let battle = &*battle_state;
        (
            battle.phase,
            battle.active,
            battle.menu_index,
            battle.ability_index,
            battle.ability,
            battle.targets.clone(),
            battle.log.clone(),
            battle.outcome,
        )
    };
    let shown_unit = active
        .and_then(|entity| roster_index(&roster, entity))
        .or_else(|| unit_at(&roster, cursor_column, cursor_row));
    if let Some(index) = shown_unit {
        let unit = &roster[index].1;
        let sheet = unit.class.sheet();
        set_text(commands, handles.unit_name, unit.name);
        set_text(
            commands,
            handles.unit_class,
            &format!(
                "{} {}   {}   CT {}",
                unit.team.name(),
                sheet.name,
                unit.mobility.name(),
                unit.ct
            ),
        );
        set_text(
            commands,
            handles.unit_hp,
            &format!("HP {} / {}", unit.hp.max(0), unit.max_hp),
        );
        let hp_color = if hp_share(unit) < 0.35 {
            theme::HP_LOW
        } else {
            theme::HP_FILL
        };
        set_fill(
            &mut dirty_state,
            commands,
            handles.unit_hp_fill,
            hp_share(unit),
            hp_color,
        );
        set_text(
            commands,
            handles.unit_mp,
            &format!("MP {} / {}", unit.mp, unit.max_mp),
        );
        set_fill(
            &mut dirty_state,
            commands,
            handles.unit_mp_fill,
            mp_share(unit),
            theme::MP_FILL,
        );
        set_text(
            commands,
            handles.unit_stats,
            &format!(
                "ATK {}   DEF {}   FIRE {}   ARM {}",
                unit.attack, unit.defense, unit.magic, unit.resist
            ),
        );
        set_text(
            commands,
            handles.unit_movement,
            &format!(
                "SPD {}   MOV {}   JMP {}   EVA {}",
                unit.speed,
                effective_move(unit),
                unit.jump,
                unit.evasion
            ),
        );
        set_text(commands, handles.unit_status, &status_text(unit));
    }

    {
        let terrain = &terrain;
        set_text(
            commands,
            handles.tile_label,
            &format!(
                "Tile {}, {}   Height {}   {}",
                cursor_column,
                cursor_row,
                crate::queries::terrain::height(terrain, cursor_column, cursor_row),
                terrain_name(terrain, cursor_column, cursor_row)
            ),
        );
    }
    match site_at(&sites, cursor_column, cursor_row) {
        Some(index) => {
            let structure = &sites[index].1;
            let owner = structure.owner.map_or("Neutral", Team::name);
            set_text(
                commands,
                handles.tile_site,
                &format!(
                    "{} ({})   {}",
                    structure.kind.name(),
                    owner,
                    structure.kind.effect()
                ),
            );
        }
        None => set_text(commands, handles.tile_site, ""),
    }
    match unit_at(&roster, cursor_column, cursor_row) {
        Some(index) => {
            let unit = &roster[index].1;
            set_text(
                commands,
                handles.tile_unit,
                &format!(
                    "{}   {} {}",
                    unit.name,
                    unit.team.name(),
                    unit.class.sheet().name
                ),
            );
            if let Some(label) = handles.tile_unit {
                ui_set_color::<UiBase>(commands, label, unit.team.color());
            }
            set_text(
                commands,
                handles.tile_hp,
                &format!(
                    "HP {} / {}   MP {} / {}   CT {}",
                    unit.hp.max(0),
                    unit.max_hp,
                    unit.mp,
                    unit.max_mp,
                    unit.ct
                ),
            );
        }
        None => {
            set_text(commands, handles.tile_unit, "Nobody here");
            if let Some(label) = handles.tile_unit {
                ui_set_color::<UiBase>(commands, label, theme::DIM_TEXT_COLOR);
            }
            set_text(commands, handles.tile_hp, "");
        }
    }

    set_text(commands, handles.order_title, "Turn order");
    let order = turn_order(&roster, theme::ORDER_LENGTH);
    for (slot, row) in handles.order_rows.iter().enumerate() {
        match order
            .get(slot)
            .and_then(|entity| roster_index(&roster, *entity))
        {
            Some(index) => {
                let unit = &roster[index].1;
                ui_set_text(commands, *row, &format!("{}  {}", slot + 1, unit.name));
                ui_set_color::<UiBase>(commands, *row, unit.team.color());
            }
            None => ui_set_text(commands, *row, ""),
        }
    }

    for (slot, row) in handles.log_rows.iter().enumerate() {
        let start = log.len().saturating_sub(theme::LOG_LINES);
        let text = log.get(start + slot).map_or("", String::as_str);
        ui_set_text(commands, *row, text);
    }

    let active_unit = active
        .and_then(|entity| roster_index(&roster, entity))
        .map(|index| roster[index].1.clone());

    if phase == Phase::Menu
        && let Some(unit) = &active_unit
    {
        set_text(
            commands,
            handles.menu_title,
            &format!("{}   {}", unit.name, unit.class.sheet().name),
        );
        let menu_enabled = menu_items(&battle_state, unit, &sites);
        for (item, button) in handles.menu_buttons.iter().enumerate() {
            let enabled = menu_enabled.get(item).copied().unwrap_or(false);
            let color = if !enabled {
                theme::PANEL_COLOR
            } else if item == menu_index {
                theme::MENU_SELECTED
            } else {
                theme::MENU_BUTTON
            };
            ui_set_color::<UiBase>(commands, *button, color);
            ui_set_disabled(commands, *button, !enabled);
            if enabled && item == menu_index {
                focused = Some(*button);
            }
        }
    }

    if phase == Phase::ChooseAbility
        && let Some(unit) = &active_unit
    {
        let sheet = unit.class.sheet();
        set_text(commands, handles.ability_title, sheet.skill_set);
        for (item, button) in handles.ability_buttons.iter().enumerate() {
            match sheet.abilities.get(item) {
                Some(ability) => {
                    let shape = ability.shape();
                    ui_button_set_text(
                        commands,
                        *button,
                        &format!("{}   {} MP", shape.name, shape.mp),
                    );
                    let enabled = shape.mp <= unit.mp
                        && !(battle_state.moved && is_stationary(unit, *ability));
                    let color = if !enabled {
                        theme::PANEL_COLOR
                    } else if item == ability_index {
                        theme::MENU_SELECTED
                    } else {
                        theme::MENU_BUTTON
                    };
                    ui_set_color::<UiBase>(commands, *button, color);
                    ui_set_disabled(commands, *button, !enabled);
                    if enabled && item == ability_index {
                        focused = Some(*button);
                    }
                }
                None => ui_button_set_text(commands, *button, ""),
            }
        }
    }

    if let Some(button) = focused {
        ui_focus(&mut interaction, button);
    }

    let mut predict_shown = false;
    if phase == Phase::ChooseTarget
        && let (Some(unit), Some(ability)) = (&active_unit, ability)
        && targets.contains(&(cursor_column, cursor_row))
    {
        let terrain = &terrain;
        let area = area_tiles(terrain, (cursor_column, cursor_row), ability.shape().area);
        let touched = affected(&roster, &area, unit, ability);
        if let Some(first) = touched.first() {
            predict_shown = true;
            let target = &roster[*first].1;
            let strike = predict(
                terrain,
                &sites,
                &roster,
                unit,
                (unit.column, unit.row),
                target,
                ability,
            );
            let more = if touched.len() > 1 {
                format!("  and {} more", touched.len() - 1)
            } else {
                String::new()
            };
            set_text(
                commands,
                handles.predict_label,
                &format!("{}   HP {}{}", target.name, target.hp.max(0), more),
            );
            let detail = if ability.helps() {
                format!("{}  heals {}", ability.name(), strike.amount)
            } else {
                format!(
                    "{}  {} damage   {}% hit   {}   {}",
                    ability.name(),
                    strike.amount,
                    strike.hit_chance,
                    strike.side.name(),
                    if strike.height_gain > 0 {
                        "high ground"
                    } else if strike.height_gain < 0 {
                        "low ground"
                    } else {
                        "level"
                    }
                )
            };
            set_text(commands, handles.predict_detail, &detail);
        }
    }

    if screen == Screen::Over {
        let (chapter_index, report, finished) = {
            let campaign = &*campaign;
            (
                campaign.chapter,
                campaign.report.clone(),
                campaign.chapter + 1 >= CHAPTERS.len(),
            )
        };
        let chapter = &CHAPTERS[chapter_index.min(CHAPTERS.len() - 1)];
        let won = outcome == Some(Team::Player);
        let label = match (won, finished) {
            (true, true) => "The valley is quiet",
            (true, false) => "Victory",
            _ => "Defeat",
        };
        set_text(commands, handles.over_label, label);
        set_text(
            commands,
            handles.over_detail,
            if won {
                chapter.victory
            } else {
                "The squad falls back to camp and counts the cost."
            },
        );
        for (slot, row) in handles.over_rows.iter().enumerate() {
            let text = report.get(slot).map_or("", String::as_str);
            ui_set_text(commands, *row, text);
        }
        if let Some(button) = handles.retry_button {
            ui_button_set_text(
                commands,
                button,
                match (won, finished) {
                    (true, true) => "Take the title screen",
                    (true, false) => "Return to camp",
                    _ => "Back to camp",
                },
            );
        }
    }

    let (shown_phase, last_predict) = {
        let handles = &*handles_state;
        (handles.shown_phase, handles.predict_visible)
    };
    let predict_key = phase == Phase::ChooseTarget && predict_shown;
    if shown_phase != Some(phase) || predict_key != last_predict {
        {
            let handles = &mut *handles_state;
            handles.shown_phase = Some(phase);
            handles.predict_visible = predict_key;
        }
        let dirty = &mut *dirty_state;
        let in_battle = screen == Screen::Battle;
        let panels = [
            (handles.menu_panel, in_battle && phase == Phase::Menu),
            (
                handles.ability_panel,
                in_battle && phase == Phase::ChooseAbility,
            ),
            (handles.predict_panel, in_battle && predict_key),
        ];
        for (panel, visible) in panels {
            if let Some(panel) = panel {
                ui_set_visible(dirty, commands, panel, visible);
            }
        }
    }
}

fn speech_step(campaign: &Campaign) -> String {
    if campaign.speech.is_empty() {
        return String::new();
    }
    format!("{} of {}", campaign.said + 1, campaign.speech.len())
}
