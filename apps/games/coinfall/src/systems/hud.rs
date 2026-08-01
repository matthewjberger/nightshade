use crate::data;
use crate::resources::{CoinfallResources, HudHandles, Screen};
use crate::theme;
use nightshade::prelude::*;
use nightshade_ui::components::layout_types::FlowDirection;
use nightshade_ui::prelude::*;
use nightshade_ui_banner::prelude::*;
use nightshade_ui_menu::prelude::*;

pub fn build(tree: &mut UiTreeBuilder) -> (HudHandles, BannerHandles, Vec<Entity>) {
    let root = tree
        .add_node()
        .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
        .entity();

    let mut handles = HudHandles::default();
    let mut banner = BannerHandles::default();
    let mut dots = Vec::new();
    tree.in_parent(root, |tree| {
        banner = add_banner(tree, &banner_style());
        build_title(tree, &mut handles);
        build_overworld(tree, &mut handles, &mut dots);
        build_pause(tree, &mut handles);
        build_cleared(tree, &mut handles);
    });
    (handles, banner, dots)
}

fn build_title(tree: &mut UiTreeBuilder, handles: &mut HudHandles) {
    handles.title = veil(tree, true);
    tree.in_parent(handles.title, |tree| {
        let style = title_style();
        let panel = add_menu_panel(tree, &style, theme::TITLE_WIDTH, theme::TITLE_HEIGHT);
        tree.in_parent(panel, |tree| {
            add_menu_heading(tree, &style, "COINFALL");
            rule(tree);
            add_menu_row(tree, &style, "the clock always falls");
            add_menu_row(tree, &style, "coins buy it back");
            rule(tree);
            handles.play_button = tree.add_button("Play");
            handles.quit_button = tree.add_button("Quit");
        });
    });
}

fn build_overworld(tree: &mut UiTreeBuilder, handles: &mut HudHandles, dots: &mut Vec<Entity>) {
    handles.overworld = veil(tree, false);
    tree.in_parent(handles.overworld, |tree| {
        let style = gold_style();
        let panel = add_menu_panel(tree, &style, theme::MAP_WIDTH, theme::MAP_HEIGHT);
        tree.in_parent(panel, |tree| {
            add_menu_heading(tree, &style, "Choose a level");
            rule(tree);
            let count = data::LEVELS.len() as f32;
            let span = count * theme::DOT_SIZE + (count - 1.0) * theme::DOT_GAP;
            let row = tree
                .add_node()
                .flow_child(Ab(vec2(span, theme::DOT_SIZE)))
                .flow(FlowDirection::Horizontal, 0.0, theme::DOT_GAP)
                .entity();
            tree.in_parent(row, |tree| {
                for _ in 0..data::LEVELS.len() {
                    dots.push(
                        tree.add_node()
                            .flow_child(Ab(vec2(theme::DOT_SIZE, theme::DOT_SIZE)))
                            .with_rect(theme::DOT_SIZE * 0.5, 2.0, theme::PANEL_BORDER)
                            .color_raw::<UiBase>(theme::DOT_LOCKED)
                            .entity(),
                    );
                }
            });
            handles.overworld_name = add_menu_row(tree, &menu_style(), "");
            handles.overworld_best = add_menu_row(tree, &dim_style(), "");
            handles.start_button = tree.add_button("Start");
            handles.overworld_title_button = tree.add_button("Title");
        });
    });
}

fn build_pause(tree: &mut UiTreeBuilder, handles: &mut HudHandles) {
    handles.pause = veil(tree, false);
    tree.in_parent(handles.pause, |tree| {
        let style = menu_style();
        let panel = add_menu_panel(tree, &style, theme::MENU_WIDTH, theme::MENU_HEIGHT);
        tree.in_parent(panel, |tree| {
            add_menu_heading(tree, &style, "Paused");
            rule(tree);
            handles.resume_button = tree.add_button("Resume");
            handles.restart_button = tree.add_button("Start over");
            handles.pause_map_button = tree.add_button("Level select");
        });
    });
}

fn build_cleared(tree: &mut UiTreeBuilder, handles: &mut HudHandles) {
    handles.cleared = veil(tree, false);
    tree.in_parent(handles.cleared, |tree| {
        let style = gold_style();
        let panel = add_menu_panel(tree, &style, theme::MENU_WIDTH, theme::MENU_HEIGHT);
        tree.in_parent(panel, |tree| {
            add_menu_heading(tree, &style, "Cleared");
            rule(tree);
            handles.cleared_score = add_menu_row(tree, &dim_style(), "");
            handles.again_button = tree.add_button("Run it again");
            handles.cleared_map_button = tree.add_button("Level select");
        });
    });
}

