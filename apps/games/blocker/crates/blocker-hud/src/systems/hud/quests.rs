use crate::resources::HudBuild;
use crate::resources::QuestPanel;
use crate::theme;
use blocker_pocket::resources::Inventory;
use blocker_quests::data::quests::QUESTS;
use blocker_quests::queries::objective_progress;
use blocker_quests::resources::Journal;
use nalgebra_glm::vec2;
use nightshade::prelude::*;
use nightshade_ui::commands::builder::UiTreeBuilder;
use nightshade_ui::components::TextOverflow;
use nightshade_ui::components::layout_types::FlowDirection;
use nightshade_ui::prelude::*;
use nightshade_ui::resources::RetainedUiDirty;
use nightshade_ui_menu::prelude::add_frame;

pub fn build_quests(tree: &mut UiTreeBuilder<'_, '_>, hud: &mut HudBuild<'_>) {
    let height = 30.0 + theme::CHAT_LINE * theme::QUEST_ROWS as f32;
    hud.quests.panel = tree
        .add_node()
        .window(
            Rl(vec2(100.0, 0.0)) + Ab(vec2(-16.0, 16.0)),
            Ab(vec2(248.0, height)),
            Anchor::TopRight,
        )
        .with_rect(6.0, 1.0, theme::PANEL_BORDER)
        .color_raw::<UiBase>(theme::PANEL_BACKGROUND)
        .flow(FlowDirection::Vertical, 8.0, 3.0)
        .with_visible(false)
        .entity();
    let frame_layer = hud.art.frame;
    tree.in_parent(hud.quests.panel, |tree| {
        add_frame(tree, hud.style, frame_layer);
        hud.quests.title = tree
            .add_node()
            .fill_width()
            .auto_height()
            .with_text("Quests", theme::SMALL_SIZE)
            .text_left()
            .color_raw::<UiBase>(theme::ACCENT_COLOR)
            .entity();
        hud.quests.rows = (0..theme::QUEST_ROWS)
            .map(|_| {
                tree.add_node()
                    .fill_width()
                    .auto_height()
                    .with_text("", theme::SMALL_SIZE)
                    .with_text_overflow(TextOverflow::Ellipsis)
                    .text_left()
                    .color_raw::<UiBase>(theme::TEXT_BRIGHT)
                    .entity()
            })
            .collect();
    });
}

pub fn show_the_quests(
    journal: Res<Journal>,
    inventory: Res<Inventory>,
    mut quests_panel: ResMut<QuestPanel>,
    mut dirty: ResMut<RetainedUiDirty>,
    mut commands: Commands,
) {
    if quests_panel.rows.len() < theme::QUEST_ROWS {
        return;
    }
    let mut lines: Vec<String> = Vec::new();
    for index in journal.active.iter().take(theme::QUEST_ROWS) {
        let quest = &QUESTS[*index];
        let (have, want) = objective_progress(&quest.objective, &inventory, &journal);
        lines.push(format!("{}  {have}/{want}", quest.title));
    }
    if lines == quests_panel.shown {
        return;
    }
    let visible = !lines.is_empty();
    ui_set_visible(&mut dirty, &mut commands, quests_panel.panel, visible);
    quests_panel.shown = lines.clone();
    for index in 0..theme::QUEST_ROWS {
        let text = lines.get(index).map(|line| line.as_str()).unwrap_or("");
        ui_set_text(&mut commands, quests_panel.rows[index], text);
    }
}