fn rule(tree: &mut UiTreeBuilder) {
    tree.add_node()
        .flow_child(Ab(vec2(theme::RULE_WIDTH, theme::RULE_HEIGHT)))
        .with_rect(2.0, 0.0, theme::CLEAR)
        .color_raw::<UiBase>(theme::TEXT_GOLD)
        .without_pointer_events();
}

fn veil(tree: &mut UiTreeBuilder, visible: bool) -> Entity {
    tree.add_node()
        .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
        .with_rect(0.0, 0.0, theme::CLEAR)
        .color_raw::<UiBase>(theme::VEIL)
        .with_visible(visible)
        .entity()
}

pub fn show(
    mut resources: ResMut<CoinfallResources>,
    mut banner: ResMut<Banner>,
    mut commands: Commands,
) {
    let handles = resources.hud;
    let seconds = resources.run.seconds.ceil() as u32;
    let coins = resources.run.coins;
    let name = data::LEVELS[resources.run.level].name;
    banner.line = format!("{name}      {seconds}s      {coins} coins");
    banner.color = if resources.run.seconds <= theme::URGENT_SECONDS {
        theme::TEXT_URGENT
    } else {
        theme::TEXT_PRIMARY
    };

    if resources.run.screen == Screen::Cleared && resources.run.score != resources.run.last_score {
        resources.run.last_score = resources.run.score;
        let level = resources.run.level;
        ui_set_text(
            &mut commands,
            handles.cleared_score,
            &format!(
                "score {}      best {}",
                resources.run.score, resources.run.bests[level]
            ),
        );
    }

    if resources.run.selected != resources.run.last_selected {
        resources.run.last_selected = resources.run.selected;
        show_selection(&mut resources, &mut commands);
    }
}

fn show_selection(resources: &mut CoinfallResources, commands: &mut Commands) {
    let selected = resources.run.selected;
    let unlocked = resources.run.unlocked;
    for (index, dot) in resources.dots.iter().enumerate() {
        let color = if index == selected {
            theme::DOT_SELECTED
        } else if index < unlocked {
            theme::DOT_OPEN
        } else {
            theme::DOT_LOCKED
        };
        ui_set_color::<UiBase>(commands, *dot, color);
    }

    let level = &data::LEVELS[selected];
    ui_set_text(
        commands,
        resources.hud.overworld_name,
        &format!("{}      {}s", level.name, level.seconds as u32),
    );
    let best = resources.run.bests[selected];
    let summary = if selected >= unlocked {
        "locked".to_string()
    } else if best > 0 {
        format!("best {best}")
    } else {
        "not cleared".to_string()
    };
    ui_set_text(commands, resources.hud.overworld_best, &summary);
}

pub fn menu_style() -> MenuStyle {
    MenuStyle {
        background: theme::PANEL_BACKGROUND,
        border: theme::PANEL_BORDER,
        corner_radius: 12.0,
        border_width: 2.0,
        padding: theme::ROW_PADDING,
        spacing: theme::ROW_GAP,
        visible: true,
        centered: true,
        heading_size: theme::HEADING_SIZE,
        heading_color: theme::TEXT_PRIMARY,
        row_size: theme::SUBTITLE_SIZE,
        row_color: theme::TEXT_PRIMARY,
        outline: theme::TEXT_OUTLINE,
        outline_width: 1.0,
    }
}

fn title_style() -> MenuStyle {
    MenuStyle {
        heading_size: theme::TITLE_SIZE,
        heading_color: theme::TEXT_GOLD,
        row_color: theme::TEXT_DIM,
        ..menu_style()
    }
}

fn gold_style() -> MenuStyle {
    MenuStyle {
        heading_color: theme::TEXT_GOLD,
        row_size: theme::LABEL_SIZE,
        row_color: theme::TEXT_DIM,
        ..menu_style()
    }
}

fn dim_style() -> MenuStyle {
    MenuStyle {
        row_size: theme::LABEL_SIZE,
        row_color: theme::TEXT_DIM,
        ..menu_style()
    }
}

fn banner_style() -> BannerStyle {
    BannerStyle {
        width: theme::READOUT_WIDTH,
        height: theme::READOUT_HEIGHT,
        margin: theme::READOUT_MARGIN,
        size: theme::READOUT_SIZE,
        color: theme::TEXT_PRIMARY,
        background: theme::PANEL_BACKGROUND,
        border: theme::PANEL_BORDER,
        corner_radius: 10.0,
        border_width: 1.0,
        outline: theme::TEXT_OUTLINE,
        outline_width: 1.0,
    }
}
